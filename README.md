# Prosopon — the face of the agent

> *Prosopon (πρόσωπον): the mask worn by an actor; the visible person; what is placed before the eyes.*

**Prosopon is a display server for AI agents.** Agents emit *semantic intent* — what they're doing, what they need, what they've decided, what they've observed. Compositors render that intent to whichever surface the human happens to be watching: text, 2D, 3D, shader, audio, spatial, tactile. One IR, many faces.

This is not "a GUI library." It is the protocol + runtime + reference compositor layer that sits one level *below* any specific GUI — the Wayland of agent UIs. Mission Control, Prompter, Relay, your own product, a third party — all can be compositors on top of Prosopon.

---

## Repo at a glance

| Crate | Role |
|---|---|
| [`prosopon-core`](crates/prosopon-core/) | The semantic IR — `Node`, `Intent`, `Scene`, `ProsoponEvent`. JSON-native, schema-published. |
| [`prosopon-protocol`](crates/prosopon-protocol/) | Wire envelope, version negotiation, JSON + JSONL codecs. |
| [`prosopon-runtime`](crates/prosopon-runtime/) | Reactive signal bus, scene store, compositor trait + registry. |
| [`prosopon-compositor-text`](crates/prosopon-compositor-text/) | Reference ANSI terminal compositor — the "universal fallback." |
| [`prosopon-sdk`](crates/prosopon-sdk/) | Ergonomic builder API for agents (`ir::prose()`, `ir::section()`, …). |
| [`prosopon-pneuma`](crates/prosopon-pneuma/) | `Pneuma<B = L0ToExternal>` — the Life Agent OS boundary binding. |
| [`prosopon-cli`](crates/prosopon-cli/) | `prosopon inspect` / `validate` / `demo` / `schema` / `info`. |

## Quickstart

```bash
# Run the in-repo demo
cargo run -p prosopon-cli -- demo

# Or the minimal hello example
cargo run -p hello-prosopon

# Publish the IR schema for out-of-repo tooling
cargo run -p prosopon-cli -- schema scene > scene.schema.json
cargo run -p prosopon-cli -- schema event > event.schema.json
```

Programmatic use:

```rust
use prosopon_sdk::ir;
use prosopon_sdk::Session;

let scene = ir::section("Analysis")
    .child(ir::prose("Inspected 3 entities."))
    .child(ir::progress(0.66).label("Scoring"))
    .into_scene();

let mut session = Session::new();
for env in session.scene_reset_stream(scene) {
    println!("{}", serde_json::to_string(&env).unwrap());
}
```

## Documentation

- [**Positioning**](docs/positioning.md) — where prosopon sits in the a16z "Windows moment for agents" thesis and the emerging MCP-Apps / A2UI / AG-UI landscape.
- [**Architecture**](ARCHITECTURE.md) — module boundaries, dependency chain, and the Flutter-inspired 3-tree mental model.
- [**RFC-0001 — IR schema**](docs/rfcs/0001-ir-schema.md) — Node / Intent / Scene / ProsoponEvent, design axioms.
- [**RFC-0002 — Compositor contract**](docs/rfcs/0002-compositor-contract.md) — the trait, capabilities, error contract, lifecycle.
- [**RFC-0003 — Pneuma binding**](docs/rfcs/0003-pneuma-binding.md) — how prosopon becomes `Pneuma<L0ToExternal>` in the Life Agent OS.
- [**Surface notes**](docs/surfaces/) — per-surface design notes (text, glass, field, spatial, audio).

## Design axioms

1. **Intent-heavy, appearance-light.** No color or typography in the IR. Compositors decide how the shape is rendered on their surface.
2. **Wayland-minimal core.** A tiny versioned protocol; features arrive as extensions, never as breaking changes.
3. **Flutter-separated layers.** IR (Widget) → runtime (Element) → compositor (RenderObject). Agents rebuild intent cheaply; surfaces reuse layout expensively.
4. **JSON-native, language-agnostic.** The IR is serde-first; compositors in other languages consume the published JSON Schema.
5. **Pneuma-integrated.** Prosopon implements the outward boundary of the Life Agent OS — it is not a standalone product but a substrate.

## Status

**v0.1.0 — research-quality proof.** The IR, protocol, runtime, and text compositor compile, test, and demo end-to-end. Glass (2D web), Field (shader), Spatial, and Audio compositors are designed but not yet implemented — see the [surface notes](docs/surfaces/) and [PLANS.md](PLANS.md).

## Tracking

- **Linear project:** [Prosopon — Display Server for Agents](https://linear.app/broomva/project/prosopon-display-server-for-agents-b75c23a05005)
- **Milestones:** v0.1.0 (shipped 2026-04-21), v0.2.0 (2026-06-15), v0.3.0 (2026-07-31), v0.4.0 (2026-09-30)
- **Bootstrap ticket:** [BRO-759](https://linear.app/broomva/issue/BRO-759) · Done

## License

Apache-2.0 — see [LICENSE](LICENSE).
