use crate::constants;
use anyhow::{bail, Context, Result};
use async_compression::tokio::write::GzipEncoder;
use std::path::Path;
use tokio::fs::metadata;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

enum SendMode {
    Raw,
    Gzip,
}

struct FileInfo<'a> {
    path: &'a Path,
    name: String,
    size: u64,
}

pub async fn run(host: String, port: u16, input: String) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    println!("Connecting to {}", addr);
    let mut socket = TcpStream::connect(&addr)
        .await
        .context("Failed to connect to server")?;

    let file_info = get_file_info(&input).await?;
    let mode = SendMode::Raw;

    write_header(
        &file_info.name,
        file_info.size,
        match mode {
            SendMode::Raw => 0,
            SendMode::Gzip => 1,
        },
        &mut socket,
    )
    .await
    .context("Failed to send file header")?;

    match mode {
        SendMode::Raw => {
            write_file_stream(&file_info.path, &mut socket)
                .await
                .context("Failed to send file data")?;
        }
        SendMode::Gzip => {
            let mut encoder = GzipEncoder::new(&mut socket);
            write_file_stream(&file_info.path, &mut encoder)
                .await
                .context("Failed to send compressed file data")?;
            encoder
                .shutdown()
                .await
                .context("Failed to shutdown encoder")?;
        }
    }

    write_file_stream(&file_info.path, &mut socket)
        .await
        .context("Failed to send file data")?;

    socket
        .shutdown()
        .await
        .context("Failed to shutdown socket")?;

    Ok(())
}

async fn get_file_info(input: &String) -> Result<FileInfo<'_>> {
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

    let file_name = path
        .file_name()
        .context("Failed to get file name from path")?;

    let file_size = target_metadata.len();

    Ok(FileInfo {
        path,
        name: file_name.to_string_lossy().to_string(),
        size: file_size,
    })
}

async fn write_header(
    file_name: &String,
    file_size: u64,
    compression: u64,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut header = vec![0u8; constants::PACKAGE_HEADER_SIZE];

    let filename_len = file_name.len() as u64;
    header[0..8].copy_from_slice(&filename_len.to_be_bytes());
    header[8..16].copy_from_slice(&file_size.to_be_bytes());
    header[16..24].copy_from_slice(&compression.to_be_bytes());

    socket
        .write_all(&header)
        .await
        .context("Failed to write header")?;

    socket
        .write_all(file_name.as_bytes())
        .await
        .context("Failed to write filename")?;

    Ok(())
}

async fn write_file_stream<W>(path: &Path, writer: &mut W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut file = File::open(path)
        .await
        .context("Failed to read input file")?;

    let mut buffer = vec![0u8; constants::CHUNK_SIZE];

    loop {
        let n = file.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        writer
            .write_all(&buffer[..n])
            .await
            .context("Failed to write data to socket")?;
    }

    writer.flush().await.context("Failed to flush writer")?;

    Ok(())
}
