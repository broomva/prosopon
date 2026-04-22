//! A tiny session helper that mints envelopes for an event stream.
//!
//! A `Session` tracks the session id and monotonic sequence number so agents don't
//! have to thread those manually through every event.

use prosopon_core::{NodeId, ProsoponEvent, Scene, SignalValue, Topic};
use prosopon_protocol::{Envelope, SessionId};

/// A session — owns the session id + seq counter for a single connection.
pub struct Session {
    id: SessionId,
    seq: u64,
}

impl Session {
    /// Create a fresh session.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            seq: 0,
        }
    }

    /// Create a session with a specific id.
    #[must_use]
    pub fn with_id(id: SessionId) -> Self {
        Self { id, seq: 0 }
    }

    /// Session id accessor.
    #[must_use]
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Current sequence number (next envelope will be `seq + 1`).
    #[must_use]
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// Wrap an event in the next envelope.
    pub fn envelope(&mut self, event: ProsoponEvent) -> Envelope {
        self.seq += 1;
        Envelope::new(self.id.clone(), self.seq, event)
    }

    /// Convenience: return an iterator over a single scene-reset envelope.
    pub fn scene_reset_stream(&mut self, scene: Scene) -> impl Iterator<Item = Envelope> + '_ {
        std::iter::once(self.envelope(ProsoponEvent::SceneReset { scene }))
    }

    /// Convenience: a signal-changed envelope.
    pub fn signal(&mut self, topic: impl Into<Topic>, value: SignalValue) -> Envelope {
        self.envelope(ProsoponEvent::SignalChanged {
            topic: topic.into(),
            value,
            ts: chrono::Utc::now(),
        })
    }

    /// Convenience: a heartbeat envelope.
    pub fn heartbeat(&mut self) -> Envelope {
        self.envelope(ProsoponEvent::Heartbeat {
            ts: chrono::Utc::now(),
        })
    }

    /// Convenience: node removal.
    pub fn remove(&mut self, id: impl Into<NodeId>) -> Envelope {
        self.envelope(ProsoponEvent::NodeRemoved { id: id.into() })
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node};

    #[test]
    fn session_mints_monotonic_envelopes() {
        let mut s = Session::new();
        let scene = Scene::new(Node::new(Intent::Empty));
        let e1 = s.envelope(ProsoponEvent::SceneReset {
            scene: scene.clone(),
        });
        let e2 = s.heartbeat();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(e1.session_id, e2.session_id);
    }
}
