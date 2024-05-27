use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_distr::Uniform;
use rand_xoshiro::Xoshiro256StarStar;
use serde::Deserialize;
use std::io::prelude::*;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_utils::RateLimiter;

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct Config {
    /// the port to listen to
    port: usize,
    /// the number of elements to generate
    size: usize,
    /// limit the rate of this stream to this many elements per second
    max_rate: f64,
    /// The pairs in this list denote how many (first element) items should have the given (second
    /// element) proportion in the stream
    proportions: Vec<(usize, f64)>,
    /// if `true`, then the socket first reads an integer to be used as seed.
    /// Otherwise just uses [[DEFAULT_SEED]] as the seed.
    #[serde(default = "Config::default_ask_seed")]
    ask_seed: bool,
    #[serde(default = "Config::default_default_seed")]
    default_seed: u64,
    #[serde(default = "Config::default_random_seed")]
    random_seed: bool,
}

impl Config {
    fn from_file(path: &str) -> Self {
        let mut f = std::fs::File::open(path).unwrap();
        let mut cfg_str = String::new();
        f.read_to_string(&mut cfg_str).unwrap();
        toml::from_str(&cfg_str).unwrap()
    }

    fn default_ask_seed() -> bool {
        false
    }

    fn default_random_seed() -> bool {
        false
    }

    fn default_default_seed() -> u64 {
        1234
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .expect("missing the path to the configuration");

    let config = Arc::new(RwLock::new(Config::from_file(&config_path)));
    let config2 = Arc::clone(&config);

    let _config_reader = tokio::spawn(async move {
        // watch for configuration changes
        eprintln!("watching configuration file {:?}", config_path);
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let c = Config::from_file(&config_path);
            if (*config2.read().await) != c {
                eprintln!("Configuration changed: {:?}", c);
                *config2.write().await = c;
            }
        }
    });
    eprintln!(
        "This instance will ask a seed every time: {}",
        config.read().await.ask_seed
    );

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.read().await.port)).await?;

    loop {
        let (mut socket, client_info) = listener.accept().await?;
        let mut reader = BufReader::new(&mut socket);
        eprintln!("Serving {:?}", client_info,);

        let cfg = config.read().await;
        let m: usize = cfg.size;

        let seed = if cfg.ask_seed {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            let seed = line.trim().parse::<i64>().unwrap_or(1234);
            let seed = seed as u64;
            eprintln!("Seed is {} (line was {})", seed, line);
            seed
        } else if cfg.random_seed {
            rand::thread_rng().sample(Uniform::new(0u64, u64::MAX))
        } else {
            cfg.default_seed
        };
        eprintln!("Using seed {}", seed);
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);

        let elements: Vec<u32> = Uniform::new(0u32, u32::MAX)
            .sample_iter(&mut rng)
            .take(m)
            .collect();
        let mut weights = vec![0.0; m];

        let mut i = 0;
        let mut attributed_weight = 0.0;
        for (n, prop) in &cfg.proportions {
            for _ in 0..*n {
                weights[i] = *prop;
                attributed_weight += *prop;
                i += 1;
            }
        }
        let rem_weight = (1.0 - attributed_weight) / (elements.len() - i) as f64;
        while i < elements.len() {
            weights[i] = rem_weight;
            i += 1;
        }

        let distr = WeightedIndex::new(weights).unwrap();

        let limiter = RateLimiter::new(std::time::Duration::from_secs_f64(
            1.0 / config.read().await.max_rate,
        ));
        tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut cnt = 0;
            let t_start = Instant::now();
            loop {
                let is_err = limiter
                    .throttle(|| async {
                        let s = elements[distr.sample(&mut rng)];
                        buf.clear();
                        writeln!(buf, "{}", s).unwrap();
                        cnt += 1;
                        socket.write_all(&buf).await.is_err()
                    })
                    .await;
                if is_err {
                    break;
                }
            }
            let t_end = Instant::now();
            let throughput = (cnt as f64) / (t_end - t_start).as_secs_f64();
            eprintln!(
                "done serving {:?} (throughput {} nums/sec)",
                client_info, throughput
            );
        });
    }
}
