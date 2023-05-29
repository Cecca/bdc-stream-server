use rand::prelude::*;
use rand_distr::{Bernoulli, Uniform, Zipf};
use rand_xoshiro::Xoshiro256StarStar;
use serde::Deserialize;
use std::io::prelude::*;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tokio::io;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_utils::RateLimiter;

#[derive(Deserialize, Debug, Clone, Copy)]
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
}

impl Config {
    fn from_file(path: &str) -> Self {
        let mut f = std::fs::File::open(path).unwrap();
        let mut cfg_str = String::new();
        f.read_to_string(&mut cfg_str).unwrap();
        toml::from_str(&cfg_str).unwrap()
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
        eprintln!("watching configuration file");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let c = Config::from_file(&config_path);
            *config2.write().await = c;
        }
    });

    let mut seeder = Xoshiro256StarStar::seed_from_u64(1234);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.read().await.port)).await?;

    loop {
        // Set up distributions
        let zipf_n = config.read().await.zipf_upper;
        let background = Uniform::new(0, config.read().await.uniform_max);
        let balancer = Bernoulli::new(config.read().await.prob_of_uniform).unwrap();

        seeder.jump();
        let mut rng = seeder.clone();

        let alpha =
            Uniform::new_inclusive(config.read().await.alpha_min, config.read().await.alpha_max)
                .sample(&mut rng);
        let offset = Uniform::new(0, config.read().await.max_zipf_offset).sample(&mut rng);
        let distr = Zipf::new(zipf_n, alpha).unwrap();

        let (mut socket, client_info) = listener.accept().await?;
        eprintln!(
            "Serving {:?} with alpha {} offset {} {:?}",
            client_info, alpha, offset, config
        );

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
