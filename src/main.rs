mod proto;
mod recv;
mod send;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Recv {
        #[arg(short, long)]
        port: u16,

        #[arg(short, long)]
        output: String,
    },

    Send {
        #[arg( long)]
        host: String,

        #[arg(short, long)]
        port: u16,

        #[arg(short, long)]
        input: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Recv { port, output } => {
            recv::run(*port, output.clone())
                .await
                .context("Failed to receive file")?;
        }
        Commands::Send { host, port, input } => {
            send::run(host.clone(), *port, input.clone())
                .await
                .context("Failed to send file")?;
        }
    }

    Ok(())
}
