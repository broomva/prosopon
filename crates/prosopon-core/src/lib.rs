//! # prosopon-core
//!
//! The semantic intermediate representation (IR) for the Prosopon display server.
//!
//! ## What this crate is
//!
//! Agents emit **intent**, compositors render **appearance**. This crate defines the
//! intent side: a tree of `Node`s, each carrying an `Intent`, optional reactive
//! `Binding`s, and optional interactive `ActionSlot`s, assembled into `Scene`s and
//! updated via `ProsoponEvent` streams.
//!
//! Every type round-trips through JSON losslessly and publishes a JSON Schema (via
//! `schemars`) so compositors written in any language can consume the IR.
//!
//! ## Design axioms
//!
//! 1. **Intent-heavy, appearance-light.** No color, typography, or layout constants
//!    live here — those are compositor concerns.
//! 2. **Additive-first.** `Intent` and `ProsoponEvent` are `#[non_exhaustive]` so
//!    new variants do not break existing consumers.
//! 3. **JSON-native.** `Value` is `serde_json::Value`; every field uses `serde_json`'s
//!    field-tagging conventions (`{"kind": "prose", "text": "…"}`).
//! 4. **Compositor-polymorphic.** The same IR targets text, 2D, 3D, shader, audio,
//!    spatial, and tactile surfaces. See [`SurfaceKind`][scene::SurfaceKind].
//!
//! ## Quickstart
//!
//! ```rust
//! use prosopon_core::prelude::*;
//!
//! let scene = Scene::new(
//!     Node::new(Intent::Section {
//!         title: Some("Hello, agent".into()),
//!         collapsible: false,
//!     })
//!     .child(Node::new(Intent::Prose {
//!         text: "I inspected 3 entities.".into(),
//!     }))
//!     .child(Node::new(Intent::Progress {
//!         pct: Some(0.66),
//!         label: Some("Scoring".into()),
//!     })),
//! );
//! let json = serde_json::to_string_pretty(&scene).unwrap();
//! assert!(json.contains("\"type\": \"section\""));
//! ```
//!
//! ## Companion crates
//!
//! - `prosopon-protocol` — wire envelope, codecs, version negotiation.
//! - `prosopon-runtime` — reactive signal graph and compositor registry.
//! - `prosopon-compositor-text` — reference ANSI/Pretext-style text compositor.
//! - `prosopon-sdk` — ergonomic agent-facing builders.
//! - `prosopon-pneuma` — `Pneuma<B = L0ToExternal>` binding for the Life Agent OS.

#![forbid(unsafe_code)]

pub mod action;
pub mod error;
pub mod event;
pub mod ids;
pub mod intent;
pub mod lifecycle;
pub mod node;
pub mod scene;
pub mod signal;
pub mod value;

/// Re-exports for ergonomic `use prosopon_core::prelude::*;`.
pub mod prelude {
    pub use crate::action::{ActionKind, ActionSlot, Valence, Visibility};
    pub use crate::error::{CoreError, Result};
    pub use crate::event::{ChunkPayload, ProsoponEvent, StreamChunk};
    pub use crate::ids::{ActionId, NodeId, SceneId, StreamId, Topic};
    pub use crate::intent::{
        ChoiceOption, FormationKind, GroupKind, InputKind, Intent, Projection, SignalDisplay,
        SpatialFrame, StreamKind,
    };
    pub use crate::lifecycle::{Lifecycle, NodeStatus, Priority, Severity};
    pub use crate::node::{ChildrenPatch, Node, NodePatch};
    pub use crate::scene::{Density, IntentProfile, Scene, SceneHints, SurfaceKind, Viewport};
    pub use crate::signal::{BindTarget, Binding, SignalRef, SignalValue, TimePoint, Transform};
    pub use crate::value::{Value, json};
}

pub use prelude::*;

/// Semantic version string of the IR schema. Bump on any wire-incompatible change.
pub const IR_SCHEMA_VERSION: &str = "0.1.0";

/// Emit the JSON Schema for the top-level `Scene` type, as a JSON string.
///
/// Compositors in other languages can consume this to generate typed bindings.
#[must_use]
pub fn scene_schema_json() -> String {
    let schema = schemars::schema_for!(Scene);
    serde_json::to_string_pretty(&schema).expect("schema is always serializable")
}

/// Emit the JSON Schema for `ProsoponEvent`.
#[must_use]
pub fn event_schema_json() -> String {
    let schema = schemars::schema_for!(ProsoponEvent);
    serde_json::to_string_pretty(&schema).expect("schema is always serializable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn scene_schema_emits_valid_json() {
        let s = scene_schema_json();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("$schema").is_some() || v.get("title").is_some());
    }

    #[test]
    fn quickstart_example_roundtrips() {
        let scene = Scene::new(
            Node::new(Intent::Section {
                title: Some("Hello".into()),
                collapsible: false,
            })
            .child(Node::new(Intent::Prose {
                text: "I inspected 3 entities.".into(),
            })),
        );
        let json = serde_json::to_string(&scene).unwrap();
        assert!(json.contains("\"type\":\"section\""));
        let back: Scene = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, scene.id);
        assert_eq!(back.root.children.len(), 1);
    }
}
