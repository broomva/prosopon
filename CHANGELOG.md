# Changelog

All notable changes to this project will be documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0-alpha] — 2026-04-23

### Added
- `prosopon-compositor-glass` crate — 2D web compositor, Arcan Glass-styled,
  axum HTTP + WebSocket + SSE server with embedded Preact bundle.
- `@prosopon/compositor-glass` bun-workspace TypeScript package — Preact +
  `@preact/signals-core` SceneStore/SignalBus mirroring the Rust runtime;
  total intent registry over every current `Intent` variant; `Fallback` for
  `field` / `locus` / `formation` pending the field compositor (BRO-774).
- `prosopon-glass` CLI — `serve --addr --fixture <jsonl>` for local dev +
  fixture replay.
- Cross-surface golden tests — shared JSON fixtures drive both vitest
  snapshots (glass web) and `insta` snapshots (Rust text). Any IR change
  lighting up in one surface lights up in the other.
- Shared fixture `tests/fixtures/demo_scene.jsonl`, authoritative for the
  concrete IR shape.
- Well-known attribute key `glass.variant` — RFC-0001.

### Fixed
- `prosopon-runtime::StoreEvent::Reset` now boxes its `Scene` payload —
  clearing the `clippy::large_enum_variant` regression on rustc 1.93
  and unblocking `make smoke`.
- `NodePatch::default()` test construction switched to struct literal
  (`field_reassign_with_default`).

### Tooling
- Workspace gains `axum 0.8`, `tower`, `tower-http`, `include_dir`,
  `mime_guess`, `bytes`, `async-stream` dependencies for the glass crate
  and future `prosopon-daemon` (BRO-768).

### Deferred
- Public demo route at broomva.tech/prosopon/demo — BRO-862 (blocked on
  npm publish of `@prosopon/compositor-glass`).
- Pretext-measured token-stream tail — BRO-760 follow-up.
- Shiki syntax highlighting for `Code` intent — not planned for v0.2.
- WS → runtime action round-trip — BRO-778.

## [0.1.0] — 2026-04-21

### Added
- Initial workspace with seven crates: `prosopon-core`, `prosopon-protocol`,
  `prosopon-runtime`, `prosopon-compositor-text`, `prosopon-sdk`, `prosopon-cli`,
  `prosopon-pneuma`.
- Semantic IR (`Node`, `Intent`, `Scene`, `ProsoponEvent`, `Binding`, `ActionSlot`)
  with serde + JsonSchema support.
- Wire protocol with versioned `Envelope`, `Hello` handshake, JSON + JSONL codecs.
- Reactive runtime: `SignalBus`, `SceneStore`, `Compositor` trait, `Runtime` glue.
- Reference ANSI terminal compositor with live signal binding hydration.
- SDK with `ir::*` builder functions and `Session` helper.
- CLI with `demo`, `info`, `inspect`, `validate`, `schema` subcommands.
- `hello-prosopon` example demonstrating end-to-end signal binding.
- Local `Pneuma` trait mirror + `L0ToExternal` / `ExternalToL0` boundary markers.
- RFC-0001 (IR schema), RFC-0002 (compositor contract), RFC-0003 (Pneuma binding).
- Positioning memo against a16z's "Windows moment for agents" thesis.
- Per-surface design notes for text, glass, field, spatial, audio.
- Control metalayer setpoints (CONTROL.md + .control/policy.yaml).

[0.1.0]: https://github.com/broomva/prosopon/releases/tag/v0.1.0
