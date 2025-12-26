use crate::constants;
use crate::utils;
use anyhow::{bail, Context, Result};
use get_if_addrs::get_if_addrs;
use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(port: u16, output: String, create_output: bool) -> Result<()> {
    check_output_path(&output, create_output)
        .await
        .context("Output path check failed")?;

    let ip = match get_local_ip() {
        Ok(ip) => ip,
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };

    let addr = format!("{}:{}", ip, port);
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .context("Failed to bind TCP listener")?;

    let (mut socket, _) = listener
        .accept()
        .await
        .context("Failed to accept incoming connection")?;

    println!("Client connected");

    let (filename, _file_size) = read_header(&mut socket)
        .await
        .context("Failed to read file header")?;

    let (file, temp_file_name) = init_output_file(&output, &filename)
        .await
        .context("Failed to initialize output file")?;

    read_buffer(socket, file)
        .await
        .context("Failed to read data from socket")?;

    finalize_output_file(output, temp_file_name, filename)
        .await
        .context("Failed to finalize output file")?;

    Ok(())
}

async fn check_output_path(output: &String, should_create: bool) -> Result<()> {
    let path = Path::new(output);
    if path.exists() && path.is_dir() {
        Ok(())
    } else if !path.exists() {
        if should_create {
            tokio::fs::create_dir_all(path)
                .await
                .context("Failed to create output directory")?;
            Ok(())
        } else {
            bail!("Output does not exist");
        }
    } else {
        bail!("Output is not a directory");
    }
}

async fn read_header(socket: &mut TcpStream) -> Result<(String, u64)> {
    let mut header = vec![0u8; constants::PACKAGE_HEADER_SIZE];
    socket.read_exact(&mut header).await?;

    let file_name_len = u64::from_be_bytes(header[0..8].try_into().unwrap()) as usize;
    let file_size = u64::from_be_bytes(header[8..16].try_into().unwrap());

    let mut file_name_buf = vec![0u8; file_name_len];
    socket.read_exact(&mut file_name_buf).await?;
    let file_name = String::from_utf8_lossy(&file_name_buf).to_string();
    println!("Receiving file: {} ({} bytes)", file_name, file_size);

    Ok((file_name, file_size))
}

async fn read_buffer(mut socket: TcpStream, mut file: File) -> Result<()> {
    let mut buffer = vec![0u8; constants::CHUNK_SIZE];

    loop {
        let n = socket.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        file.write_all(&buffer[..n])
            .await
            .context("Failed to write data to file")?;
    }

    Ok(())
}

async fn init_output_file(output_dir: &String, file_name: &String) -> Result<(File, String)> {
    let mut temp_file_name = format!("{}/{}.dat", output_dir, file_name);

    if Path::new(&temp_file_name).exists() {
        let timestamp = utils::get_timestamp();
        temp_file_name = format!("{}/{}_{}.dat", output_dir, file_name, timestamp);
    }

    let file = tokio::fs::File::create(&temp_file_name)
        .await
        .context("Failed to create output file")?;
    Ok((file, temp_file_name))
}

async fn finalize_output_file(
    output_path: String,
    temp_file_name: String,
    original_file_name: String,
) -> Result<()> {
    let mut final_name = original_file_name;

    let get_final_path = |file_name: &str| -> String { format!("{}/{}", output_path, file_name) };

    if Path::new(&get_final_path(&final_name)).exists() {
        let timestamp = utils::get_timestamp();
        let path = Path::new(&final_name);
        let name_without_extension = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&final_name);
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        final_name = format!("{}_{}.{}", name_without_extension, timestamp, extension);
    }

    let from = temp_file_name;
    let to = get_final_path(&final_name);

    tokio::fs::rename(&from, &to).await.context(format!(
        "Failed to rename output file from {} to {} with path: {}",
        from, to, output_path
    ))?;

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
