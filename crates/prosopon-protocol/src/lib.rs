//! # prosopon-protocol
//!
//! Wire envelope, codecs, and version negotiation for the Prosopon display server.
//!
//! The protocol layer wraps [`prosopon_core::ProsoponEvent`] in an `Envelope` carrying
//! the protocol version, a session id, a monotonic sequence number, and a timestamp.
//! Compositors and agents negotiate by exchanging `Hello` frames before streaming
//! events.
//!
//! ## Design
//!
//! Wayland's lesson: a small core protocol + versioned extensions. We version the
//! envelope shape itself with `PROTOCOL_VERSION`; `Intent` extensibility is handled by
//! [`prosopon_core::Intent`]'s `#[non_exhaustive]` + `Custom` escape hatch, not by
//! bumping this version.
//!
//! Two codecs ship:
//!
//! - [`Codec::Json`]     — single JSON document per envelope, for request/response.
//! - [`Codec::Jsonl`]    — line-delimited JSON, for streaming. One envelope per line.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use prosopon_core::ProsoponEvent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current wire version. Bump on any backwards-incompatible envelope change.
pub const PROTOCOL_VERSION: u32 = 1;

/// Stable identifier for a connection between an agent and a compositor.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

impl SessionId {
    /// Mint a new random session id.
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Wire frame wrapping a single event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version — MUST equal `PROTOCOL_VERSION` on the receiving end.
    pub version: u32,
    /// Session this envelope belongs to.
    pub session_id: SessionId,
    /// Monotonic, per-session sequence number (start at 1, gap = packet loss).
    pub seq: u64,
    /// When this envelope was produced.
    pub ts: DateTime<Utc>,
    /// The event itself.
    pub event: ProsoponEvent,
}

impl Envelope {
    /// Wrap an event in a new envelope for a session with the next sequence number.
    #[must_use]
    pub fn new(session_id: SessionId, seq: u64, event: ProsoponEvent) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            session_id,
            seq,
            ts: Utc::now(),
            event,
        }
    }
}

/// First frame of a session. Sent by the agent to the compositor (or vice versa)
/// to announce capabilities and negotiate the wire version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    /// Highest protocol version supported.
    pub max_version: u32,
    /// Human-readable agent name (e.g. `"arcan/0.9.2"`, `"mission-control/web"`).
    pub agent: String,
    /// Semantic role of this endpoint.
    pub role: PeerRole,
    /// Self-described capabilities (what surfaces, what encodings).
    pub capabilities: PeerCapabilities,
}

/// Whether this endpoint emits IR or renders it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerRole {
    /// The agent — produces events, consumes actions.
    Agent,
    /// A compositor — consumes events, produces actions.
    Compositor,
}

/// Capabilities advertised at handshake time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PeerCapabilities {
    /// Surfaces this compositor can render onto (empty for agents).
    #[serde(default)]
    pub surfaces: Vec<prosopon_core::SurfaceKind>,
    /// Supported wire codecs.
    #[serde(default = "default_codecs")]
    pub codecs: Vec<Codec>,
    /// Whether signal values can be pushed (subscription).
    #[serde(default)]
    pub supports_signal_push: bool,
    /// Whether streaming chunks are supported.
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
}

fn default_codecs() -> Vec<Codec> {
    vec![Codec::Json, Codec::Jsonl]
}

fn default_true() -> bool {
    true
}

/// Supported wire codecs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    /// Single JSON document per envelope.
    Json,
    /// Newline-delimited JSON (one envelope per line, trailing `\n`).
    Jsonl,
}

impl Codec {
    /// Encode a single envelope. For `Jsonl`, appends a trailing newline.
    ///
    /// # Errors
    /// Returns [`ProtocolError::Serialization`] if serde_json serialization fails.
    pub fn encode(&self, env: &Envelope) -> Result<Vec<u8>, ProtocolError> {
        match self {
            Self::Json => serde_json::to_vec(env).map_err(Into::into),
            Self::Jsonl => {
                let mut bytes = serde_json::to_vec(env)?;
                bytes.push(b'\n');
                Ok(bytes)
            }
        }
    }

    /// Decode a single envelope from a byte slice. For `Jsonl`, trailing whitespace
    /// and newlines are permitted.
    ///
    /// # Errors
    /// Returns [`ProtocolError::Serialization`] if parsing fails, or
    /// [`ProtocolError::VersionMismatch`] if the envelope version does not match
    /// [`PROTOCOL_VERSION`].
    pub fn decode(&self, bytes: &[u8]) -> Result<Envelope, ProtocolError> {
        let trimmed: &[u8] = match self {
            Self::Jsonl => trim_trailing_newline(bytes),
            Self::Json => bytes,
        };
        let env: Envelope = serde_json::from_slice(trimmed)?;
        if env.version != PROTOCOL_VERSION {
            return Err(ProtocolError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: env.version,
            });
        }
        Ok(env)
    }
}

fn trim_trailing_newline(b: &[u8]) -> &[u8] {
    let mut end = b.len();
    while end > 0 && matches!(b[end - 1], b'\n' | b'\r' | b' ' | b'\t') {
        end -= 1;
    }
    &b[..end]
}

/// Errors produced by the protocol layer.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    #[error("envelope rejected: {0}")]
    Rejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node, Scene};

    fn make_event() -> ProsoponEvent {
        let scene = Scene::new(Node::new(Intent::Prose {
            text: "hello".into(),
        }));
        ProsoponEvent::SceneReset { scene }
    }

    #[test]
    fn json_roundtrip() {
        let env = Envelope::new(SessionId::new(), 1, make_event());
        let bytes = Codec::Json.encode(&env).unwrap();
        let back = Codec::Json.decode(&bytes).unwrap();
        assert_eq!(back, env);
    }

    #[test]
    fn jsonl_roundtrip_with_newline() {
        let env = Envelope::new(SessionId::new(), 1, make_event());
        let bytes = Codec::Jsonl.encode(&env).unwrap();
        assert_eq!(*bytes.last().unwrap(), b'\n');
        let back = Codec::Jsonl.decode(&bytes).unwrap();
        assert_eq!(back, env);
    }

    #[test]
    fn version_mismatch_detected() {
        let mut env = Envelope::new(SessionId::new(), 1, make_event());
        env.version = 999;
        let bytes = serde_json::to_vec(&env).unwrap();
        let err = Codec::Json.decode(&bytes).unwrap_err();
        assert!(matches!(err, ProtocolError::VersionMismatch { .. }));
    }

    #[test]
    fn hello_serializes() {
        let h = Hello {
            max_version: PROTOCOL_VERSION,
            agent: "arcan/0.9.2".into(),
            role: PeerRole::Agent,
            capabilities: PeerCapabilities::default(),
        };
        let json = serde_json::to_value(&h).unwrap();
        assert_eq!(json["role"], "agent");
        assert_eq!(json["max_version"], PROTOCOL_VERSION);
    }
}
