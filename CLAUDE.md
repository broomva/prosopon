# CLAUDE.md — Prosopon

## What this repo is

A Rust-native **display server for AI agents**. The outward-facing `Pneuma<B=L0ToExternal>` boundary of the Life Agent OS. Sibling of Sensorium (`Pneuma<ExternalToL0>`).

Agents emit **semantic intent**; compositors render it to whichever surface the human is watching (text / 2D / 3D / shader / audio / spatial / tactile). Not a GUI — the protocol + runtime + reference compositors underneath every GUI.

## On session start

1. `cat README.md` — mental model.
2. `cat ARCHITECTURE.md` — module boundaries.
3. `ls docs/rfcs/` — RFCs shaping the IR + contracts.
4. `make smoke` — verify the tree is green before editing.
5. Scan `docs/conversations/` (when the bridge lands) for prior-session context.

## Common commands

```bash
make smoke          # cargo check + clippy + test
make test           # cargo test --workspace
make check          # cargo check + clippy only
make fmt            # cargo fmt
make control-audit  # smoke + fmt check (pre-push gate)

cargo run -p prosopon-cli -- demo        # in-repo demo render
cargo run -p prosopon-cli -- info        # versions
cargo run -p prosopon-cli -- schema scene  # publish IR schema as JSON
cargo run -p hello-prosopon              # minimal end-to-end example
```

## Conventions (inherit from Life)

- Rust 2024, MSRV 1.85, resolver = "2".
- `thiserror` for library errors, `anyhow` for apps.
- `tracing` for logs — never `println!`.
- `tokio` for async; `serde` for all serialization.
- Tests co-located (`#[cfg(test)] mod tests` at the bottom of each file).

## Key files for orientation

| Concern | File |
|---|---|
| IR shape | `crates/prosopon-core/src/intent.rs` |
| Scene root | `crates/prosopon-core/src/scene.rs` |
| Event stream | `crates/prosopon-core/src/event.rs` |
| Wire envelope | `crates/prosopon-protocol/src/lib.rs` |
| Runtime glue | `crates/prosopon-runtime/src/lib.rs` |
| Compositor trait | `crates/prosopon-runtime/src/compositor.rs` |
| Reference text compositor | `crates/prosopon-compositor-text/src/render.rs` |
| Pneuma binding | `crates/prosopon-pneuma/src/lib.rs` |

## When adding a new Intent variant

1. Add it to `crates/prosopon-core/src/intent.rs`. Document the variant.
2. Add a serde roundtrip test in the same file.
3. Handle it in `crates/prosopon-compositor-text/src/render.rs::render_intent` — the reference compositor must render *every* variant, even if just as a placeholder.
4. Update `docs/rfcs/0001-ir-schema.md` with the new variant and its semantics.
5. Add an SDK helper in `crates/prosopon-sdk/src/ir.rs` if it's a common pattern.
6. Bump `IR_SCHEMA_VERSION` in `crates/prosopon-core/src/lib.rs` only if the variant is non-additive (rare).

## When adding a new compositor

1. Create `crates/prosopon-compositor-<surface>/`.
2. Implement `prosopon_runtime::Compositor`.
3. Advertise capabilities accurately in `Capabilities { surfaces, max_fps, ... }`.
4. Add a surface note in `docs/surfaces/<surface>.md`.
5. Add a golden-file test for at least one fixture scene.

## Relationship to the broader Broomva workspace

- **Life** (`../life/`) — aios-protocol, arcan, lago, autonomic, plexus, sensorium, vigil, …
- **Mission Control** (`../../apps/mission-control/`) — desktop compositor consumer; will emit/consume Prosopon envelopes once stabilized.
- **Prompter** (`../../apps/prompter/`) — teleprompter; consumes Prosopon text + signal updates.
- **Relay** (`../life/crates/relay/`) — remote session transport; will tunnel Prosopon envelopes.

## License

Apache-2.0 (inherits Broomva default).
