//! The `Intent` enum — the semantic core of the Prosopon IR.
//!
//! `Intent` answers "what IS this node?" — not "how should it look." Every compositor
//! receives the same `Intent` tree and decides its own representation (text, 2D, 3D,
//! shader, audio, spatial, tactile).
//!
//! ## Design principles
//!
//! 1. **Intent-heavy, appearance-light.** No `color`, `font`, `padding` here. Those are
//!    compositor concerns.
//! 2. **Orthogonal categories.** Textual, entity, live, decision, process, structural,
//!    spatial, media, meta. New categories extend via `Custom`.
//! 3. **Typed where it pays.** Enums for finite choice sets (`GroupKind`, `SeverityKind`);
//!    free-form `Value` for domain payloads.
//! 4. **Additive compatibility.** `#[non_exhaustive]` on open enums so new variants are
//!    non-breaking.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{StreamId, Topic};
use crate::lifecycle::Severity;
use crate::value::Value;

/// The semantic type of a node.
///
/// See module docs for the design philosophy. Compositors SHOULD handle every variant
/// gracefully; falling back to rendering via `Intent::Prose` of the variant's `Display`
/// is acceptable when no better representation is available.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intent {
    // ─────────────────── Textual ────────────────────
    /// Plain semantic prose. Inline markup is NOT interpreted — compositors format
    /// according to the surrounding node's attributes.
    Prose { text: String },
    /// Source code in a named language. Compositors MAY syntax-highlight.
    Code { lang: String, source: String },
    /// Mathematical expression in LaTeX or MathML. Optional; text compositors fall
    /// back to raw source.
    Math { source: String },

    // ──────────────── Entities & references ─────────────────
    /// A pointer to a known entity in the agent's knowledge graph.
    /// Compositors MAY render as a clickable reference.
    EntityRef {
        kind: String,
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// A hyperlink to an external resource.
    Link {
        href: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// A citation reference — source + optional anchor/page.
    Citation {
        source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<String>,
    },

    // ───────────────── Live state ─────────────────
    /// Displays the current value of a reactive signal.
    Signal {
        topic: Topic,
        #[serde(default)]
        display: SignalDisplay,
    },
    /// A streaming output channel (token stream, audio stream, etc.).
    Stream { id: StreamId, kind: StreamKind },

    // ───────────────── Decision surface ─────────────────
    /// Present a choice between mutually exclusive options.
    Choice {
        prompt: String,
        options: Vec<ChoiceOption>,
    },
    /// Request yes/no confirmation before the agent proceeds.
    Confirm {
        message: String,
        #[serde(default)]
        severity: Severity,
    },
    /// Request typed input from the human.
    Input {
        prompt: String,
        input: InputKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<Value>,
    },

    // ───────────────── Process / tool ─────────────────
    /// An in-flight or completed tool invocation.
    ToolCall {
        name: String,
        args: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stream: Option<StreamId>,
    },
    /// The outcome of a tool call.
    ToolResult { success: bool, payload: Value },
    /// Bounded progress — optional percentage + optional label.
    Progress {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pct: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },

    // ───────────────── Structural ─────────────────
    /// A container whose `GroupKind` tells the compositor how to lay out children.
    Group { layout: GroupKind },
    /// A named section — compositors MAY render a heading; children are the body.
    Section {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default)]
        collapsible: bool,
    },
    /// A visual break between siblings.
    Divider,

    // ───────────────── Spatial / field ─────────────────
    /// A scalar or vector field, indexed by coordinate, projected for the chosen surface.
    Field {
        topic: Topic,
        projection: Projection,
    },
    /// A named spatial anchor in the given frame of reference.
    Locus {
        frame: SpatialFrame,
        position: [f32; 3],
    },
    /// A coherent arrangement of agents (quorum, swarm, formation).
    Formation { topic: Topic, kind: FormationKind },

    // ───────────────── Media ─────────────────
    /// A bitmap or vector image.
    Image {
        uri: String,
        #[serde(default)]
        alt: String,
    },
    /// An audio source. Either a pre-rendered URI, a live stream, or a TTS voice.
    Audio {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stream: Option<StreamId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        voice: Option<String>,
    },
    /// A video source.
    Video {
        uri: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        poster: Option<String>,
    },

    // ───────────────── Meta / escape hatch ─────────────────
    /// Intentionally empty (useful as a container whose content will stream in).
    Empty,
    /// Compositor-specific or experimental intents. Compositors that don't know `kind`
    /// SHOULD fall back to rendering a generic placeholder with the payload preview.
    Custom { kind: String, payload: Value },
}

// ───────────────── Supporting enums ─────────────────

/// How a `Group` lays out its children.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    /// Top-to-bottom list.
    #[default]
    List,
    /// Gridded layout — compositors decide column count.
    Grid,
    /// Strictly ordered sequence — render as a process or timeline.
    Sequence,
    /// Concurrent/parallel items — render side-by-side or overlapped.
    Parallel,
    /// A stack — only the top child is fully visible.
    Stack,
}

/// How a `Signal` intent is surfaced.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SignalDisplay {
    /// Just the current value, formatted by the compositor.
    #[default]
    Inline,
    /// A miniature plot (sparkline, bar).
    Sparkline,
    /// A badge (colored chip with value).
    Badge,
}

/// The content of a `Stream` channel.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamKind {
    /// Token-by-token text (LLM generation, log tail).
    Text,
    /// Audio PCM or encoded.
    Audio,
    /// Arbitrary binary frames.
    Binary,
    /// Structured JSON lines.
    Jsonl,
}

/// One of the options presented by a `Choice` intent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ChoiceOption {
    /// Stable identifier used in the corresponding `ActionKind::Choose`.
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub default: bool,
}

/// Shape of an `Input` intent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputKind {
    Text {
        #[serde(default)]
        multiline: bool,
    },
    Number {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    Boolean,
    Date,
    Json,
}

/// How a field is projected into the compositor's surface.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    /// Flat 2D heatmap.
    Heatmap,
    /// Contour lines.
    Contour,
    /// 3D volume rendering — only honored by 3D compositors.
    Volume,
    /// Reduced to a scalar summary (min/mean/max) by the compositor.
    Summary,
}

/// Frame of reference for a spatial locus.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpatialFrame {
    /// Viewer-centric — `position` is in the compositor's viewport space.
    #[default]
    Viewer,
    /// Scene-world — `position` is in the shared world coordinate system.
    World,
    /// Earth-fixed — `position` is (lat, lon, alt).
    Geo,
}

/// A coherent grouping of agents — renders as a cluster, swarm, or quorum.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FormationKind {
    /// Threshold-reached consensus.
    Quorum,
    /// Moving coordinated group.
    Swarm,
    /// Static ring, line, or geometric arrangement.
    Geometric,
    /// Leftover scent traces (stigmergic).
    Stigmergy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_roundtrip_tagged() {
        let i = Intent::ToolCall {
            name: "search".into(),
            args: serde_json::json!({"q": "prosopon"}),
            stream: None,
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["type"], "tool_call");
        let back: Intent = serde_json::from_value(json).unwrap();
        assert_eq!(back, i);
    }

    #[test]
    fn choice_with_options() {
        let i = Intent::Choice {
            prompt: "Approve migration?".into(),
            options: vec![
                ChoiceOption {
                    id: "yes".into(),
                    label: "Approve".into(),
                    description: None,
                    default: true,
                },
                ChoiceOption {
                    id: "no".into(),
                    label: "Reject".into(),
                    description: Some("Abort the rollout".into()),
                    default: false,
                },
            ],
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["options"][0]["default"], true);
        assert_eq!(json["options"][1]["description"], "Abort the rollout");
    }

    #[test]
    fn field_projection() {
        let i = Intent::Field {
            topic: Topic::new("plexus.load"),
            projection: Projection::Heatmap,
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["projection"], "heatmap");
    }
}
