# Architecture

Prosopon is structured around a single question:

> *How do we let an agent emit what it means, and let many surfaces each render that meaning in the way they can?*

The answer is three layers, borrowed explicitly from the Flutter team's original insight: **Widget / Element / RenderObject**.

## The three layers

```
┌───────────────────────────────────────────────────────────────────────────────┐
│  Agent                                                                        │
│  (any language / any runtime; uses the SDK or writes JSON directly)           │
└───────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │  ProsoponEvent stream (JSON / JSONL)
                                    ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│  Widget layer  ─  prosopon-core                                               │
│  ─ Node / Intent / Scene / Binding / ActionSlot                               │
│  ─ Pure data. No state, no side effects.                                      │
│  ─ Answers "what is this node?" (intent), never "how does it look?"           │
└───────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│  Element layer  ─  prosopon-runtime                                           │
│  ─ SceneStore applies events; SignalBus routes live values.                   │
│  ─ Owns identity + diff; deduplicates compositor work.                        │
│  ─ Compositor trait — the handoff boundary to the RenderObject layer.         │
└───────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│  RenderObject layer  ─  prosopon-compositor-*                                 │
│  ─ Surface-specific renderers: text (here), 2D (planned), 3D / shader /       │
│    audio / spatial / tactile (planned).                                       │
│  ─ Each is authoritative for its surface's pixels / frames / samples.         │
└───────────────────────────────────────────────────────────────────────────────┘
```

The dependency direction is downward only. A compositor never reaches into the runtime's internal state, and the runtime never reaches into a compositor's surface.

## Crate dependency graph

```
prosopon-core ◄──┬── prosopon-protocol   ◄── prosopon-cli
                 ├── prosopon-runtime     ◄── prosopon-sdk
                 └── prosopon-pneuma      ◄── prosopon-compositor-text
                                                       ▲
                                                       │
                                                 prosopon-cli
                                                 hello-prosopon
```

`prosopon-core` has no internal dependencies. `prosopon-protocol` depends only on `prosopon-core`. `prosopon-runtime` depends on core + protocol. Compositors depend on core + runtime. The SDK depends on core + protocol + runtime. The CLI depends on everything.

This keeps the **protocol + IR contract** independent of runtime/tokio/serialization choices downstream. A compositor in another language only needs the JSON schema that `prosopon-core` publishes; it does not need to port the runtime.

## Intent design

The core `Intent` enum is the one place where opinions are concentrated:

- Categories are orthogonal: **Textual / Entity / Live / Decision / Process / Structural / Spatial / Media / Meta**.
- Every tagged enum uses `#[non_exhaustive]` so consumers don't break when we add a new category.
- The serde tag is `"type"` (not `"kind"`) to free `kind` as a natural field name in variants like `EntityRef { kind, id, label }`.
- `Intent::Custom { kind, payload }` is the escape hatch for surface-specific experiments; when it stabilizes, it graduates to a first-class variant.

See [RFC-0001](docs/rfcs/0001-ir-schema.md) for the full schema and rationale.

## Binding + signal flow

```
  agent emits:  SignalChanged { topic, value }
                         │
                         ▼
               SignalBus (last-known cache + broadcast channels)
                         │
                         │  push to subscribers
                         ▼
               Compositor.apply(SignalChanged)
                         │
                         │  at render time, compositor reads:
                         │    ─ scene's cached signals (guaranteed correct)
                         │    ─ bindings on each node (how to hydrate)
                         ▼
                     rendered output
```

Compositors may either (a) re-render the whole scene on every signal change (simple, correct) or (b) subscribe to the bus and update only the affected nodes (efficient, more complex). Both are supported; the reference text compositor uses (a).

## Boundary integration — Pneuma<L0ToExternal>

In the Life Agent OS [RCS](https://github.com/broomva/life) hierarchy, every boundary has a `Pneuma` implementation. Prosopon implements the outward-facing L0 boundary:

```
  Sensorium : Pneuma<ExternalToL0>   (world → plant; the eyes)
  Prosopon  : Pneuma<L0ToExternal>   (plant → observer; the face)
```

`prosopon-pneuma` ships a local mirror of the Pneuma trait (pending upstream landing in `aios-protocol`; see [RFC-0003](docs/rfcs/0003-pneuma-binding.md)) and maps the associated types:

- `Signal = ProsoponEvent`   (what the plant emits outward)
- `Aggregate = Scene`         (the rendered view, pollable by upper levels)
- `Directive = ActionKind`    (user-originated actions flowing back in)

Upstream migration, when `aios_protocol::pneuma` lands, is a one-line swap of the local trait for a `pub use`.

## Why these boundaries?

Each module boundary was chosen to enable **independent extension**:

| Boundary | Why |
|---|---|
| core ↔ protocol | The IR can outlive any one wire format. Future codecs (postcard, MessagePack, CBOR) slot in without touching core. |
| protocol ↔ runtime | The protocol is language-agnostic; a TypeScript compositor needs only the protocol, not the runtime. |
| runtime ↔ compositor | Compositors can live in separate repos, separate languages, or be dynamically loaded. The trait is the only contract. |
| compositor ↔ pneuma | Pneuma integration is a single narrow crate so prosopon is usable outside the Life Agent OS (no forced Life dependency). |

## Non-goals

- **Layout computation.** This is delegated to each compositor (yoga for 2D, wgpu for 3D, etc.). Prosopon provides intent + hints; compositors resolve geometry.
- **Styling.** No color/font/padding system. Compositors ship their own visual language.
- **Animation curves.** Lifecycle has decay/expiration semantics; specific animation tweens are a compositor concern.
- **Security / sandboxing.** Prosopon events are plain data. If a surface executes code (e.g. MCP Apps iframes), that sandbox is the compositor's responsibility — we do not ship an eval layer.

## Where this is going

The four compositors documented but not yet implemented represent the vision:

- **glass** — 2D web. Shadcn-style components hydrated from IR. Arcan Glass design language.
- **field** — GPU shader. Plexus field physics as reaction-diffusion; agents as attractors; quorum as resonance.
- **spatial** — Vision Pro / Quest. IR rendered volumetrically; rooms = projects.
- **audio** — Gemini TTS + ambient sonification. Agents have timbre; system health as soundscape.

See [`docs/surfaces/`](docs/surfaces/) for per-surface notes.
