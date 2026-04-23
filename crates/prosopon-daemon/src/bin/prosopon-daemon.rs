//! Standalone daemon binary. Starts a DaemonServer with no surface bundle —
//! useful for headless deployments where the UI lives elsewhere (e.g. a
//! browser-hosted bundle connecting over CORS). For a UI-attached daemon,
//! use a compositor-specific binary such as `prosopon-glass`.

use clap::{Parser, Subcommand};
use prosopon_daemon::{DaemonConfig, DaemonServer};

#[derive(Parser)]
#[command(name = "prosopon-daemon", about = "Prosopon daemon — HTTP/WS/SSE transport")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Serve {
        #[arg(long, default_value = "127.0.0.1:4321")]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve { addr } => {
            let config = DaemonConfig {
                addr: addr.parse()?,
                surface: None,
            };
            let server = DaemonServer::bind(config).await?;
            println!("prosopon-daemon serving at http://{}/", server.local_addr());
            server.serve().await?;
            Ok(())
        }
    }
}
