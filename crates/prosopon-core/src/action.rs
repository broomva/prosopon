//! User-to-agent input surfaces.
//!
//! `ActionSlot` declares "a user can do this thing here." When the user invokes it,
//! the compositor emits `ProsoponEvent::ActionEmitted` back up the boundary.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{ActionId, NodeId};
use crate::value::Value;

/// An interactive slot attached to a node.
///
/// Slots are *declarative* — the node says "this action is available," the compositor
/// decides how to present it (button, keybind, voice command, gesture), and the
/// agent reacts to the emitted `ActionEvent`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionSlot {
    pub id: ActionId,
    pub kind: ActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub visibility: Visibility,
}

fn default_true() -> bool {
    true
}

impl ActionSlot {
    #[must_use]
    pub fn new(kind: ActionKind) -> Self {
        Self {
            id: ActionId::new(),
            kind,
            label: None,
            enabled: true,
            visibility: Visibility::Visible,
        }
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.visibility = Visibility::Hidden;
        self
    }
}

/// What the action *means*. Compositors choose representation; agents react by semantics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionKind {
    /// Submit a payload back to the agent — the most generic action.
    Submit { payload: Value },
    /// Request the agent inspect another node (drill-down, "tell me more").
    Inspect { target: NodeId },
    /// Move focus to another node without side effects.
    Focus { target: NodeId },
    /// Invoke a named command on the agent (e.g. "rerun", "cancel").
    Invoke {
        command: String,
        #[serde(default)]
        args: Value,
    },
    /// Record human feedback on a result — thumb-up/down plus optional comment.
    Feedback {
        valence: Valence,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
    },
    /// Pick one of the options declared by a `Choice` intent.
    Choose { option_id: String },
    /// Fill an `Input` intent's value.
    Input { value: Value },
    /// Confirm a `Confirm` intent.
    Confirm { accepted: bool },
}

/// Qualitative feedback direction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Valence {
    Positive,
    Neutral,
    Negative,
}

/// Whether this slot should be offered to the user right now.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Visible,
    /// Rendered but visually de-emphasized.
    Muted,
    /// Not rendered; still in the IR for programmatic inspection.
    Hidden,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_slot_builder() {
        let s = ActionSlot::new(ActionKind::Invoke {
            command: "retry".into(),
            args: Value::Null,
        })
        .with_label("Retry")
        .disabled();
        assert_eq!(s.label.as_deref(), Some("Retry"));
        assert!(!s.enabled);
    }

    #[test]
    fn action_kind_tagged_enum() {
        let a = ActionKind::Feedback {
            valence: Valence::Positive,
            comment: None,
        };
        let json = serde_json::to_value(&a).unwrap();
        assert_eq!(json["kind"], "feedback");
        assert_eq!(json["valence"], "positive");
    }
}
