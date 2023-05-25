use rand::prelude::*;
use rand_distr::{Bernoulli, Uniform, Zipf};
use rand_xoshiro::Xoshiro256StarStar;
use serde::Deserialize;
use std::io::prelude::*;
use std::io::Write;
use std::time::Instant;
use tokio::io;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpListener;

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

    let config = Config::from_file(&config_path);
    let mut seeder = Xoshiro256StarStar::seed_from_u64(1234);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

    loop {
        // Read config, which may have changed
        let config = Config::from_file(&config_path);

        // Set up distributions
        let zipf_n = config.zipf_upper;
        let background = Uniform::new(0, config.uniform_max);
        let balancer = Bernoulli::new(config.prob_of_uniform).unwrap();

        seeder.jump();
        let mut rng = seeder.clone();

        let alpha = Uniform::new_inclusive(config.alpha_min, config.alpha_max).sample(&mut rng);
        let offset = Uniform::new(0, config.max_zipf_offset).sample(&mut rng);
        let distr = Zipf::new(zipf_n, alpha).unwrap();

        let (socket, client_info) = listener.accept().await?;
        eprintln!(
            "Serving {:?} with alpha {} offset {} {:?}",
            client_info, alpha, offset, config
        );

        tokio::spawn(async move {
            let mut socket = BufWriter::new(socket);
            let mut buf = Vec::new();
            let mut cnt = 0;
            let t_start = Instant::now();
            loop {
                let s: u64 = if balancer.sample(&mut rng) {
                    background.sample(&mut rng)
                } else {
                    offset + distr.sample(&mut rng).floor() as u64
                };
                buf.clear();
                writeln!(buf, "{}", s).unwrap();
                cnt += 1;
                if socket.write_all(&buf).await.is_err() {
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
