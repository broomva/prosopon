//! The renderer — pure function from `(Scene × width × options) → String`.
//!
//! This is deliberately side-effect-free so the same logic runs in tests, stdout
//! rendering, and log sinks. Pretext's insight applied to semantic IR: layout is a
//! pure computation over measurable units.

use std::fmt::Write as _;

use prosopon_core::{
    ActionKind, ActionSlot, Binding, Intent, Node, NodeStatus, Priority, Scene, SignalValue, Topic,
    Value,
};

/// Tunable rendering knobs.
#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// Emit ANSI colors/styles. Disable for plain-text capture (tests, logs).
    pub ansi: bool,
    /// Indent per tree depth (spaces).
    pub indent: u16,
    /// Render bindings inline next to bound attributes. Off = render bindings list
    /// separately (debug view).
    pub inline_bindings: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            ansi: true,
            indent: 2,
            inline_bindings: true,
        }
    }
}

impl RenderOptions {
    /// Plain text — no ANSI escapes. Use for snapshot tests.
    #[must_use]
    pub fn plain() -> Self {
        Self {
            ansi: false,
            indent: 2,
            inline_bindings: true,
        }
    }
}

/// Render a whole scene to a string.
#[must_use]
pub fn render_scene(scene: &Scene, width: u16, opts: &RenderOptions) -> String {
    let mut out = String::new();
    render_node(&scene.root, 0, width, &scene.signals, opts, &mut out);
    out
}

fn render_node(
    node: &Node,
    depth: usize,
    width: u16,
    signals: &indexmap::IndexMap<Topic, SignalValue>,
    opts: &RenderOptions,
    out: &mut String,
) {
    let pad_width = depth * opts.indent as usize;
    let pad = " ".repeat(pad_width);

    let hydrated = hydrate_intent(&node.intent, &node.bindings, signals);
    render_intent(&hydrated, &pad, width, signals, node, opts, out);

    for b in &node.bindings {
        render_binding(b, &pad, signals, opts, out);
    }

    if !node.actions.is_empty() {
        render_actions(&node.actions, &pad, opts, out);
    }

    for c in &node.children {
        render_node(c, depth + 1, width, signals, opts, out);
    }
}

/// Resolve well-known bindings into the intent so renderers see live values.
///
/// This is intentionally conservative: only a small set of `BindTarget::Attr` keys
/// are mapped to intent slots (`pct` → `Progress.pct`). Anything else is left alone
/// and still rendered as a binding-trace line by `render_binding`.
fn hydrate_intent(
    intent: &Intent,
    bindings: &[Binding],
    signals: &indexmap::IndexMap<Topic, SignalValue>,
) -> Intent {
    let mut out = intent.clone();
    for b in bindings {
        let prosopon_core::BindTarget::Attr { key } = &b.target else {
            continue;
        };
        let Some(value) = signals.get(&b.source.topic) else {
            continue;
        };
        let SignalValue::Scalar(v) = value else {
            continue;
        };
        if let (Intent::Progress { pct, .. }, "pct") = (&mut out, key.as_str()) {
            if let Some(f) = v.as_f64() {
                *pct = Some(f as f32);
            }
        }
    }
    out
}

fn render_intent(
    intent: &Intent,
    pad: &str,
    width: u16,
    signals: &indexmap::IndexMap<Topic, SignalValue>,
    node: &Node,
    opts: &RenderOptions,
    out: &mut String,
) {
    let prefix = status_prefix(&node.lifecycle.status, node.lifecycle.priority, opts);
    match intent {
        Intent::Empty => {}
        Intent::Prose { text } => {
            wrap_and_write(out, pad, &prefix, text, width, opts);
        }
        Intent::Code { lang, source } => {
            writeln!(out, "{pad}{prefix}```{lang}").ok();
            for line in source.lines() {
                writeln!(out, "{pad}  {line}").ok();
            }
            writeln!(out, "{pad}```").ok();
        }
        Intent::Math { source } => {
            writeln!(out, "{pad}{prefix}{}", style("math", Color::Cyan, opts)).ok();
            writeln!(out, "{pad}  {source}").ok();
        }
        Intent::EntityRef { kind, id, label } => {
            let shown = label.clone().unwrap_or_else(|| format!("{kind}:{id}"));
            writeln!(
                out,
                "{pad}{prefix}{} {}",
                style("→", Color::Blue, opts),
                style(&shown, Color::Cyan, opts)
            )
            .ok();
        }
        Intent::Link { href, label } => {
            let shown = label.clone().unwrap_or_else(|| href.clone());
            writeln!(
                out,
                "{pad}{prefix}[{}]({})",
                style(&shown, Color::Blue, opts),
                href
            )
            .ok();
        }
        Intent::Citation { source, anchor } => {
            let a = anchor.as_deref().unwrap_or("");
            writeln!(
                out,
                "{pad}{prefix}cite: {source}{}",
                if a.is_empty() {
                    "".into()
                } else {
                    format!("#{a}")
                }
            )
            .ok();
        }
        Intent::Signal { topic, display: _ } => {
            let current = signals
                .get(topic)
                .map(SignalValue::preview)
                .unwrap_or_else(|| "<pending>".to_string());
            writeln!(
                out,
                "{pad}{prefix}{} {} = {}",
                style("~", Color::Magenta, opts),
                topic,
                style(&current, Color::Yellow, opts)
            )
            .ok();
        }
        Intent::Stream { id, kind: _ } => {
            writeln!(
                out,
                "{pad}{prefix}{} stream:{id}",
                style("⟳", Color::Magenta, opts)
            )
            .ok();
        }
        Intent::Choice { prompt, options } => {
            writeln!(out, "{pad}{prefix}{}", style(prompt, Color::Bold, opts)).ok();
            for o in options {
                let marker = if o.default { "● " } else { "○ " };
                writeln!(out, "{pad}  {marker}{} — {}", o.id, o.label).ok();
            }
        }
        Intent::Confirm { message, severity } => {
            let color = match severity {
                prosopon_core::Severity::Info => Color::Blue,
                prosopon_core::Severity::Notice => Color::Cyan,
                prosopon_core::Severity::Warning => Color::Yellow,
                prosopon_core::Severity::Danger => Color::Red,
            };
            writeln!(
                out,
                "{pad}{prefix}{} {}",
                style("?", color, opts),
                style(message, color, opts)
            )
            .ok();
        }
        Intent::Input {
            prompt,
            input: _,
            default,
        } => {
            let def = default
                .as_ref()
                .map(|v| format!(" [{}]", preview_value(v)))
                .unwrap_or_default();
            writeln!(out, "{pad}{prefix}{}{}: ", prompt, def).ok();
        }
        Intent::ToolCall {
            name,
            args,
            stream: _,
        } => {
            writeln!(
                out,
                "{pad}{prefix}{} {}({})",
                style("⚙", Color::Cyan, opts),
                style(name, Color::Bold, opts),
                preview_value(args)
            )
            .ok();
        }
        Intent::ToolResult { success, payload } => {
            let marker = if *success { "✓" } else { "✗" };
            let color = if *success { Color::Green } else { Color::Red };
            writeln!(
                out,
                "{pad}{prefix}{} {}",
                style(marker, color, opts),
                preview_value(payload)
            )
            .ok();
        }
        Intent::Progress { pct, label } => {
            let bar = progress_bar(pct.unwrap_or(0.0), 24, opts);
            let label = label.as_deref().unwrap_or("");
            writeln!(out, "{pad}{prefix}{bar} {label}").ok();
        }
        Intent::Group { layout: _ } => {
            // A group is a container; children will render themselves.
        }
        Intent::Section {
            title,
            collapsible: _,
        } => {
            if let Some(t) = title {
                writeln!(out, "{pad}{}", style(t, Color::Bold, opts)).ok();
                writeln!(
                    out,
                    "{pad}{}",
                    "─".repeat(t.chars().count().min(width.into()))
                )
                .ok();
            }
        }
        Intent::Divider => {
            writeln!(
                out,
                "{pad}{}",
                "─".repeat(width.saturating_sub(pad.len() as u16).into())
            )
            .ok();
        }
        Intent::Field {
            topic,
            projection: _,
        } => {
            let summary = signals
                .get(topic)
                .map(SignalValue::preview)
                .unwrap_or_else(|| "<no data>".into());
            writeln!(
                out,
                "{pad}{prefix}{} field:{} {}",
                style("▦", Color::Magenta, opts),
                topic,
                style(&summary, Color::Yellow, opts)
            )
            .ok();
        }
        Intent::Locus { frame, position } => {
            writeln!(
                out,
                "{pad}{prefix}@ {:?} ({:.2}, {:.2}, {:.2})",
                frame, position[0], position[1], position[2]
            )
            .ok();
        }
        Intent::Formation { topic, kind } => {
            writeln!(out, "{pad}{prefix}{:?} on {}", kind, topic).ok();
        }
        Intent::Image { uri, alt } => {
            writeln!(out, "{pad}{prefix}[image: {alt}] {uri}").ok();
        }
        Intent::Audio { uri, stream, voice } => {
            let src = uri
                .as_deref()
                .or(stream.as_ref().map(|s| s.as_str()))
                .unwrap_or("<live>");
            let v = voice.as_deref().unwrap_or("default");
            writeln!(
                out,
                "{pad}{prefix}{} audio:{src} voice:{v}",
                style("♪", Color::Magenta, opts)
            )
            .ok();
        }
        Intent::Video { uri, poster: _ } => {
            writeln!(out, "{pad}{prefix}[video] {uri}").ok();
        }
        Intent::Custom { kind, payload } => {
            writeln!(out, "{pad}{prefix}[{kind}] {}", preview_value(payload)).ok();
        }
        // Forwards-compatible: render an informative placeholder for unknown
        // `Intent` variants so agents emitting future variants still produce
        // something visible to the user.
        _ => {
            writeln!(
                out,
                "{pad}{prefix}[unknown intent — compositor upgrade suggested]"
            )
            .ok();
        }
    }
}

fn render_binding(
    b: &Binding,
    pad: &str,
    signals: &indexmap::IndexMap<Topic, SignalValue>,
    opts: &RenderOptions,
    out: &mut String,
) {
    if !opts.inline_bindings {
        return;
    }
    let current = signals
        .get(&b.source.topic)
        .map(SignalValue::preview)
        .unwrap_or_else(|| "<pending>".into());
    writeln!(
        out,
        "{pad}  {} {} ← {}",
        style("↻", Color::Magenta, opts),
        target_label(&b.target),
        style(&current, Color::Yellow, opts)
    )
    .ok();
}

fn target_label(t: &prosopon_core::BindTarget) -> String {
    match t {
        prosopon_core::BindTarget::Attr { key } => format!("attr:{key}"),
        prosopon_core::BindTarget::IntentSlot { path } => format!("intent:{path}"),
        prosopon_core::BindTarget::ChildContent { id } => format!("child:{id}"),
    }
}

fn render_actions(actions: &[ActionSlot], pad: &str, opts: &RenderOptions, out: &mut String) {
    let labels: Vec<String> = actions
        .iter()
        .filter(|a| !matches!(a.visibility, prosopon_core::Visibility::Hidden))
        .map(|a| {
            let label = a
                .label
                .clone()
                .unwrap_or_else(|| action_default_label(&a.kind));
            if a.enabled {
                style(&label, Color::Cyan, opts)
            } else {
                style(&label, Color::Dim, opts)
            }
        })
        .collect();
    if !labels.is_empty() {
        writeln!(out, "{pad}  [{}]", labels.join(" | ")).ok();
    }
}

fn action_default_label(k: &ActionKind) -> String {
    match k {
        ActionKind::Submit { .. } => "submit".into(),
        ActionKind::Inspect { .. } => "inspect".into(),
        ActionKind::Focus { .. } => "focus".into(),
        ActionKind::Invoke { command, .. } => command.clone(),
        ActionKind::Feedback { .. } => "feedback".into(),
        ActionKind::Choose { option_id } => format!("choose:{option_id}"),
        ActionKind::Input { .. } => "input".into(),
        ActionKind::Confirm { .. } => "confirm".into(),
    }
}

fn status_prefix(status: &NodeStatus, priority: Priority, opts: &RenderOptions) -> String {
    let base = match status {
        NodeStatus::Active => "",
        NodeStatus::Pending => "… ",
        NodeStatus::Resolved => "✓ ",
        NodeStatus::Failed { .. } => "✗ ",
        NodeStatus::Decaying { .. } => "· ",
    };
    let urgency = match priority {
        Priority::Blocking => "! ",
        Priority::Urgent => "* ",
        _ => "",
    };
    let raw = format!("{urgency}{base}");
    if raw.is_empty() {
        return String::new();
    }
    let color = match priority {
        Priority::Blocking => Color::Red,
        Priority::Urgent => Color::Yellow,
        _ => match status {
            NodeStatus::Failed { .. } => Color::Red,
            NodeStatus::Resolved => Color::Green,
            NodeStatus::Decaying { .. } => Color::Dim,
            _ => Color::Reset,
        },
    };
    style(&raw, color, opts)
}

fn wrap_and_write(
    out: &mut String,
    pad: &str,
    prefix: &str,
    text: &str,
    width: u16,
    _opts: &RenderOptions,
) {
    let effective = width.saturating_sub(pad.len() as u16).max(20) as usize;
    for paragraph in text.split('\n') {
        let mut line_len = 0usize;
        let mut first = true;
        for word in paragraph.split_whitespace() {
            let wlen = word.chars().count();
            if line_len == 0 {
                out.push_str(pad);
                if first {
                    out.push_str(prefix);
                    first = false;
                }
                out.push_str(word);
                line_len = pad.len() + prefix.len() + wlen;
            } else if line_len + 1 + wlen > effective {
                out.push('\n');
                out.push_str(pad);
                out.push_str(word);
                line_len = pad.len() + wlen;
            } else {
                out.push(' ');
                out.push_str(word);
                line_len += 1 + wlen;
            }
        }
        out.push('\n');
    }
}

fn progress_bar(pct: f32, width: usize, opts: &RenderOptions) -> String {
    let filled = ((pct.clamp(0.0, 1.0)) * width as f32) as usize;
    let empty = width - filled;
    let full = "█".repeat(filled);
    let gap = "░".repeat(empty);
    let label = format!("{:>3.0}%", (pct * 100.0).round());
    format!(
        "{}{} {}",
        style(&full, Color::Green, opts),
        style(&gap, Color::Dim, opts),
        label
    )
}

fn preview_value(v: &Value) -> String {
    let s = serde_json::to_string(v).unwrap_or_default();
    if s.len() > 80 {
        format!("{}…", &s[..79])
    } else {
        s
    }
}

#[derive(Copy, Clone)]
enum Color {
    Reset,
    Bold,
    Dim,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
}

fn style(s: &str, c: Color, opts: &RenderOptions) -> String {
    if !opts.ansi {
        return s.to_string();
    }
    let code = match c {
        Color::Reset => "0",
        Color::Bold => "1",
        Color::Dim => "2",
        Color::Red => "31",
        Color::Green => "32",
        Color::Yellow => "33",
        Color::Blue => "34",
        Color::Magenta => "35",
        Color::Cyan => "36",
    };
    format!("\x1b[{code}m{s}\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node, Scene};

    #[test]
    fn prose_renders_with_width() {
        let scene = Scene::new(Node::new(Intent::Prose {
            text: "hello world from the prosopon text compositor".into(),
        }));
        let out = render_scene(&scene, 20, &RenderOptions::plain());
        // Should wrap onto at least two lines.
        assert!(out.lines().count() >= 2);
        assert!(out.contains("hello"));
    }

    #[test]
    fn section_renders_title_and_divider() {
        let scene = Scene::new(
            Node::new(Intent::Section {
                title: Some("Status".into()),
                collapsible: false,
            })
            .child(Node::new(Intent::Prose { text: "ok".into() })),
        );
        let out = render_scene(&scene, 40, &RenderOptions::plain());
        assert!(out.contains("Status"));
        assert!(out.contains("─"));
        assert!(out.contains("ok"));
    }

    #[test]
    fn progress_renders_bar_and_pct() {
        let scene = Scene::new(Node::new(Intent::Progress {
            pct: Some(0.5),
            label: Some("scoring".into()),
        }));
        let out = render_scene(&scene, 60, &RenderOptions::plain());
        assert!(out.contains("50%"));
        assert!(out.contains("scoring"));
    }
}
