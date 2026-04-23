//! # prosopon-compositor-glass
//!
//! 2D web compositor for Prosopon — serves an embedded Preact bundle over HTTP,
//! streams [`prosopon_protocol::Envelope`] over WebSocket + SSE, and implements
//! [`prosopon_runtime::Compositor`] for in-process use.
//!
//! The web bundle under `web/dist/` is embedded at build time via `include_dir`.
//! Agents can consume this crate three ways:
//!
//! 1. **In-process compositor.** Register a [`GlassCompositor`] on a
//!    [`prosopon_runtime::Runtime`]; envelopes fan out to any connected browser.
//! 2. **Standalone server.** Use the `prosopon-glass` binary (`serve --port 4321`).
//! 3. **Consumed bundle.** Import the `@prosopon/compositor-glass` TS package from
//!    a downstream web app and connect it to any Prosopon-speaking endpoint.
//!
//! See `docs/surfaces/glass.md` for the design note.

#![forbid(unsafe_code)]

pub mod assets;
pub mod compositor;
pub mod fanout;
pub mod server;

pub use compositor::{GlassCompositor, GlassCompositorBuilder};
pub use fanout::{EnvelopeFanout, EnvelopeReceiver};
pub use server::{GlassServer, GlassServerConfig};

/// Version of this compositor crate. Distinct from `PROTOCOL_VERSION` and
/// `IR_SCHEMA_VERSION` — bumps independently.
pub const COMPOSITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
