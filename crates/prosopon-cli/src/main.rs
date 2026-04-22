//! # prosopon (CLI)
//!
//! Inspection, validation, and demo commands for the Prosopon display server.

#![forbid(unsafe_code)]

use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use prosopon_compositor_text::{RenderOptions, TextCompositor, TextTarget, render_scene};
use prosopon_core::{IR_SCHEMA_VERSION, Intent, Node, Scene, event_schema_json, scene_schema_json};
use prosopon_protocol::{Codec, PROTOCOL_VERSION};
use prosopon_runtime::Compositor;
use prosopon_sdk::ir;

/// Top-level CLI.
#[derive(Parser)]
#[command(
    name = "prosopon",
    version,
    about = "Inspect, validate, and demo the Prosopon display-server IR",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print schema JSON for scenes or events.
    Schema {
        /// Which schema to print.
        #[arg(value_enum)]
        target: SchemaTarget,
    },
    /// Validate an IR document (scene or envelope) from a file or stdin.
    Validate {
        /// Input file. Use `-` or omit for stdin.
        #[arg(default_value = "-")]
        input: String,

        /// What kind of document to validate.
        #[arg(long, value_enum, default_value_t = DocKind::Scene)]
        kind: DocKind,
    },
    /// Render an IR document to the terminal via the text compositor.
    Inspect {
        /// Input file. Use `-` or omit for stdin.
        #[arg(default_value = "-")]
        input: String,

        /// Terminal width in columns.
        #[arg(long, default_value_t = 80)]
        width: u16,

        /// Strip ANSI styling (good for captures).
        #[arg(long)]
        plain: bool,
    },
    /// Render a small demo scene to the terminal.
    Demo {
        /// Terminal width.
        #[arg(long, default_value_t = 80)]
        width: u16,
    },
    /// Print repo-level version info.
    Info,
}

#[derive(Copy, Clone, clap::ValueEnum)]
enum SchemaTarget {
    Scene,
    Event,
}

#[derive(Copy, Clone, clap::ValueEnum)]
enum DocKind {
    Scene,
    Envelope,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Schema { target } => cmd_schema(target),
        Cmd::Validate { input, kind } => cmd_validate(&input, kind),
        Cmd::Inspect {
            input,
            width,
            plain,
        } => cmd_inspect(&input, width, plain),
        Cmd::Demo { width } => cmd_demo(width),
        Cmd::Info => cmd_info(),
    }
}

fn cmd_schema(target: SchemaTarget) -> Result<()> {
    let json = match target {
        SchemaTarget::Scene => scene_schema_json(),
        SchemaTarget::Event => event_schema_json(),
    };
    println!("{json}");
    Ok(())
}

fn cmd_validate(input: &str, kind: DocKind) -> Result<()> {
    let bytes = read_input(input)?;
    match kind {
        DocKind::Scene => {
            serde_json::from_slice::<Scene>(&bytes).context("scene failed to parse")?;
        }
        DocKind::Envelope => {
            Codec::Json
                .decode(&bytes)
                .context("envelope failed to parse")?;
        }
    }
    eprintln!(
        "ok: document is a valid {kind:?}",
        kind = match kind {
            DocKind::Scene => "scene",
            DocKind::Envelope => "envelope",
        }
    );
    Ok(())
}

fn cmd_inspect(input: &str, width: u16, plain: bool) -> Result<()> {
    let bytes = read_input(input)?;
    let scene: Scene =
        serde_json::from_slice(&bytes).context("input is not a Scene JSON document")?;
    let opts = if plain {
        RenderOptions::plain()
    } else {
        RenderOptions::default()
    };
    let rendered = render_scene(&scene, width, &opts);
    print!("{rendered}");
    Ok(())
}

fn cmd_demo(width: u16) -> Result<()> {
    let scene: Scene = ir::section("Prosopon demo")
        .child(ir::prose(
            "One IR, many faces. Agents emit intent; compositors render the shape \
             they can produce. This demo runs the reference text compositor.",
        ))
        .child(ir::divider())
        .child(
            ir::list()
                .child(ir::prose("A prose child."))
                .child(ir::tool_call(
                    "search",
                    prosopon_core::json!({"q": "prosopon"}),
                ))
                .child(ir::tool_result(
                    true,
                    prosopon_core::json!(["prosopon-core", "prosopon-runtime"]),
                ))
                .child(ir::progress(0.66).label("scoring")),
        )
        .child(ir::divider())
        .child(
            ir::choice("Approve the new surface?")
                .option("y", "Approve")
                .default()
                .option("n", "Reject"),
        )
        .into_scene();

    let mut compositor = TextCompositor::new(TextTarget::stdout(), width);
    compositor
        .apply(&prosopon_core::ProsoponEvent::SceneReset { scene })
        .context("failed to apply demo scene")?;
    compositor.flush().ok();
    Ok(())
}

fn cmd_info() -> Result<()> {
    println!("prosopon-cli {}", env!("CARGO_PKG_VERSION"));
    println!("ir schema   : {IR_SCHEMA_VERSION}");
    println!("wire protocol: v{PROTOCOL_VERSION}");
    Ok(())
}

fn read_input(input: &str) -> Result<Vec<u8>> {
    if input == "-" {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .context("reading from stdin")?;
        Ok(buf)
    } else {
        std::fs::read(PathBuf::from(input)).with_context(|| format!("reading file {input}"))
    }
}

// Silence the "unused import" warning for `Intent`/`Node` — they come in via ir::*
// but not every subcommand references them directly.
#[allow(dead_code)]
fn _force_imports(_a: Intent, _b: Node) {}
