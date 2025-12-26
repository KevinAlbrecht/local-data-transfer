use anyhow::{bail, Context, Result};
use std::path::Path;
use tokio::fs::metadata;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn run(host: String, port: u16, input: String) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    println!("Connecting to {}", addr);
    let mut socket = TcpStream::connect(&addr)
        .await
        .context("Failed to connect to server")?;

    let path = get_path(&input).await?;
    write_in_chunks(path, &mut socket)
        .await
        .context("Failed to send file data")?;

    socket
        .shutdown()
        .await
        .context("Failed to shutdown socket")?;

    Ok(())
}

async fn get_path(input: &String) -> Result<&std::path::Path> {
    let path = Path::new(input);
    if !path.exists() {
        bail!("Input path does not exist");
    }

    let target_metadata = metadata(path)
        .await
        .context("Failed to get file metadata")?;

    if !target_metadata.is_file() {
        bail!("Only files are supported atm");
    }

    Ok(path)
}

async fn write_in_chunks(path: &Path, socket: &mut TcpStream) -> Result<()> {
    let mut file = File::open(path)
        .await
        .context("Failed to read input file")?;

    let chunk_size = 64 * 1024;
    let mut buffer = vec![0u8; chunk_size];

    loop {
        let n = file.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        buffer.truncate(n);
        socket
            .write_all(&buffer)
            .await
            .context("Failed to write data to socket")?;
    }

    Ok(())
}
