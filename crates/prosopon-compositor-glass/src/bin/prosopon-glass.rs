//! `prosopon-glass` — stand up a local glass compositor server against a
//! fixture envelope stream. Useful for dev, goldens, and demo pages.

use clap::{Parser, Subcommand};
use prosopon_compositor_glass::{GlassCompositor, GlassServer, GlassServerConfig};
use prosopon_core::ProsoponEvent;
use prosopon_runtime::Compositor;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prosopon-glass", about = "Prosopon glass compositor server")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Serve the compositor on an HTTP port, optionally replaying a fixture.
    Serve {
        #[arg(long, default_value = "127.0.0.1:4321")]
        addr: String,
        /// Path to a JSONL fixture of ProsoponEvents to replay on start.
        #[arg(long)]
        fixture: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve { addr, fixture } => {
            let config = GlassServerConfig {
                addr: addr.parse()?,
            };
            let server = GlassServer::bind(config).await?;
            println!("prosopon-glass serving at http://{}/", server.local_addr());
            let mut compositor = GlassCompositor::new(server.fanout());
            if let Some(path) = fixture {
                replay_fixture(&mut compositor, &path)?;
            }
            server.serve().await?;
            Ok(())
        }
    }
}

fn replay_fixture(c: &mut GlassCompositor, path: &std::path::Path) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: ProsoponEvent =
            serde_json::from_str(line).map_err(|e| anyhow::anyhow!("line {}: {e}", n + 1))?;
        c.apply(&event)?;
    }
    Ok(())
}
