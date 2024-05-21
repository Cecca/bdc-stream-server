use rand::prelude::*;
use rand_distr::{Bernoulli, Uniform, Zipf};
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

const DEFAULT_SEED: u64 = 1234;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
struct Config {
    /// the port to listen to
    port: usize,
    zipf_upper: u64,
    alpha_min: f64,
    alpha_max: f64,
    uniform_max: u64,
    prob_of_uniform: f64,
    max_zipf_offset: u64,
    max_rate: f64,
    /// if `true`, then the socket first reads an integer to be used as seed.
    /// Otherwise just uses [[DEFAULT_SEED]] as the seed.
    #[serde(default = "Config::default_ask_seed")]
    ask_seed: bool,
}

impl Config {
    fn from_file(path: &str) -> Self {
        let mut f = std::fs::File::open(path).unwrap();
        let mut cfg_str = String::new();
        f.read_to_string(&mut cfg_str).unwrap();
        toml::from_str(&cfg_str).unwrap()
    }

    fn default_ask_seed() -> bool {
        true
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
        // Set up distributions
        let zipf_n = config.read().await.zipf_upper;
        let background = Uniform::new(0, config.read().await.uniform_max);
        let balancer = Bernoulli::new(config.read().await.prob_of_uniform).unwrap();

        let (mut socket, client_info) = listener.accept().await?;
        let mut reader = BufReader::new(&mut socket);
        eprintln!("Serving {:?}", client_info,);

        let seed = if config.read().await.ask_seed {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            let seed = line.trim().parse::<i64>().unwrap_or(1234);
            let seed = seed as u64;
            eprintln!("Seed is {} (line was {})", seed, line);
            seed
        } else {
            DEFAULT_SEED
        };
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);

        let alpha =
            Uniform::new_inclusive(config.read().await.alpha_min, config.read().await.alpha_max)
                .sample(&mut rng);
        let offset = Uniform::new(0, config.read().await.max_zipf_offset).sample(&mut rng);
        let distr = Zipf::new(zipf_n, alpha).unwrap();

        let limiter = RateLimiter::new(std::time::Duration::from_secs_f64(
            1.0 / config.read().await.max_rate,
        ));
        tokio::spawn(async move {
            // let mut socket = BufWriter::new(socket);
            let mut buf = Vec::new();
            let mut cnt = 0;
            let t_start = Instant::now();
            loop {
                let is_err = limiter
                    .throttle(|| async {
                        let s: u64 = if balancer.sample(&mut rng) {
                            background.sample(&mut rng)
                        } else {
                            offset + distr.sample(&mut rng).floor() as u64
                        };
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
