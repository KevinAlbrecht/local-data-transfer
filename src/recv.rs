use anyhow::{Context, Result};
use get_if_addrs::get_if_addrs;
use std::error::Error;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

pub async fn run(port: u16, output: String) -> Result<()> {
    let ip = match get_local_ip() {
        Ok(ip) => ip,
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };

    let addr = format!("{}:{}", ip, port);
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .context("Failed to bind TCP listener")?;

    println!("Client connected");

    let (socket, _) = listener
        .accept()
        .await
        .context("Failed to accept incoming connection")?;

    read_buffer(socket, output)
        .await
        .context("Failed to read data from socket")?;

    Ok(())
}

async fn read_buffer(mut socket: TcpStream, output: String) -> Result<()> {
    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer).await?;

    println!("Received {} bytes", buffer.len());

    write_in_file(output, &buffer).await?;

    Ok(())
}

fn get_local_ip() -> Result<String, Box<dyn Error>> {
    let if_addrs = get_if_addrs()?;
    for iface in if_addrs {
        if !iface.is_loopback() && iface.ip().is_ipv4() {
            return Ok(iface.ip().to_string());
        }
    }
    Err("No non-loopback IPv4 address found".into())
}

async fn write_in_file(output: String, data: &[u8]) -> Result<()> {
    let path = Path::new(&output);

    let file_path = if path.is_dir() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("output_{}.txt", timestamp);
        path.join(filename)
    } else {
        path.to_path_buf()
    };

    let text = String::from_utf8_lossy(data);

    tokio::fs::write(&file_path, text.as_bytes())
        .await
        .context("Failed to write data to file")?;

    println!("File saved to: {}", file_path.display());
    Ok(())
}
