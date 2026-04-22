# AGENTS.md

Operational conventions for any agent (human or otherwise) working in this repo.

## Before you change code

1. Read [`README.md`](README.md) for the mental model.
2. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) for the layer/crate boundaries.
3. Scan the relevant RFC in [`docs/rfcs/`](docs/rfcs/) before touching the IR or compositor contract.
4. Run `make smoke` — the repo must be green before you start.

## The control loop for every change

```
1. CHECK     — which .control/policy.yaml setpoints does this touch?
2. IMPLEMENT — smallest change that satisfies them.
3. MEASURE   — make smoke / make test.
4. VERIFY    — setpoints still green; no new `#[allow]`s without justification.
5. DOCUMENT  — update the RFC if the IR/contract changed; update CHANGELOG.
6. COMMIT    — conventional-commit style (`feat:`, `fix:`, `docs:`, …).
```

## Boundaries — what NOT to do

- **Do not add styling to `prosopon-core`.** No color, font, padding, icon names. Those belong to compositors.
- **Do not add tokio / async to `prosopon-core`.** The IR must be pure data — runnable from `no_std` consumers eventually.
- **Do not widen the `Pneuma` trait surface in this repo.** The canonical spec lives upstream in `core/life/docs/specs/pneuma-trait-surface.md`. Local deviations get flagged in RFC-0003.
- **Do not break the wire version silently.** Any change to `Envelope` or to the serialization shape of `ProsoponEvent` / `Scene` bumps `PROTOCOL_VERSION` or `IR_SCHEMA_VERSION`.

## What IS expected

- **New Intent variants** are free to add — `#[non_exhaustive]` covers forward compatibility. Document in RFC-0001 and the intent module docs.
- **New compositors** can live in separate crates (`crates/prosopon-compositor-<surface>/`). Implement `prosopon_runtime::Compositor`; publish a surface note in `docs/surfaces/`.
- **New codecs** go in `prosopon-protocol` as `Codec::<Variant>`.
- **Bridge crates** to MCP Apps / A2UI / AG-UI belong in `crates/prosopon-compat-*/` — they translate external IR into Prosopon IR.

## Testing discipline

- Every public type in `prosopon-core` must have at least one serde roundtrip test.
- Every compositor must have a golden-file test against a fixture scene (see `crates/prosopon-compositor-text/tests/` — to be added in v0.2).
- `cargo test --workspace` is the gate; it must stay green.

## Commits and PRs

- **One logical change per commit.** Don't bundle a new Intent with a refactor.
- **PR title is conventional-commit.** Body includes: Summary, RFC impact (y/n + which), Test plan.
- **Never merge with failing CI.** If you disagree with a gate, update the gate in `.control/policy.yaml` in a separate PR.

## Publishing

- Every workspace crate inherits `version.workspace = true`. Bumps happen at the workspace level.
- `cargo publish -p <crate>` is always manual for now; automate in v0.2.

## Agent-authored changes

If you are an LLM working in this repo:
- Add the `🤖 Generated with [Claude Code](https://claude.com/claude-code)` trailer to PRs.
- Never merge your own PR without human review for this phase (v0.1.x).
- Keep diffs tight — prefer one PR per RFC section rather than sweeping rewrites.

## Related

- [`CLAUDE.md`](CLAUDE.md) — Claude-specific shortcuts and context.
- [`PLANS.md`](PLANS.md) — roadmap and near-term milestones.
- [`CONTROL.md`](CONTROL.md) — control-systems setpoints this repo enforces.
