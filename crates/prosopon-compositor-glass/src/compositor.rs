//! `GlassCompositor` — implements `prosopon_runtime::Compositor` by forwarding
//! every envelope into an [`EnvelopeFanout`] for browser clients. In "detached"
//! mode (no fanout) all events are accepted and dropped; this mode exists for
//! tests and for embedding the compositor before an HTTP listener is bound.

use prosopon_core::{ProsoponEvent, SurfaceKind};
use prosopon_protocol::{Envelope, SessionId};
use prosopon_runtime::{Capabilities, Compositor, CompositorError, CompositorId};
use std::sync::atomic::{AtomicU64, Ordering};

use prosopon_daemon::EnvelopeFanout;

/// A glass-surface compositor.
pub struct GlassCompositor {
    id: CompositorId,
    session: SessionId,
    seq: AtomicU64,
    fanout: Option<EnvelopeFanout>,
}

impl GlassCompositor {
    /// Create a compositor wired to an [`EnvelopeFanout`].
    #[must_use]
    pub fn new(fanout: EnvelopeFanout) -> Self {
        Self {
            id: CompositorId::new("prosopon-compositor-glass"),
            session: SessionId::new(),
            seq: AtomicU64::new(1),
            fanout: Some(fanout),
        }
    }

    /// Create a compositor with no connected fanout. Useful for tests and for
    /// programmatic registration before the server has been bound.
    #[must_use]
    pub fn detached() -> Self {
        Self {
            id: CompositorId::new("prosopon-compositor-glass"),
            session: SessionId::new(),
            seq: AtomicU64::new(1),
            fanout: None,
        }
    }

    fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::Relaxed)
    }
}

impl Compositor for GlassCompositor {
    fn id(&self) -> CompositorId {
        self.id.clone()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            surfaces: vec![SurfaceKind::TwoD],
            max_fps: Some(60),
            supports_signal_push: true,
            supports_streaming: true,
        }
    }

    fn apply(&mut self, event: &ProsoponEvent) -> Result<(), CompositorError> {
        let Some(fanout) = &self.fanout else {
            return Ok(());
        };
        let envelope = Envelope::new(self.session.clone(), self.next_seq(), event.clone());
        if let Err(e) = fanout.send(envelope) {
            tracing::debug!(target: "prosopon::glass", "no subscribers: {e}");
        }
        Ok(())
    }
}

/// Builder for `GlassCompositor` configured with a specific session id. Reserved
/// for future multi-session use; v0.2 always mints a fresh id.
pub struct GlassCompositorBuilder;
