use anyhow::Result;
use simpleredis::{network, Backend};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    info!("Simple Redis Server is listening on {}", addr);

    let backend = Backend::new();

    let listener = TcpListener::bind(addr).await?;
    loop {
        let cloned_backend = backend.clone();
        let (stream, raddr) = listener.accept().await?;
        tokio::spawn(async move {
            match network::stream_handler(stream, cloned_backend).await {
                Ok(_) => {
                    info!("Connection from {} exited", raddr);
                }
                Err(e) => {
                    warn!("handle error for {}: {:?}", raddr, e);
                }
            }
        });
    }
}
