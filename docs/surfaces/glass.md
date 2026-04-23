# Surface: Glass (2D web)

**Status:** shipped in v0.2.0-alpha (BRO-767)

## What it is

The web compositor — renders the Prosopon IR into Arcan Glass-styled Preact
components. Runtime (compositor + scene store + signal bus) written in
TypeScript that consumes the JSON Schema published by `prosopon-core`. Served
by a Rust axum daemon that fans out `ProsoponEvent`s to connected browsers
over WebSocket and Server-Sent Events; the web bundle itself is embedded into
the binary via `include_dir!`.

## Why "glass"

Arcan Glass is Broomva's established design language. This compositor adopts
it wholesale so Prosopon output looks native in broomva.tech, Mission Control,
Prompter, and Relay without custom theming per surface.

## Design choices (as shipped)

- **TS + Preact signals** — mirrors the Rust runtime's signal semantics via
  `@preact/signals-core`; `SceneStore` and `SignalBus` are hand-rolled TS
  mirrors of the Rust types.
- **Widget registry** — A2UI-inspired. Each `Intent.type` maps to a Preact
  component. The registry holds the dispatch table; agents never ship
  components. Unknown variants fall through to a built-in `Fallback`.
- **Totality** — every current `Intent` variant has a component (21 intents +
  `NodeView` wrapper). `Field` / `Locus` / `Formation` route to `Fallback`
  until the field compositor (BRO-774) lands.
- **Streaming** — WebSocket for bidirectional sessions; Server-Sent Events
  available at `/events` for receive-only clients (fixture replay, devtools).
- **Design tokens** — inlined Arcan Glass CSS variables at
  `web/src/tokens/glass.css`. External package publish (`@arcan/glass-tokens`)
  deferred — see open questions.

## Module layout (as shipped)

```
crates/prosopon-compositor-glass/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── compositor.rs              — GlassCompositor + EnvelopeFanout hook
│   ├── fanout.rs                  — tokio broadcast fanout
│   ├── server.rs                  — axum Router: / /assets/{*path} /ws /events /schema/...
│   ├── assets.rs                  — include_dir! embedded web bundle
│   └── bin/prosopon-glass.rs      — CLI
├── tests/
│   ├── apply_totality.rs
│   ├── server_handshake.rs
│   └── fixtures/demo_scene.jsonl
└── web/                           — bun workspace, Preact + @preact/signals, Arcan Glass tokens
    ├── package.json               — @prosopon/compositor-glass@0.2.0
    ├── src/
    │   ├── app.tsx
    │   ├── runtime/               — scene-store, signal-bus, transport, types
    │   ├── registry/              — context, intents dispatcher, fallback
    │   ├── components/            — 22 intent components (Node wrapper + 21 intents)
    │   ├── util/                  — binding, format
    │   ├── tokens/                — inlined Arcan Glass CSS variables
    │   └── actions/               — emit helper
    └── tests/                     — apply, binding, totality, goldens (vitest snapshots)
```

## Intent → component map (shipped)

| Intent | Component |
|---|---|
| `Prose` | `<Prose>` |
| `Code` | `<Code>` (plain `<pre>` — Shiki highlighting deferred) |
| `Math` | `<Math>` |
| `EntityRef` | `<EntityRef>` |
| `Link` | `<Link>` |
| `Citation` | `<Citation>` |
| `Signal` | `<Signal>` |
| `Stream` | `<Stream>` |
| `Choice` | `<Choice>` |
| `Confirm` | `<Confirm>` |
| `Input` | `<Input>` |
| `ToolCall` | `<ToolCall>` |
| `ToolResult` | `<ToolResult>` |
| `Progress` | `<Progress>` |
| `Group` | `<Group>` |
| `Section` | `<Section>` |
| `Divider` | `<Divider>` |
| `Image` / `Audio` / `Video` | `<Image>`, `<Audio>`, `<Video>` |
| `Empty` | `<Empty>` |
| `Custom` | `<Custom>` |
| `Field` / `Locus` / `Formation` | `<Fallback>` (pending field compositor — BRO-774) |

## Open questions (still open)

- **SSR vs. client-only.** Shipped as client-only; SSR path not yet prototyped.
  Revisit once public broomva.tech route (BRO-862) lands.
- **Action round-trip shape.** `ActionEmitted` envelopes are emitted client →
  server today but not yet threaded back into the runtime — see BRO-778.
  Multi-user shape (CRDT? per-session queues?) still undecided.
- **Design token publishing.** Tokens live inline at `web/src/tokens/glass.css`.
  Cross-surface consistency argues for lifting to a shared
  `@arcan/glass-tokens` package once consumers exist beyond this crate.
- **Pretext-measured streaming tail.** Deferred (BRO-760 follow-up).
- **Shiki syntax highlighting for `Code`.** Not planned for v0.2; treat plain
  `<pre>` as the minimum-viable contract.

## Demo target

```
cargo run -p prosopon-compositor-glass --bin prosopon-glass -- \
    serve --addr 127.0.0.1:4321 \
          --fixture crates/prosopon-compositor-glass/tests/fixtures/demo_scene.jsonl
```

Opens the embedded web bundle at `http://127.0.0.1:4321/` and replays the
fixture scene through WebSocket. A public route at `broomva.tech/prosopon/demo`
is tracked under BRO-862 — blocked on an npm publish of
`@prosopon/compositor-glass` so the site can consume it as a package.
