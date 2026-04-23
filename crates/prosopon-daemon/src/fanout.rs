//! `EnvelopeFanout` — a cloneable tokio broadcast sender that duplicates each
//! [`Envelope`] to every connected browser client.
//!
//! Receivers lag silently; the fanout uses a bounded channel (capacity 1024)
//! and slow clients observe `RecvError::Lagged` — they reconnect via SSE or
//! re-subscribe via WS.

use prosopon_protocol::Envelope;
use thiserror::Error;
use tokio::sync::broadcast;

const FANOUT_CAPACITY: usize = 1024;

/// Publisher side of the fanout — held by the compositor.
#[derive(Clone)]
pub struct EnvelopeFanout {
    tx: broadcast::Sender<Envelope>,
}

impl Default for EnvelopeFanout {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeFanout {
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(FANOUT_CAPACITY);
        Self { tx }
    }

    /// Create a new receiver. Receivers see envelopes sent after they
    /// subscribe; they do NOT replay the historical scene. The HTTP handler
    /// snapshots the current scene from the runtime and sends it as the first
    /// frame to bridge the gap.
    pub fn subscribe(&self) -> EnvelopeReceiver {
        EnvelopeReceiver {
            rx: self.tx.subscribe(),
        }
    }

    /// Publish an envelope. Returns the number of currently connected
    /// subscribers; a value of 0 is NOT an error.
    ///
    /// # Errors
    /// Never returns `Err` in the current implementation; the signature
    /// preserves room for future flow-control failures.
    pub fn send(&self, envelope: Envelope) -> Result<usize, FanoutError> {
        match self.tx.send(envelope) {
            Ok(n) => Ok(n),
            Err(_) => Ok(0),
        }
    }
}

/// Subscriber side — one per HTTP connection.
pub struct EnvelopeReceiver {
    rx: broadcast::Receiver<Envelope>,
}

impl EnvelopeReceiver {
    /// Await the next envelope.
    ///
    /// # Errors
    /// Returns `FanoutError::Closed` if the publisher side dropped, or
    /// `FanoutError::Lagged(n)` if this subscriber fell too far behind.
    pub async fn recv(&mut self) -> Result<Envelope, FanoutError> {
        self.rx.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Closed => FanoutError::Closed,
            broadcast::error::RecvError::Lagged(n) => FanoutError::Lagged(n),
        })
    }
}

/// Errors surfaced by the fanout.
#[derive(Debug, Error)]
pub enum FanoutError {
    #[error("fanout channel closed")]
    Closed,
    #[error("subscriber lagged by {0} envelopes — reconnect required")]
    Lagged(u64),
}
