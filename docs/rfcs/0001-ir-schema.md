# RFC-0001 — The Prosopon IR Schema

- **Status:** Accepted
- **Shipped in:** v0.1.0
- **Owns:** `prosopon-core`

---

## Motivation

Agents emit ad-hoc text today. Some emit tool-call JSON. Some emit HTML in a
chatbox. Each surface has to reinvent "what did the agent mean by this chunk?"
The cost is that every agent-UI product builds its own micro-protocol, and no
two of them can compose.

The Prosopon IR is the semantic-intent layer every surface shares:

- agents say **what is this thing** (not **how to show it**)
- compositors decide **how to show it** (not **what it is**)

This RFC specifies the IR.

## Design axioms

1. **Intent-heavy, appearance-light.** No styling primitives.
2. **Additive-first.** `#[non_exhaustive]` on growable enums.
3. **JSON-native.** `Value` is `serde_json::Value`; codecs are JSON and JSONL
   for v0.1.
4. **Schema-published.** `prosopon-core::scene_schema_json()` and
   `event_schema_json()` emit JSON Schema for cross-language consumption.
5. **Compositor-polymorphic.** Every variant must be renderable in principle
   on Text / 2D / 3D / Shader / Audio / Spatial surfaces, even if fallback
   representations are coarse.

## The type tree

```
Scene
├── id: SceneId
├── root: Node
├── signals: IndexMap<Topic, SignalValue>      (last-known value cache)
└── hints: SceneHints
           ├── preferred_surfaces: Vec<SurfaceKind>
           ├── intent_profile: IntentProfile
           ├── locale
           ├── density
           └── viewport

Node
├── id: NodeId
├── intent: Intent                              ← see Intent categories below
├── children: Vec<Node>
├── bindings: Vec<Binding>                      (reactive → this node)
├── actions: Vec<ActionSlot>                    (user input surfaces)
├── attrs: IndexMap<String, Value>              (compositor hints)
└── lifecycle: Lifecycle
           ├── created_at / expires_at
           ├── priority (Ambient | Normal | Urgent | Blocking)
           └── status (Active | Pending | Resolved | Failed | Decaying)

Binding
├── source: SignalRef { topic, path }
├── target: BindTarget (Attr | IntentSlot | ChildContent)
└── transform: Option<Transform> (Identity | Format | Clamp | Round | Percent)

ActionSlot
├── id: ActionId
├── kind: ActionKind (Submit | Inspect | Focus | Invoke | Feedback | Choose | Input | Confirm)
├── label, enabled, visibility
```

## Intent categories

All `Intent` variants are tagged with `"type"` in JSON (not `"kind"`, since
multiple variants carry a natural `kind` field such as
`EntityRef { kind, id, label }`).

| Category | Variants |
|---|---|
| Textual | `Prose`, `Code`, `Math` |
| Entity | `EntityRef`, `Link`, `Citation` |
| Live state | `Signal`, `Stream` |
| Decision surface | `Choice`, `Confirm`, `Input` |
| Process | `ToolCall`, `ToolResult`, `Progress`, `FileRead`, `FileWrite` (RFC-0004) |
| Structural | `Group { layout }`, `Section`, `Divider` |
| Spatial | `Field`, `Locus`, `Formation` |
| Media | `Image`, `Audio`, `Video` |
| Meta | `Empty`, `Custom { kind, payload }` |

Guidelines:

- Prefer the most specific variant. `ToolCall` beats `Custom { kind: "tool_call" }`
  because downstream compositors can specialize rendering without knowing the
  specific tool.
- Use `Custom` for experimental intents. When they stabilize across surfaces,
  promote to a first-class variant and deprecate the custom form.
- Do *not* add styling fields to Intent variants. If a compositor needs a hint,
  add it to `Node.attrs` with a well-known key documented here.

## Well-known attribute keys

Compositor-specific hints MAY use a dotted namespace (`glass.*`, `text.*`,
`field.*`, `audio.*`, `spatial.*`) so multi-surface scenes can carry
per-compositor overrides without conflict. Unnamespaced keys apply across
compositors.

| Key | Value | Applies to | Meaning |
|---|---|---|---|
| `emphasis` | `"low"` \| `"normal"` \| `"high"` | any | Visual prominence |
| `semantic_role` | `"error"` \| `"success"` \| `"info"` \| `"warning"` | any | Semantic classification the compositor MAY color |
| `width_hint` | `0.0..=1.0` | structural | Preferred fraction of available width |
| `voice` | voice id (e.g. `"gemini.kore"`) | Audio, Prose (for TTS) | Preferred TTS voice |
| `density_hint` | `"compact"` \| `"comfortable"` \| `"spacious"` | structural | Per-subtree override of scene density |
| `glass.variant` | `"card"` \| `"inline"` \| `"ambient"` | structural | Glass compositor override for the presentation density of the bearing node (card = bordered + padded; inline = flush; ambient = de-emphasized, reduced opacity). |

New well-known keys are added via PR updating this RFC.

## ProsoponEvent stream

Events are the *only* way the IR changes. Compositors apply them in order;
out-of-order application is undefined (sequence numbers from `prosopon-protocol`
let compositors detect gaps).

| Event | Effect |
|---|---|
| `SceneReset { scene }` | Replace the entire scene. |
| `NodeAdded { parent, node }` | Attach `node` under `parent`. |
| `NodeUpdated { id, patch }` | Apply `NodePatch` (intent / attrs / lifecycle / children). |
| `NodeRemoved { id }` | Drop node + subtree. |
| `SignalChanged { topic, value, ts }` | Update signal cache; notify subscribers. |
| `StreamChunk { id, chunk }` | Append to a `Stream` intent (token, audio frame, JSON line). |
| `ActionEmitted { slot, source, kind }` | User-origin event; flows compositor → agent. |
| `Heartbeat { ts }` | Liveness ping. |

All event variants are `#[non_exhaustive]`.

## Binding resolution

Bindings connect live signals to rendered output. Compositors MUST hydrate
bindings before rendering a node.

- `BindTarget::Attr { key }` — reads from the bound signal, writes to
  `Node.attrs[key]` at render time.
- `BindTarget::IntentSlot { path }` — targets a specific field inside an Intent
  (e.g. `"progress.pct"`). Compositors implement per-variant substitution; the
  reference text compositor handles `Progress.pct` in v0.1.
- `BindTarget::ChildContent { id }` — replaces the full content of a named
  child node.

`Transform` variants are pure functions a compositor can evaluate without
a scripting runtime. Future transforms will be added through this RFC rather
than by introducing an `Eval { expr }` escape.

## Version compatibility

- `IR_SCHEMA_VERSION` is a SemVer string in `prosopon-core`.
- Additive changes (new variants, new well-known attrs) bump the **minor**.
- Renames or removed variants bump the **major**.
- Wire-format changes (serde field names, tag names) bump the
  `prosopon-protocol::PROTOCOL_VERSION` integer.

## Open questions (deferred)

- **Ordered vs. unordered children.** Some intents (lists, timelines) want
  ordered; others (live logs) want bounded + ordered. Today we always preserve
  insertion order via `IndexMap`; a future `ordering_hint` attr may formalize
  intent-level semantics.
- **CRDT wire format.** For multi-user coherence, consider a CRDT layer above
  the event stream. Current `NodePatch` is a patch language, not a CRDT; upgrade
  path TBD.
- **Secrets.** Streams may carry sensitive data; there is no secrecy primitive
  in v0.1. Compositors responsible for redaction until a `secret: true` attr
  lands.
