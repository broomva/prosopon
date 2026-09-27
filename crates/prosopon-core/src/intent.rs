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

    // ───────────────── Filesystem ─────────────────
    /// A filesystem read. The `content` slot fills in when the read completes.
    ///
    /// Lifecycle — mirrors the `ToolCall` pattern:
    /// - `NodeAdded` with `content: None` when the agent decides to read.
    ///   Paired lifecycle status `pending`.
    /// - `NodeUpdated { patch: { intent: FileRead { content: Some(...) } } }`
    ///   when the content lands. Lifecycle moves to `resolved`.
    ///
    /// For very large reads the agent MAY pair this with a sibling `Stream`
    /// intent and populate `content` only on `StreamChunk { final_: true }`.
    FileRead {
        /// Absolute or workspace-relative path. Compositors MAY render it as
        /// a clickable target. No scheme is implied; the emitter decides.
        path: String,
        /// File content once the read completes. Absent while in-flight.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Byte count. Compositors may compute from `content.len()` if the
        /// emitter doesn't supply it; supplying it authoritatively is
        /// preferred for multi-byte encodings.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        /// MIME type hint — enables syntax highlighting and preview-mode
        /// switching. Compositors SHOULD assume `text/plain` when absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime: Option<String>,
    },

    /// A filesystem write. `op` narrows the kind of write (create / write /
    /// append / delete); `content` slots in when the write completes.
    ///
    /// Lifecycle:
    /// 1. `NodeAdded` with `content: None` and `op` set. Lifecycle `pending`.
    /// 2. `NodeUpdated` patching `content` (and `bytes` if the emitter knows
    ///    them) when the write resolves. Lifecycle `resolved` on success,
    ///    `failed` on error.
    ///
    /// For streamed writes, pair with a sibling `Stream` intent and patch
    /// `content` on stream completion.
    FileWrite {
        /// Target path — see `FileRead::path`.
        path: String,
        /// The kind of write. Constrained via `FileWriteKind`.
        op: FileWriteKind,
        /// The full body written. Absent until the write resolves.
        /// SHOULD be absent for `FileWriteKind::Delete`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Byte count — authoritative when supplied, else derive from
        /// `content.len()`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        /// Optional human-readable title the agent chose for this artifact.
        /// Useful when `path` is a synthetic workspace path and `title` is
        /// the user-facing name the compositor should surface.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// MIME type hint, as in `FileRead`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime: Option<String>,
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

/// What kind of filesystem write a `FileWrite` intent represents.
///
/// `#[non_exhaustive]` so future variants (e.g. `Patch` for diff-based writes)
/// land additively.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum FileWriteKind {
    /// New file at `path`. The write MUST fail if the file already exists.
    Create,
    /// Overwrite an existing file at `path`. Compositors MAY render as
    /// a `mod`/`modified` badge.
    Write,
    /// Append content to the end of an existing file.
    Append,
    /// Remove the file at `path`. `content` SHOULD be absent.
    Delete,
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

    #[test]
    fn file_read_pending_roundtrip() {
        // Pending read — `content` + `bytes` + `mime` omitted entirely.
        let i = Intent::FileRead {
            path: "/workspace/notes/hello.md".into(),
            content: None,
            bytes: None,
            mime: None,
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["type"], "file_read");
        assert_eq!(json["path"], "/workspace/notes/hello.md");
        // Optional fields skip serialization when None.
        assert!(json.get("content").is_none());
        assert!(json.get("bytes").is_none());
        assert!(json.get("mime").is_none());
        let back: Intent = serde_json::from_value(json).unwrap();
        assert_eq!(back, i);
    }

    #[test]
    fn file_read_resolved_roundtrip() {
        let i = Intent::FileRead {
            path: "notes/hello.md".into(),
            content: Some("# hi\n".into()),
            bytes: Some(5),
            mime: Some("text/markdown".into()),
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["content"], "# hi\n");
        assert_eq!(json["bytes"], 5);
        assert_eq!(json["mime"], "text/markdown");
        let back: Intent = serde_json::from_value(json).unwrap();
        assert_eq!(back, i);
    }

    #[test]
    fn file_write_create_pending() {
        let i = Intent::FileWrite {
            path: "notes/audit.md".into(),
            op: FileWriteKind::Create,
            content: None,
            bytes: None,
            title: Some("Audit report".into()),
            mime: Some("text/markdown".into()),
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["type"], "file_write");
        assert_eq!(json["op"], "create");
        assert_eq!(json["title"], "Audit report");
        assert!(json.get("content").is_none());
        let back: Intent = serde_json::from_value(json).unwrap();
        assert_eq!(back, i);
    }

    #[test]
    fn file_write_op_kinds() {
        // Each variant serializes to its snake_case name.
        for (kind, name) in [
            (FileWriteKind::Create, "create"),
            (FileWriteKind::Write, "write"),
            (FileWriteKind::Append, "append"),
            (FileWriteKind::Delete, "delete"),
        ] {
            let i = Intent::FileWrite {
                path: "/tmp/x".into(),
                op: kind,
                content: None,
                bytes: None,
                title: None,
                mime: None,
            };
            let json = serde_json::to_value(&i).unwrap();
            assert_eq!(json["op"], name, "op {kind:?} should serialize as {name}");
            let back: Intent = serde_json::from_value(json).unwrap();
            assert_eq!(back, i);
        }
    }

    #[test]
    fn file_write_resolved_with_content() {
        let i = Intent::FileWrite {
            path: "/tmp/note.txt".into(),
            op: FileWriteKind::Write,
            content: Some("resolved body".into()),
            bytes: Some(13),
            title: None,
            mime: None,
        };
        let json = serde_json::to_value(&i).unwrap();
        assert_eq!(json["bytes"], 13);
        let back: Intent = serde_json::from_value(json).unwrap();
        assert_eq!(back, i);
    }
}
