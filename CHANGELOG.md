# Changelog

All notable changes to this project will be documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
