use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn run(host: String, port: u16, input: String) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    println!("Connecting to {}", addr);

    let mut socket = TcpStream::connect(&addr)
        .await
        .context("Failed to connect to server")?;

    socket
        .write_all(input.as_bytes())
        .await
        .context("Error during writting")?;

    socket.flush().await.context("Failed to flush socket")?;
    socket
        .shutdown()
        .await
        .context("Failed to shutdown socket")?;

    Ok(())
}
