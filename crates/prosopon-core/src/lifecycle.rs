//! Temporal state attached to every node — the "when" of the IR.
//!
//! Nodes are not merely present; they are *active for a period*. Compositors use
//! lifecycle to decide fade-in, decay animation, urgency styling, and eviction.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Temporal and attentional state of a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Lifecycle {
    /// When this node was first emitted by the agent.
    pub created_at: DateTime<Utc>,
    /// Optional expiration. Compositors SHOULD remove or fade past this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Attention priority — shapes placement, color, and notification behavior.
    #[serde(default)]
    pub priority: Priority,
    /// Current status in the node's lifecycle.
    #[serde(default)]
    pub status: NodeStatus,
}

impl Lifecycle {
    /// A freshly-created node with normal priority and active status.
    #[must_use]
    pub fn now() -> Self {
        Self {
            created_at: Utc::now(),
            expires_at: None,
            priority: Priority::Normal,
            status: NodeStatus::Active,
        }
    }

    /// Builder: set expiration relative to now.
    #[must_use]
    pub fn with_ttl(mut self, seconds: i64) -> Self {
        self.expires_at = Some(self.created_at + chrono::Duration::seconds(seconds));
        self
    }

    /// Builder: set priority.
    #[must_use]
    pub fn with_priority(mut self, p: Priority) -> Self {
        self.priority = p;
        self
    }

    /// Builder: set status.
    #[must_use]
    pub fn with_status(mut self, s: NodeStatus) -> Self {
        self.status = s;
        self
    }

    /// True if this node should still be rendered at the given time.
    #[must_use]
    pub fn is_live_at(&self, t: DateTime<Utc>) -> bool {
        self.expires_at.is_none_or(|exp| exp > t)
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::now()
    }
}

/// Attention priority of a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Background presence — persistent but non-demanding (e.g. a live metric).
    Ambient,
    /// Default rendering priority.
    #[default]
    Normal,
    /// Requires user attention soon but does not block other flows.
    Urgent,
    /// Halts dependent flows until resolved (e.g. a destructive-action confirm).
    Blocking,
}

/// Discrete lifecycle state of a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeStatus {
    /// Node is live and interactive.
    #[default]
    Active,
    /// Node is waiting on an upstream process (tool call, user response).
    Pending,
    /// Node's intent has been satisfied — useful for collapsing completed items.
    Resolved,
    /// Node failed — `reason` is human-readable and SHOULD be surfaced in the UI.
    Failed { reason: String },
    /// Node is fading out according to a decay curve (0.0 = fresh, 1.0 = gone).
    Decaying { progress: f32 },
}

/// Severity level for confirmation dialogs, warnings, and alerts.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    #[default]
    Notice,
    Warning,
    Danger,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_live_check() {
        let l = Lifecycle::now().with_ttl(60);
        assert!(l.is_live_at(l.created_at + chrono::Duration::seconds(30)));
        assert!(!l.is_live_at(l.created_at + chrono::Duration::seconds(120)));
    }

    #[test]
    fn status_roundtrip() {
        let s = NodeStatus::Failed {
            reason: "timeout".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: NodeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
