use tokio::io;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use rand::prelude::*;
use rand_distr::Geometric;
use rand_xoshiro::Xoshiro256StarStar;

#[tokio::main]
async fn main() -> io::Result<()> {
    let p = std::env::args().nth(1).unwrap().parse::<f64>().unwrap();
    let distr = Geometric::new(p).unwrap();
    let mut seeder = Xoshiro256StarStar::seed_from_u64(1234);
    let listener = TcpListener::bind("0.0.0.0:9999").await?;

    loop {
        seeder.jump();
        let mut rng = seeder.clone();

        let (mut socket, client_info) = listener.accept().await?;
        eprintln!("Serving {:?}", client_info);

        tokio::spawn(async move {
            loop {
                let s: u64 = distr.sample(&mut rng);
                let data = format!("{}\n", s);
                let bytes = data.as_bytes();
                if socket.write_all(bytes).await.is_err() {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
            eprintln!("done serving {:?}", client_info);
        });
    }
}


