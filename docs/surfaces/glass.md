# Surface: Glass (2D web)

**Status:** planned for v0.2.0

## What it will be

The web compositor — renders the Prosopon IR into Arcan Glass-styled React /
Web Components. Target: shadcn-compatible, Tailwind-tokened, runtime
(compositor + runtime) written in TypeScript that consumes the JSON Schema
published by `prosopon-core`.

## Why "glass"

Arcan Glass is Broomva's established design language. This compositor adopts
it wholesale so Prosopon output looks native in broomva.tech, Mission Control,
Prompter, and Relay without custom theming per surface.

## Design choices (tentative)

- **TS + Preact signals** — mirror the Rust runtime's signal semantics
  (`@preact/signals-core` already used in IKR).
- **Widget registry** — A2UI-inspired. Each Intent variant maps to a React
  component. Compositors hold the registry; agents never ship components.
- **Layout via Yoga** — reuse the IKR layout engine rather than reinventing
  flexbox. (Pretext handles text measurement separately.)
- **Streaming** — Server-Sent Events for `StreamChunk` payloads; WebSocket for
  bidirectional sessions.

## Module sketch

```
crates/prosopon-compositor-glass/        # Rust wrapper (serves assets + proxies events)
  ├── src/
  │   └── lib.rs                         # axum router, event → SSE fanout
  └── web/                               # TS compositor
      ├── package.json
      ├── src/
      │   ├── runtime/                   # scene store, signal bus (mirror of Rust)
      │   ├── registry/                  # Intent → React component map
      │   ├── components/                # per-intent components
      │   └── index.ts
      └── tests/
```

## Intent → component sketch

| Intent | Component |
|---|---|
| `Prose` | `<Prose>` (typography stack) |
| `Code` | `<Code>` (syntax-highlighted, Shiki) |
| `Section` | `<Section>` (h2 + divider) |
| `ToolCall` | `<ToolCall>` (expandable) |
| `ToolResult` | `<ToolResult>` |
| `Progress` | `<Progress>` (radix-ui primitive) |
| `Choice` | `<Choice>` (radio group) |
| `Confirm` | `<ConfirmDialog>` |
| `Stream` | `<TokenStream>` (Pretext-measured for 120fps) |
| `Field` | `<Heatmap>` / `<Contour>` (react-three-fiber for Volume) |
| `Image/Audio/Video` | `<Image>`, `<AudioPlayer>`, `<Video>` |

## Open questions

- **SSR vs. client-only.** SSR gives instant-visible first paint; client-only
  gives simpler signal wiring. A/B during v0.2 prototyping.
- **Action round-trip.** POST per action vs. WebSocket command — start with
  POST for simplicity.
- **Design tokens.** Pull from arcan-glass package or define inline? Prefer
  external for cross-surface consistency.

## Demo target

A standalone Next.js page at `broomva.tech/prosopon/demo` that:
1. Connects via SSE to a `prosopon-daemon` instance.
2. Renders the same `cargo run -p prosopon-cli -- demo` scene through
   `<TextCompositor>` and `<GlassCompositor>` side by side.
3. Shows the JSON envelopes in a third panel.

One IR → three surfaces → visually obviously equivalent.
