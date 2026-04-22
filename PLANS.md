# PLANS

**Linear project:** [Prosopon — Display Server for Agents](https://linear.app/broomva/project/prosopon-display-server-for-agents-b75c23a05005)

## v0.1.0 — shipped in first commit (BRO-759 · Done · 2026-04-21)

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

**v0.1 follow-ups** — Backlog items emerged during bootstrap:
- [ ] [BRO-760](https://linear.app/broomva/issue/BRO-760) — Expand binding resolution beyond `Progress.pct`.
- [ ] [BRO-761](https://linear.app/broomva/issue/BRO-761) — Re-enable `#![warn(missing_docs)]` with full public-surface coverage.
- [ ] [BRO-762](https://linear.app/broomva/issue/BRO-762) — Golden-file snapshot tests for text compositor.

## v0.2.0 — compat bridges + glass compositor (target 2026-06-15)

**Goal:** render unchanged through the compositor pipeline every common agent-UI protocol.

- [ ] [BRO-763](https://linear.app/broomva/issue/BRO-763) — `prosopon-compat-mcp-apps` — translate MCP Apps `_meta.ui.resourceUri` into Intent tree.
- [ ] [BRO-764](https://linear.app/broomva/issue/BRO-764) — `prosopon-compat-a2ui` — accept A2UI `createSurface` / `updateComponents` / `updateDataModel` / `deleteSurface` messages.
- [ ] [BRO-765](https://linear.app/broomva/issue/BRO-765) — `prosopon-compat-ag-ui` — AG-UI event → ProsoponEvent translation.
- [ ] [BRO-766](https://linear.app/broomva/issue/BRO-766) — `prosopon-compat-ai-sdk` — Vercel AI SDK 5.0 `message.parts[]` importer.
- [ ] [BRO-767](https://linear.app/broomva/issue/BRO-767) — `prosopon-compositor-glass` — 2D web compositor (TypeScript, consumes published JSON schema).
- [ ] [BRO-768](https://linear.app/broomva/issue/BRO-768) — `prosopon-daemon` — long-running HTTP + WebSocket server (axum).
- [ ] [BRO-769](https://linear.app/broomva/issue/BRO-769) — Re-enable `clippy -D warnings` gate in CI.

## v0.3.0 — Life integration + field compositor (target 2026-07-31)

- [ ] [BRO-770](https://linear.app/broomva/issue/BRO-770) — Swap local Pneuma mirror for upstream `aios_protocol::pneuma` (blocked on upstream).
- [ ] [BRO-771](https://linear.app/broomva/issue/BRO-771) — `prosopon-lago` — subscribe to Lago event journal, translate to Prosopon events.
- [ ] [BRO-772](https://linear.app/broomva/issue/BRO-772) — `prosopon-vigil` — OpenTelemetry spans → Prosopon observability intents.
- [ ] [BRO-773](https://linear.app/broomva/issue/BRO-773) — Arcan session view emits through Prosopon envelopes natively.
- [ ] [BRO-774](https://linear.app/broomva/issue/BRO-774) — `prosopon-compositor-field` — GPU shader compositor (wgpu), Plexus field physics rendering.

## v0.4.0 — spatial + audio surfaces (target 2026-09-30)

- [ ] [BRO-775](https://linear.app/broomva/issue/BRO-775) — `prosopon-compositor-spatial` — Vision Pro / Quest; rooms = projects.
- [ ] [BRO-776](https://linear.app/broomva/issue/BRO-776) — `prosopon-compositor-audio` — Gemini TTS + ambient sonification; per-agent timbre.
- [ ] [BRO-777](https://linear.app/broomva/issue/BRO-777) — Cross-surface coherence tests — same IR across text / glass / field / spatial / audio.

## v0.5.0 — editor-grade experience

- [ ] `prosopon-lsp` — Language Server Protocol support for IR authoring.
- [ ] `prosopon-editor` — visual scene editor (Tauri).
- [ ] Snapshotting + time-travel replay against Lago.

## Research tracks (continuous)

- **IR diffability.** Explore CRDT-lite wire formats (beyond pure event streams) for multi-user coherence.
- **Compositor capability negotiation.** When two compositors share a surface, how do they compose their outputs?
- **Intent inference.** Given raw agent output (tokens, tool calls), automatically derive the most semantically specific Intent.
- **Generative UI integration.** How does PAGEN-style LLM-emitted UI compose with declarative IR?
