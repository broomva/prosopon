# Continuation Prompt — Glass compositor + Arcan emit

**Paste the block below into a fresh Claude Code session started in `~/broomva/`.** It is self-contained and assumes no memory of the prior session.

---

## Prompt to paste

You are resuming work on **Prosopon** — a Rust-native display server for AI agents living at `~/broomva/core/prosopon/`. v0.1.0 shipped on 2026-04-21 (commit `ea35625`, docs cross-refs in `4bd1967`). Full architectural reasoning is preserved in:

- `~/broomva/research/notes/2026-04-21-prosopon-display-server-synthesis.md` — session synthesis
- `~/broomva/core/prosopon/README.md` — repo overview
- `~/broomva/core/prosopon/ARCHITECTURE.md` — three-layer model (Widget / Element / RenderObject mapped to core / runtime / compositor)
- `~/broomva/core/prosopon/docs/positioning.md` — a16z-thesis memo
- `~/broomva/core/prosopon/docs/rfcs/0001-ir-schema.md` — IR spec
- `~/broomva/core/prosopon/docs/rfcs/0002-compositor-contract.md` — compositor trait + totality requirement
- `~/broomva/core/prosopon/docs/rfcs/0003-pneuma-binding.md` — Pneuma<L0ToExternal> binding
- `~/broomva/core/prosopon/docs/surfaces/glass.md` — the surface note for the work you are about to do

Linear project: https://linear.app/broomva/project/prosopon-display-server-for-agents-b75c23a05005 (22 tickets across 4 milestones; 1 Done, 21 Backlog).

**Your task:** execute **BRO-767** — `prosopon-compositor-glass` — the 2D web compositor, Arcan Glass styled, consuming Prosopon IR over WebSocket/SSE. Then pick up **BRO-773** — Arcan session view emits through Prosopon envelopes — to turn the glass demo from scripted to live.

### Sequencing (do NOT skip steps)

1. **Orient.** Read the files listed above in order. Run `make smoke` and `cargo run -p prosopon-cli -- demo` to confirm the v0.1 surface is green on your machine.
2. **Plan.** Use `superpowers:writing-plans` to produce a concrete implementation plan for BRO-767 before touching code. Post the plan to the Linear ticket as a comment.
3. **Scaffold.** Create `crates/prosopon-compositor-glass/` as a hybrid Rust + TypeScript crate:
   - Rust side: axum HTTP + WebSocket server that subscribes to a `Runtime` and fans envelopes out as JSONL over WS + SSE. Serves the compiled web/ assets via `include_dir!`.
   - TS side (`web/`, bun workspace): `scene store + signal bus` mirroring the Rust runtime's semantics using `@preact/signals-core`; `registry/intents.ts` mapping each `Intent` variant to a React/Preact component; `@chenglou/pretext` for text measurement; Yoga for layout. Design tokens from `arcan-glass`.
4. **Cover every Intent variant.** Per RFC-0002's totality requirement, every variant (including `Custom` and the `_` wildcard for forwards-compatibility) renders *something*. Start with: Prose, Code, Section, Divider, Progress, ToolCall, ToolResult, Choice, Confirm, Signal, Stream. Field/Locus/Formation get placeholders that point to the field compositor.
5. **Golden files.** Add TS snapshot tests (vitest) against the same fixture scenes that text compositor uses. Cross-surface parity is the point.
6. **Demo page.** Deploy a standalone page at `broomva.tech/prosopon/demo` showing text + glass side-by-side with the envelope stream in a third panel. Use the content pipeline and Arcan Glass skills.
7. **Commit + PR + CI.** Follow `AGENTS.md` and `CONTROL.md` conventions: pre-commit hook, conventional commits, PR title `BRO-767: ...`, link the Linear ticket, tests must stay green.
8. **Then BRO-773.** Create `arcan-prosopon` inside Life at `core/life/crates/arcan/arcan-prosopon/`. Subscribe to arcan's event bus (Lago journal), translate `EventKind` variants to `ProsoponEvent`s via a `translator.rs` module, push into a `Runtime` that is served by `prosopon-daemon` (BRO-768, may need to land in parallel). Fall back gracefully if the daemon is unreachable — arcan-console must keep working without Prosopon.

### Conventions (inherit from workspace)

- **Rust 2024, MSRV 1.85, resolver = "2", Apache-2.0.**
- **`thiserror` for libraries, `anyhow` for apps, `tracing` for logs, `serde` everywhere.** Never `println!`.
- **Bun** for the TS side (not npm/yarn). **Biome** for TS lint/format (not ESLint/Prettier).
- **MCP Linear, never linear CLI** (the CLI defaults to the stimulus workspace per memory rule).
- **Gemini TTS** (`gemini-2.5-flash-preview-tts`, Kore voice) for any audio — never edge-tts.
- **Conventional commits**: `feat:`, `fix:`, `docs:`, `chore:`, …
- **Knowledge graph**: save non-obvious decisions as entity pages under `research/entities/` following the existing frontmatter format.

### Architectural invariants (do not violate)

- `prosopon-core` stays pure — no tokio, no I/O, no async. Every additional dep is a deviation logged in CONTROL.md.
- Compositors are total over `Intent` and `ProsoponEvent` — always produce *something*, never panic, always include the `_` wildcard arm for `#[non_exhaustive]` forward-compat.
- No styling primitives in the IR (color, font, padding). If you need one, add a well-known attribute key and document it in RFC-0001.
- `PROTOCOL_VERSION` bumps on any wire-incompatible change; `IR_SCHEMA_VERSION` bumps on any non-additive IR change.

### What counts as done

- BRO-767 closed with a merged PR; golden-file snapshot tests green; demo page live at `broomva.tech/prosopon/demo`.
- BRO-773 closed with arcan session events rendering live through the glass compositor attached to `prosopon-daemon`.
- Session synthesis note appended (or a new one created) summarizing the decisions made during this continuation.
- A v0.2.0 tag applied once BRO-763…769 (the full v0.2 milestone) are all Done.

### Handoff fallback

If any step of the above is ambiguous or a dependency blocks you (e.g. prosopon-daemon BRO-768 isn't ready when you start BRO-773), pause, post a short status comment on the relevant Linear ticket, and pick up the next-highest-priority item from the v0.2 backlog (likely BRO-768 daemon, or one of the compat bridges BRO-763/764).

**Do not rush past the "Plan" step.** The superpowers writing-plans skill exists because rapid scaffolding of compositor infrastructure routinely introduces architectural drift that costs more to fix later than the plan costs to produce.

Start by reading the orientation files, running smoke, and writing the plan.

---

## Why this prompt is structured this way

- **Self-contained.** A fresh session with no memory can execute it.
- **File-ordered orientation.** Reading order enforces understanding substrate → runtime → contract → target surface before opening an editor.
- **Explicit skill invocation.** Names the superpowers skill so the agent invokes it rather than improvising.
- **Invariants preserved.** Re-states the architectural constraints so they do not erode through a fresh session that has not read AGENTS.md.
- **Graceful degradation.** Includes a handoff fallback so the agent does not block on an unavailable dependency.
- **Ends with "Start by ..."** — prevents a blank-stare opening that consumes a turn without progress.
