//! Shortcut constructors for common IR patterns.
//!
//! Every function here returns a [`NodeBuilder`] so you can chain `.child()`,
//! `.bind()`, `.action()`, `.attr()`, etc. Call `.into()` / `.build()` to get a
//! `Node`, or `.into_scene()` to wrap in a `Scene`.

use prosopon_core::{
    ActionKind, ActionSlot, Binding, ChoiceOption, FileWriteKind, GroupKind, Intent, Node, NodeId,
    Priority, Scene, Severity, SignalRef, StreamId, Topic, Value,
};

/// Wraps a `Node` with a fluent interface for agents.
pub struct NodeBuilder(Node);

impl NodeBuilder {
    /// Construct from a raw `Node`.
    #[must_use]
    pub fn from_node(n: Node) -> Self {
        Self(n)
    }

    /// Assign an explicit id.
    #[must_use]
    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.0.id = id.into();
        self
    }

    /// Append a child node (anything `Into<Node>`, including `NodeBuilder`).
    #[must_use]
    pub fn child(mut self, c: impl Into<Node>) -> Self {
        self.0.children.push(c.into());
        self
    }

    /// Append many children.
    #[must_use]
    pub fn children(mut self, cs: impl IntoIterator<Item = impl Into<Node>>) -> Self {
        for c in cs {
            self.0.children.push(c.into());
        }
        self
    }

    /// Attach a binding.
    #[must_use]
    pub fn bind(mut self, b: Binding) -> Self {
        self.0.bindings.push(b);
        self
    }

    /// Attach an interactive action slot.
    #[must_use]
    pub fn action(mut self, a: ActionSlot) -> Self {
        self.0.actions.push(a);
        self
    }

    /// Set an attribute.
    #[must_use]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.0.attrs.insert(key.into(), value.into());
        self
    }

    /// Set priority.
    #[must_use]
    pub fn priority(mut self, p: Priority) -> Self {
        self.0.lifecycle.priority = p;
        self
    }

    /// Convert to the underlying `Node`.
    #[must_use]
    pub fn build(self) -> Node {
        self.0
    }

    /// Wrap this node in a `Scene`.
    #[must_use]
    pub fn into_scene(self) -> Scene {
        Scene::new(self.0)
    }
}

impl From<NodeBuilder> for Node {
    fn from(b: NodeBuilder) -> Self {
        b.0
    }
}

/// Extension trait so `Node` values can also call `.into_scene()`.
pub trait NodeExt {
    /// Wrap this node in a `Scene`.
    fn into_scene(self) -> Scene;
}

impl NodeExt for Node {
    fn into_scene(self) -> Scene {
        Scene::new(self)
    }
}

// ─────────────────── Intent shortcuts ───────────────────

/// `Intent::Prose`.
#[must_use]
pub fn prose(text: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Prose { text: text.into() }))
}

/// `Intent::Code`.
#[must_use]
pub fn code(lang: impl Into<String>, source: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Code {
        lang: lang.into(),
        source: source.into(),
    }))
}

/// `Intent::Section`.
#[must_use]
pub fn section(title: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Section {
        title: Some(title.into()),
        collapsible: false,
    }))
}

/// `Intent::Divider`.
#[must_use]
pub fn divider() -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Divider))
}

/// `Intent::Group` with list layout.
#[must_use]
pub fn list() -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Group {
        layout: GroupKind::List,
    }))
}

/// `Intent::Group` with grid layout.
#[must_use]
pub fn grid() -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Group {
        layout: GroupKind::Grid,
    }))
}

/// `Intent::Progress`.
#[must_use]
pub fn progress(pct: f32) -> ProgressBuilder {
    ProgressBuilder {
        pct: Some(pct.clamp(0.0, 1.0)),
        label: None,
    }
}

/// `Intent::EntityRef`.
#[must_use]
pub fn entity(kind: impl Into<String>, id: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::EntityRef {
        kind: kind.into(),
        id: id.into(),
        label: None,
    }))
}

/// `Intent::Link`.
#[must_use]
pub fn link(href: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Link {
        href: href.into(),
        label: None,
    }))
}

/// `Intent::Signal` — live-bound scalar display.
#[must_use]
pub fn signal(topic: impl Into<Topic>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Signal {
        topic: topic.into(),
        display: prosopon_core::SignalDisplay::Inline,
    }))
}

/// `Intent::ToolCall`.
#[must_use]
pub fn tool_call(name: impl Into<String>, args: Value) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::ToolCall {
        name: name.into(),
        args,
        stream: None,
    }))
}

/// `Intent::ToolResult`.
#[must_use]
pub fn tool_result(success: bool, payload: Value) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::ToolResult { success, payload }))
}

/// `Intent::FileRead` — construct a pending filesystem read for `path`.
///
/// Call `.bytes(n)` / `.mime(s)` / `.content(s)` before pushing the node to
/// the scene; or emit as-is and patch `content` in via `NodeUpdated` when the
/// read resolves.
#[must_use]
pub fn file_read(path: impl Into<String>) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::FileRead {
        path: path.into(),
        content: None,
        bytes: None,
        mime: None,
    }))
}

/// `Intent::FileWrite` — construct a pending filesystem write of the given `kind`.
///
/// Typical shape: `ir::file_write("notes/a.md", FileWriteKind::Create)` for the
/// pending node, then `NodeUpdated` with a patched `content` once the write
/// resolves. Agents MAY provide `content` immediately for atomic writes.
#[must_use]
pub fn file_write(path: impl Into<String>, op: FileWriteKind) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::FileWrite {
        path: path.into(),
        op,
        content: None,
        bytes: None,
        title: None,
        mime: None,
    }))
}

/// `Intent::Choice`.
#[must_use]
pub fn choice(prompt: impl Into<String>) -> ChoiceBuilder {
    ChoiceBuilder {
        prompt: prompt.into(),
        options: Vec::new(),
    }
}

/// `Intent::Confirm`.
#[must_use]
pub fn confirm(message: impl Into<String>, severity: Severity) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Confirm {
        message: message.into(),
        severity,
    }))
}

/// `Intent::Stream` — token/audio/binary streaming channel.
#[must_use]
pub fn stream(id: impl Into<StreamId>, kind: prosopon_core::StreamKind) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Stream {
        id: id.into(),
        kind,
    }))
}

/// `Intent::Field` — scalar/vector field visualization hint.
#[must_use]
pub fn field(topic: impl Into<Topic>, projection: prosopon_core::Projection) -> NodeBuilder {
    NodeBuilder(Node::new(Intent::Field {
        topic: topic.into(),
        projection,
    }))
}

// ─────────────────── Builder for Progress with optional label ───────────────────

/// Chainable builder for the progress intent.
pub struct ProgressBuilder {
    pct: Option<f32>,
    label: Option<String>,
}

impl ProgressBuilder {
    /// Attach a label.
    #[must_use]
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Finalize into a `NodeBuilder` for further chaining.
    #[must_use]
    pub fn node(self) -> NodeBuilder {
        NodeBuilder(Node::new(Intent::Progress {
            pct: self.pct,
            label: self.label,
        }))
    }
}

impl From<ProgressBuilder> for NodeBuilder {
    fn from(b: ProgressBuilder) -> Self {
        b.node()
    }
}

impl From<ProgressBuilder> for Node {
    fn from(b: ProgressBuilder) -> Self {
        b.node().build()
    }
}

// ─────────────────── Choice builder ───────────────────

/// Chainable builder for the choice intent.
pub struct ChoiceBuilder {
    prompt: String,
    options: Vec<ChoiceOption>,
}

impl ChoiceBuilder {
    /// Append a choice option.
    #[must_use]
    pub fn option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(ChoiceOption {
            id: id.into(),
            label: label.into(),
            description: None,
            default: false,
        });
        self
    }

    /// Set the most-recently-added option as the default.
    #[must_use]
    pub fn default(mut self) -> Self {
        if let Some(last) = self.options.last_mut() {
            last.default = true;
        }
        self
    }

    /// Finalize into a `NodeBuilder`.
    #[must_use]
    pub fn node(self) -> NodeBuilder {
        NodeBuilder(Node::new(Intent::Choice {
            prompt: self.prompt,
            options: self.options,
        }))
    }
}

impl From<ChoiceBuilder> for Node {
    fn from(b: ChoiceBuilder) -> Self {
        b.node().build()
    }
}

impl From<ChoiceBuilder> for NodeBuilder {
    fn from(b: ChoiceBuilder) -> Self {
        b.node()
    }
}

// ─────────────────── Binding and action helpers ───────────────────

/// Construct a binding from a topic to a named attribute.
#[must_use]
pub fn bind_attr(topic: impl Into<Topic>, key: impl Into<String>) -> Binding {
    Binding {
        source: SignalRef::topic(topic),
        target: prosopon_core::BindTarget::Attr { key: key.into() },
        transform: None,
    }
}

/// Construct a simple `Invoke` action slot.
#[must_use]
pub fn action_invoke(command: impl Into<String>, label: impl Into<String>) -> ActionSlot {
    ActionSlot::new(ActionKind::Invoke {
        command: command.into(),
        args: Value::Null,
    })
    .with_label(label)
}

/// Construct a `Feedback` action slot.
#[must_use]
pub fn action_feedback(valence: prosopon_core::Valence) -> ActionSlot {
    ActionSlot::new(ActionKind::Feedback {
        valence,
        comment: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quickstart_example_builds() {
        let scene = section("Analysis")
            .child(prose("Inspected 3 entities."))
            .child(progress(0.66).label("Scoring"))
            .into_scene();
        assert_eq!(scene.root.children.len(), 2);
    }

    #[test]
    fn choice_builder_marks_default() {
        let n = choice("Accept?")
            .option("y", "Yes")
            .default()
            .option("n", "No")
            .node()
            .build();
        if let Intent::Choice { options, .. } = n.intent {
            assert!(options[0].default);
            assert!(!options[1].default);
        } else {
            panic!("expected choice intent");
        }
    }
}
