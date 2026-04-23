# CONTROL.md — quality setpoints

This file is the authoritative source for the setpoints enforced by `.control/policy.yaml`.
Every change is measured against these — if one is relaxed, log the deviation here.

## Setpoints

### S1 — Compilation
- [x] **S1.1** `cargo check --workspace --all-targets` has zero errors.
- [x] **S1.2** `cargo clippy --workspace --all-targets -- -D warnings` has zero errors (warnings currently allowed in v0; see deviations).

### S2 — Tests
- [x] **S2.1** `cargo test --workspace` passes (v0.1: 49 tests, 0 failures).
- [ ] **S2.2** Every public type in `prosopon-core` has a serde roundtrip test. *(v0.1: partial; v0.2 target)*
- [~] **S2.3** Every compositor has at least one golden-file test. *(text + glass shipped v0.2.0-alpha; field/spatial/audio target v0.3+)*

### S3 — Formatting + lint
- [x] **S3.1** `cargo fmt --all -- --check` passes.
- [x] **S3.2** No `#[allow(warnings)]` at crate level.

### S4 — IR stability
- [x] **S4.1** `prosopon-core::IR_SCHEMA_VERSION` is bumped on any non-additive change.
- [x] **S4.2** `prosopon-protocol::PROTOCOL_VERSION` is bumped on any wire-incompatible change.
- [x] **S4.3** All tagged enums that are likely to grow carry `#[non_exhaustive]`.

### S5 — Documentation
- [x] **S5.1** Every RFC under `docs/rfcs/` has a unique number and status line.
- [x] **S5.2** New Intent variants are documented in their source file AND in RFC-0001.
- [ ] **S5.3** Every public type has rustdoc. *(v0.1: core + pneuma + compositor-text are documented; runtime + sdk partial; v0.2 target)*

### S6 — Dependency hygiene
- [x] **S6.1** Workspace dependencies declared centrally; crate-level `Cargo.toml` uses `.workspace = true`.
- [x] **S6.2** `prosopon-core` has no async / tokio dependency.
- [x] **S6.3** `prosopon-pneuma` has no dependency on `aios-protocol` (yet) — migration tracked in RFC-0003.

### S7 — Boundaries
- [x] **S7.1** `prosopon-core` has no styling / color / typography types.
- [x] **S7.2** Compositors do not import from each other — they depend only on `prosopon-core` + `prosopon-runtime`.
- [x] **S7.3** `prosopon-runtime::Compositor` remains object-safe so compositors can be boxed.

## Deviations (log)

- **2026-04-21** — `#![warn(missing_docs)]` disabled on `prosopon-core/runtime/protocol/compositor-text/sdk/pneuma` during v0.1 bootstrap. Target: re-enable in v0.2 with full coverage of public surfaces.
- **2026-04-21** — `#![warn(clippy::pedantic)]` disabled on `prosopon-core`. Reason: 157 warnings on first pass; not v0.1-critical. Target: address in v0.2.

## How to change a setpoint

1. Open a PR with the proposed change to this file and `.control/policy.yaml`.
2. Include rationale.
3. Merge only with review.
