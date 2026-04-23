//! # prosopon-daemon
//!
//! Reusable HTTP + WebSocket + SSE transport for the Prosopon ecosystem.
//! Every compositor (glass, text-web, field-shader) registers an optional
//! asset bundle; every emitter (arcan, lago, vigil) publishes envelopes
//! into a shared [`EnvelopeFanout`]. The daemon is the single endpoint
//! browsers connect to.
//!
//! See `docs/rfcs/0002-compositor-contract.md` and the BRO-768 plan.

#![forbid(unsafe_code)]

pub mod fanout;
pub mod server;
pub mod surface;

pub use fanout::{EnvelopeFanout, EnvelopeReceiver, FanoutError};
pub use server::{DaemonConfig, DaemonServer};
pub use surface::SurfaceBundle;

/// Version of this daemon crate.
pub const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");
