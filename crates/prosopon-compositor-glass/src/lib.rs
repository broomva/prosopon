//! # prosopon-compositor-glass
//!
//! 2D web compositor for Prosopon — Arcan Glass-styled Preact bundle.
//!
//! Register with a [`prosopon_daemon::DaemonServer`]:
//!
//! ```no_run
//! use prosopon_compositor_glass::{GlassCompositor, glass_surface};
//! use prosopon_daemon::{DaemonConfig, DaemonServer};
//!
//! # async fn run() -> anyhow::Result<()> {
//! let server = DaemonServer::bind(DaemonConfig {
//!     addr: "127.0.0.1:4321".parse()?,
//!     surface: Some(glass_surface()),
//! }).await?;
//! let _compositor = GlassCompositor::new(server.fanout());
//! server.serve().await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod compositor;
pub mod surface;

pub use compositor::{GlassCompositor, GlassCompositorBuilder};
pub use surface::glass_surface;

// Back-compat re-exports so consumers don't break.
pub use prosopon_daemon::{EnvelopeFanout, EnvelopeReceiver, FanoutError};

pub const COMPOSITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
