# PLANS

## v0.1.0 — shipped in first commit

- [x] Rust workspace, seven crates, one example, tests green.
- [x] `prosopon-core` with Intent / Node / Scene / ProsoponEvent, serde-first, JsonSchema-published.
- [x] `prosopon-protocol` with versioned `Envelope`, Json + Jsonl codecs, `Hello` handshake.
- [x] `prosopon-runtime` with SignalBus, SceneStore, Compositor trait, Runtime glue.
- [x] `prosopon-compositor-text` — reference ANSI compositor with binding hydration.
- [x] `prosopon-sdk` — `ir::{prose, section, choice, progress, tool_call, …}` builders.
- [x] `prosopon-pneuma` — local Pneuma trait + `L0ToExternal` / `ExternalToL0` markers.
- [x] `prosopon-cli` — `demo`, `info`, `inspect`, `validate`, `schema`.
- [x] `hello-prosopon` example — live signal binding end-to-end.
- [x] RFC-0001 (IR schema), RFC-0002 (compositor contract), RFC-0003 (Pneuma binding).
- [x] Positioning memo against a16z thesis.
- [x] Per-surface design notes for glass, field, spatial, audio.
- [x] Control metalayer setpoints.

## v0.2.0 — compat bridges + glass compositor

**Goal:** render unchanged through the compositor pipeline every common agent-UI protocol.

- [ ] `prosopon-compat-mcp-apps` — translate MCP Apps `_meta.ui.resourceUri` into Intent tree.
- [ ] `prosopon-compat-a2ui` — accept A2UI `createSurface` / `updateComponents` / `updateDataModel` / `deleteSurface` messages.
- [ ] `prosopon-compat-ag-ui` — AG-UI event → ProsoponEvent translation.
- [ ] `prosopon-compat-ai-sdk` — Vercel AI SDK 5.0 `message.parts[]` importer.
- [ ] `prosopon-compositor-glass` — 2D web compositor (TypeScript, consumes published JSON schema).
- [ ] Golden-file test harness for compositors.
- [ ] `prosopon-daemon` — long-running HTTP + WebSocket server (axum).

## v0.3.0 — Life integration + field compositor

- [ ] Upstream Pneuma trait lands in `aios-protocol`; swap local mirror.
- [ ] `prosopon-lago` — subscribe to Lago event journal, translate to Prosopon events.
- [ ] `prosopon-vigil` — OpenTelemetry spans → Prosopon observability intents.
- [ ] `prosopon-compositor-field` — GPU shader compositor (wgpu), Plexus field physics rendering.
- [ ] Arcan session view emits through Prosopon envelopes natively.

## v0.4.0 — spatial + audio surfaces

- [ ] `prosopon-compositor-spatial` — Vision Pro / Quest; rooms = projects.
- [ ] `prosopon-compositor-audio` — Gemini TTS + ambient sonification; per-agent timbre.
- [ ] Cross-surface coherence tests — same IR across text / glass / field / spatial / audio.

## v0.5.0 — editor-grade experience

- [ ] `prosopon-lsp` — Language Server Protocol support for IR authoring.
- [ ] `prosopon-editor` — visual scene editor (Tauri).
- [ ] Snapshotting + time-travel replay against Lago.

## Research tracks (continuous)

- **IR diffability.** Explore CRDT-lite wire formats (beyond pure event streams) for multi-user coherence.
- **Compositor capability negotiation.** When two compositors share a surface, how do they compose their outputs?
- **Intent inference.** Given raw agent output (tokens, tool calls), automatically derive the most semantically specific Intent.
- **Generative UI integration.** How does PAGEN-style LLM-emitted UI compose with declarative IR?
