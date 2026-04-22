//! # prosopon-sdk
//!
//! Ergonomic, agent-facing API for emitting Prosopon IR.
//!
//! The core crate defines the data model; this crate defines *how you build it
//! without ceremony*. Think of it as the `html!` macro analogue: shortcut constructors,
//! builder extensions, and a [`Session`] type that ties an event stream together.
//!
//! ## Example
//!
//! ```
//! use prosopon_sdk::{ir, Session};
//!
//! let scene = ir::section("Analysis")
//!     .child(ir::prose("Inspected 3 entities."))
//!     .child(ir::progress(0.66).label("Scoring"))
//!     .into_scene();
//!
//! let mut session = Session::new();
//! for env in session.scene_reset_stream(scene) {
//!     println!("{}", serde_json::to_string(&env).unwrap());
//! }
//! ```

#![forbid(unsafe_code)]

pub mod ir;
pub mod session;

pub use session::Session;
