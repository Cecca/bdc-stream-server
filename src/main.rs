use rand::prelude::*;
use rand_distr::Geometric;
use rand_xoshiro::Xoshiro256StarStar;
use std::io::Write;
use std::time::Instant;
use tokio::io;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let p = std::env::args().nth(1).unwrap().parse::<f64>().unwrap();
    let distr = Geometric::new(p).unwrap();
    let mut seeder = Xoshiro256StarStar::seed_from_u64(1234);
    let listener = TcpListener::bind("0.0.0.0:9999").await?;

    loop {
        seeder.jump();
        let mut rng = seeder.clone();

        let (socket, client_info) = listener.accept().await?;
        eprintln!("Serving {:?}", client_info);

        tokio::spawn(async move {
            let mut socket = BufWriter::new(socket);
            let mut buf = Vec::new();
            let mut cnt = 0;
            let t_start = Instant::now();
            loop {
                let s: u64 = distr.sample(&mut rng);
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
