//! # prosopon-pneuma
//!
//! The Pneuma-family binding for Prosopon — establishes Prosopon as the
//! `Pneuma<B = L0ToExternal>` implementation (plant → observer), the outward-facing
//! mirror of Sensorium's `Pneuma<ExternalToL0>` (world → plant).
//!
//! ## Status
//!
//! As of 2026-04-21, the Pneuma trait itself does not yet live in
//! `aios-protocol` (the canonical spec is at
//! `core/life/docs/specs/pneuma-trait-surface.md`). This crate defines a *local*
//! mirror of the trait plus boundary markers so Prosopon can ship before the
//! upstream trait lands. When `aios_protocol::pneuma` appears, this crate MUST
//! replace its local definitions with `pub use aios_protocol::pneuma::*;` and
//! bump to 0.2.0.
//!
//! Tracking: [docs/rfcs/0003-pneuma-binding.md](../docs/rfcs/0003-pneuma-binding.md).
//!
//! ## Why this matters
//!
//! The Pneuma trait family formalizes inter-boundary flow in the RCS hierarchy.
//! Prosopon is not "a UI library" — it is a substrate-level sibling to Sensorium,
//! closing the loop between the Life Agent OS and the humans who observe it.
//!
//! ```text
//! Sensorium : Pneuma<ExternalToL0>  (world → plant)   — the eyes
//! Prosopon  : Pneuma<L0ToExternal>  (plant → observer) — the face
//! ```

#![forbid(unsafe_code)]

use prosopon_core::ProsoponEvent;
use prosopon_core::Scene;
use prosopon_core::action::ActionKind;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Axis discriminator for Pneuma boundaries — vertical (level↔level) or horizontal
/// (depth↔depth within a horizontal composition).
pub trait Axis: 'static {}

/// Vertical axis (L_n ↔ L_{n+1}).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vertical;
impl Axis for Vertical {}

/// Horizontal axis (D_k ↔ D_{k+1}).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Horizontal;
impl Axis for Horizontal {}

/// Pneuma boundary marker — a zero-sized type parameter that encodes
/// "at which boundary does this Pneuma implementation live?"
pub trait Boundary: 'static {
    /// The recursion axis this boundary belongs to.
    type A: Axis;
    /// Human-readable identifier, for logs and registry entries.
    const NAME: &'static str;
}

/// The outward-facing boundary where plant-level activity becomes observable.
///
/// This is the mirror of [`ExternalToL0`] (Sensorium). Where Sensorium ingests the
/// world into the plant, `L0ToExternal` emits plant-level activity outward.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct L0ToExternal;

impl Boundary for L0ToExternal {
    type A = Vertical;
    const NAME: &'static str = "l0_to_external";
}

/// The inward-facing boundary Sensorium owns — declared here for completeness.
/// Prosopon does NOT implement this; see `core/life/crates/sensorium/*`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExternalToL0;

impl Boundary for ExternalToL0 {
    type A = Vertical;
    const NAME: &'static str = "external_to_l0";
}

/// Substrate metadata a Pneuma impl advertises at registration time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubstrateProfile {
    /// What kind of substrate we sit on — informs depth-(k+1) planners about cost
    /// and reliability assumptions.
    pub kind: SubstrateKind,
    /// Informal warp factor on throughput/latency vs. reference classical silicon.
    /// `1.0` = parity; `<1.0` = slower; `>1.0` = faster.
    pub warp: f64,
    /// Rough resource ceiling, in substrate-native units. `None` = unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<f64>,
}

/// Enumerated substrate kinds. Open; add variants via `Custom`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubstrateKind {
    /// Standard CPU/GPU servers.
    ClassicalSilicon,
    /// Neuromorphic hardware.
    Neuromorphic,
    /// Quantum backend.
    Quantum,
    /// Biological / wet substrate (e.g. organoid compute).
    Biological,
    /// Human-in-the-loop fallback.
    Human,
    /// Escape hatch for anything else. `label` identifies the substrate in logs.
    Custom {
        /// Human-readable identifier for this custom substrate.
        label: String,
    },
}

impl SubstrateProfile {
    /// Default profile — classical silicon at parity.
    #[must_use]
    pub fn classical() -> Self {
        Self {
            kind: SubstrateKind::ClassicalSilicon,
            warp: 1.0,
            ceiling: None,
        }
    }
}

/// The Pneuma trait — local mirror of the `aios-protocol` spec.
///
/// See module docs for status and upstream migration plan.
pub trait Pneuma: Send + Sync {
    /// Compile-time boundary marker.
    type B: Boundary;
    /// Upward-flowing typed payload — what this boundary emits to the upper level.
    type Signal: Send + Sync + 'static;
    /// Upward-readable state snapshot — what the upper level can poll.
    type Aggregate: Send + Sync + 'static;
    /// Downward-flowing control input — what the upper level sends back.
    type Directive: Send + Sync + 'static;

    /// Emit a signal upward. Compositors and relays receive this.
    ///
    /// # Errors
    /// Returns [`PneumaError`] on publication failure.
    fn emit(&self, signal: Self::Signal) -> Result<(), PneumaError>;

    /// Produce the current upward-readable aggregate.
    fn aggregate(&self) -> Self::Aggregate;

    /// Optionally receive a pending directive from above. Non-blocking.
    fn receive(&self) -> Option<Self::Directive>;

    /// Substrate metadata for scheduling/planning.
    fn substrate(&self) -> SubstrateProfile;
}

/// Errors from Pneuma impls.
#[derive(Debug, Error)]
pub enum PneumaError {
    /// The upstream channel was closed (shutdown or crash).
    #[error("channel closed")]
    Closed,

    /// Backpressure — this many signals were dropped before acceptance resumed.
    #[error("backpressure — dropped {0} signals")]
    Dropped(usize),

    /// Implementation-internal error; message describes the cause.
    #[error("internal: {0}")]
    Internal(String),
}

// ─────────────────────────────────────────────────────────────
// Prosopon-specific associated types
// ─────────────────────────────────────────────────────────────

/// What Prosopon emits outward: individual events that a compositor consumes.
pub type ProsoponSignal = ProsoponEvent;

/// What Prosopon exposes as its current aggregate state to upper levels: the current
/// `Scene` (the "rendered view" of the agent right now).
pub type ProsoponAggregate = Scene;

/// What upper levels can direct inward: an action emitted by the user via a
/// compositor. The directive path is narrower than the full event stream — only
/// user-originated actions flow back down.
pub type ProsoponDirective = ActionKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_names_are_stable() {
        assert_eq!(L0ToExternal::NAME, "l0_to_external");
        assert_eq!(ExternalToL0::NAME, "external_to_l0");
    }

    #[test]
    fn substrate_classical_defaults() {
        let p = SubstrateProfile::classical();
        assert_eq!(p.warp, 1.0);
        assert!(matches!(p.kind, SubstrateKind::ClassicalSilicon));
    }
}
