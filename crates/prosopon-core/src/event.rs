//! `ProsoponEvent` — the unit of communication between the agent and compositors.
//!
//! Events are intentionally small and additive: a full `Scene` is only sent on reset;
//! subsequent changes flow as `NodeAdded`, `NodeUpdated`, `NodeRemoved`, and
//! `SignalChanged`. This gives the wire a "CRDT-lite" feel without locking us to
//! any specific CRDT implementation.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::ActionKind;
use crate::ids::{ActionId, NodeId, StreamId, Topic};
use crate::node::{Node, NodePatch};
use crate::scene::Scene;
use crate::signal::SignalValue;
use crate::value::Value;

/// A single event in the Prosopon stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProsoponEvent {
    /// Full scene reset. Compositors SHOULD clear state and re-render.
    SceneReset { scene: Scene },
    /// New node attached under a parent. If the parent does not exist, compositors
    /// MAY buffer or drop; implementations SHOULD log.
    NodeAdded { parent: NodeId, node: Node },
    /// Incremental patch to an existing node.
    NodeUpdated { id: NodeId, patch: NodePatch },
    /// Remove a node and its subtree.
    NodeRemoved { id: NodeId },
    /// A signal's current value changed.
    SignalChanged {
        topic: Topic,
        value: SignalValue,
        ts: DateTime<Utc>,
    },
    /// A chunk of streaming output for a `Stream` intent.
    StreamChunk { id: StreamId, chunk: StreamChunk },
    /// The user (via a compositor) invoked an action slot.
    /// Flow direction: compositor -> agent.
    ActionEmitted {
        slot: ActionId,
        source: NodeId,
        kind: ActionKind,
    },
    /// Periodic liveness. Compositors MAY use this for "last-heard-from" timeouts.
    Heartbeat { ts: DateTime<Utc> },
}

/// A chunk of streaming payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StreamChunk {
    /// Sequence number (monotonic per stream). Compositors drop out-of-order chunks.
    pub seq: u64,
    /// Encoded payload. Interpretation depends on the `StreamKind` of the originating
    /// `Stream` intent.
    pub payload: ChunkPayload,
    /// True if this is the last chunk of the stream.
    #[serde(default)]
    pub final_: bool,
}

/// Payload variants carried in a stream chunk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "encoding", rename_all = "snake_case")]
pub enum ChunkPayload {
    /// UTF-8 text fragment — append to the rendered stream.
    Text { text: String },
    /// Base64-encoded bytes (audio frames, binary data).
    B64 { data: String, mime: Option<String> },
    /// A JSON value — for JSONL streams.
    Json { value: Value },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Intent;

    #[test]
    fn event_tagging_is_snake_case() {
        let e = ProsoponEvent::Heartbeat { ts: Utc::now() };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["type"], "heartbeat");
    }

    #[test]
    fn stream_chunk_roundtrip() {
        let c = StreamChunk {
            seq: 7,
            payload: ChunkPayload::Text {
                text: "hello".into(),
            },
            final_: false,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: StreamChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn scene_reset_roundtrip() {
        let scene = Scene::new(Node::new(Intent::Prose { text: "hi".into() }));
        let e = ProsoponEvent::SceneReset { scene };
        let json = serde_json::to_string(&e).unwrap();
        let back: ProsoponEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }
}
