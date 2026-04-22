# Surface: Text

**Status:** reference implementation shipped in v0.1.0 (`prosopon-compositor-text`)

## What it is

An ANSI-capable terminal compositor. The universal fallback — any surface that
can print bytes to a TTY can render Prosopon scenes at some fidelity. Also
doubles as a string target for log sinks, snapshot tests, and CI output.

## Design choices

- **Pure-function core.** `render_scene(scene, width, opts) -> String` is
  deterministic and side-effect free. The `TextCompositor` is a thin wrapper
  that writes the rendered string to a target (stdout, stderr, in-memory, file).
- **Binding hydration.** Before rendering, known `BindTarget::Attr` bindings
  (e.g. `pct` on `Intent::Progress`) are resolved against the scene's signal
  cache. Other bindings are surfaced as trace lines (`↻ attr:key ← value`).
- **ANSI opt-in.** `RenderOptions::plain()` disables escapes for captures.

## Intent → text mapping

| Intent | Rendering |
|---|---|
| `Prose` | Word-wrapped paragraphs at `width - indent`. |
| `Code` | Fenced block; syntax highlighting deferred. |
| `Math` | Source text prefixed with `math`. |
| `EntityRef` | `→ <kind>:<id>` or label if provided, colored cyan. |
| `Link` | `[label](href)` with label in blue. |
| `Signal` | `~ <topic> = <preview>` with value in yellow. |
| `Stream` | `⟳ stream:<id>` placeholder; chunks append inline as they arrive. |
| `Choice` | Bold prompt + bullets (`●` default, `○` non-default). |
| `Confirm` | `? <message>` colored by severity. |
| `Input` | `<prompt>[<default>]: ` |
| `ToolCall` | `⚙ <name>(<args preview>)` |
| `ToolResult` | `✓` or `✗` + payload preview, colored by success. |
| `Progress` | Unicode block bar + percentage + optional label. |
| `Section` | Bold title + underline rule. |
| `Divider` | Horizontal rule at `width`. |
| `Field` | `▦ field:<topic> <summary>` — degraded from true field viz. |
| `Locus` | `@ <frame> (x, y, z)` — coordinates as text. |
| `Formation` | `<kind> on <topic>` as a stub. |
| `Image` / `Audio` / `Video` | Placeholder with URI + alt. |
| `Empty` | (renders nothing). |
| `Custom` | `[<kind>] <payload preview>` fallback. |
| Unknown | `[unknown intent — compositor upgrade suggested]` |

## Known limitations

- **No advanced text layout.** No Pretext-style measurement or line-breaking
  sophistication yet. Wrapping is word-boundary only; CJK / RTL / emoji width
  deferred. (The `unicode-width` and `unicode-segmentation` dependencies are in
  place; their use is incremental.)
- **Full re-render on signal change.** Efficient enough for small scenes; for
  dense dashboards, v0.2 should introduce partial re-render via ANSI cursor
  positioning.
- **No mouse / key input.** Output only in v0.1. Actions are rendered as
  labels; binding them to keybinds is a future concern.

## v0.2 roadmap

- Pretext-style text measurement (`@chenglou/pretext` port to Rust) to make
  layout a pure function of (intent × width).
- Partial re-render using `crossterm` cursor movement.
- Golden-file test fixtures under `tests/`.
- Cursor-driven action handling when running attached to a TTY.
