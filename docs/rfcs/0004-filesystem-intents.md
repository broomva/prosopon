# RFC-0004 — Filesystem Intents

- **Status:** Draft
- **Owns:** `prosopon-core`
- **Depends on:** RFC-0001 (IR Schema)
- **First consumer:** `broomva.tech/life/[project]` — FileTree + Preview panes

---

## Motivation

Today, when an agent performs a filesystem operation (read a file, write a note,
create a workspace artifact), there is no first-class Intent for it. Everyone
who wants to surface it reaches for `Intent::Custom { kind: "fs.op", payload }`,
then stamps a bespoke payload shape for the read / write distinction and the
path + content + byte count. That works, but:

- **No discoverability.** A compositor that doesn't know `"fs.op"` falls back
  to generic-payload rendering, losing the semantics a file operation carries
  (a path is clickable; a diff is renderable; bytes are human-formattable).
- **Every consumer invents its own payload shape.** The Life page's current
  emitter uses `{ path, op, content, title, bytes }` keys inside
  `Custom.payload`. Any second consumer (Mission Control, Prompter, a future
  IDE skin) has to mirror that shape exactly or do its own translation.
- **Lifecycle semantics don't line up.** A write-in-progress looks like any
  other pending `Custom` node; a completed write can't be distinguished from
  a failed one except by reading payload keys.
- **No typed round-trip.** The envelope adapter on the Life side has to
  narrow an opaque `Value` payload into strongly-typed Chat / Preview state
  every tick. The adapter would prefer to receive a typed `Intent::FileWrite`
  once and let the type system carry the rest.

The existing `ToolCall` / `ToolResult` pair is the closest analog — it pulls
one specific category out of the general `Custom` escape hatch because every
surface wants to specialize rendering. Filesystem operations have earned the
same promotion.

## Proposal

Add two first-class Intent variants:

```rust
// ─────────────────── Filesystem ────────────────────
/// A filesystem read. Compositors MAY render as a clickable path pointing
/// at the file content (the `content` slot fills in when the read completes).
FileRead {
    path: String,
    /// The content that was read. Absent while the read is in-flight;
    /// populated once complete via `NodePatch` or `StreamChunk`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Byte count of `content`. Optional; compositors may compute from
    /// `content.len()` if absent but the emitter knows it authoritatively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bytes: Option<u64>,
    /// MIME type hint. Enables syntax highlighting, preview mode switching,
    /// etc. SHOULD default to `text/plain` when the reader doesn't know.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mime: Option<String>,
},
/// A filesystem write. The typical lifecycle is:
/// 1. `NodeAdded` with `FileWrite { path, op, content: None }` when the agent
///    decides to write. Lifecycle pending.
/// 2. Optional `StreamChunk`s into a paired stream if the content is itself
///    a live stream (rare; most writes are atomic).
/// 3. `NodeUpdated` patching `content` + `bytes` when the write lands.
///    Lifecycle resolved.
FileWrite {
    path: String,
    /// The kind of write — `create` for new files, `write` for overwrite,
    /// `append` for concatenation, `delete` for deletion. `patch` reserved
    /// for future diff-based writes.
    op: FileWriteKind,
    /// The full body written. Absent while the write is in-flight; present
    /// once resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Byte count — authoritative when the emitter supplies it, else derive
    /// from `content.len()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bytes: Option<u64>,
    /// Optional human-readable title the agent chose for this artifact.
    /// Useful when `path` is a synthetic workspace path and the title is
    /// the user-facing name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// MIME type hint, as in `FileRead`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mime: Option<String>,
},
```

And the supporting enum:

```rust
/// What kind of write a `FileWrite` Intent represents.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum FileWriteKind {
    /// New file at `path`.
    Create,
    /// Overwrite an existing file at `path`.
    Write,
    /// Append content to an existing file.
    Append,
    /// Remove the file at `path`. `content` SHOULD be absent.
    Delete,
}
```

Placement in `intent.rs`: immediately after `ToolResult`, grouped with
Process. Filesystem operations are a specialization of process-effects on
persistent state.

## Design rationale

### Why `FileRead` and `FileWrite`, not `FileOp`?

Reads and writes have distinct lifecycles and distinct compositor
concerns. A read is pull-mode (content is a returned value); a write is
push-mode (content is an input). Consumers often want to render them
differently — reads feed the Preview pane, writes populate the FileTree
"modified" badges. Splitting the variants keeps pattern-matching clean and
lets each evolve independently.

### Why isn't `content` mandatory?

Following the `ToolCall` pattern: a `FileWrite` without `content` is a
"write in flight" — the agent has decided to write, the path is known, but
the body is still being computed. The pending lifecycle carries that state
without requiring the emitter to send a placeholder. When the write lands,
`NodeUpdated` patches the content in. Same rationale for `FileRead`.

### Why `mime` instead of a strict enum?

MIME types are open-ended by design (`text/x-rust`, `application/x-ipynb+json`,
`image/webp`, …). An enum would close that set; a free-form string with a
documented default lets compositors specialize for known MIME types
(`text/markdown` → rendered MD, `text/x-rust` → syntax-highlighted code)
and fall back to plain-text for unknown ones.

### Why not stream the content?

Most writes are atomic from the agent's perspective — the LLM produces the
full note, the runtime calls the filesystem once. For the rare case of
stream-into-file (log tailing, large exports), the agent can pair
`FileWrite { content: None }` with a `Stream` intent under the same parent
node; the compositor reads the stream and updates the FileWrite on final.
Adding a streaming mode to `FileWrite` itself would duplicate the `Stream`
variant without adding capability.

### Relationship to `ToolCall`

A `ToolCall` named `"fs.write"` today is semantically equivalent to a
`FileWrite` intent. The new variants don't deprecate `ToolCall` — agents
that reach the filesystem through a generic tool-call mechanism keep using
it. The new variants are for cases where the runtime has already committed
to surfacing the filesystem effect as a first-class intent (typical in
Life, where `fs_op` events are already a distinct RunEvent type).

## Text compositor rendering

The reference text compositor (`prosopon-compositor-text`) renders the new
variants as:

```
FileRead   → "READ  <path>" + content preview (first 3 lines) when resolved
FileWrite  → "<OP>  <path>" where <OP> ∈ {CREATE, WRITE, APPEND, DELETE},
             + title on the next line if present,
             + content preview (first 3 lines) when resolved,
             + "(<bytes> B)" trailing byte count
```

Placeholder rendering in pending state:

```
FileRead   → "READ  <path>  (reading…)"
FileWrite  → "<OP>  <path>  (writing…)"
```

## Emitter migration path (broomva.tech)

The Life page's `ProsoponEmitter` in `apps/chat/lib/life-runtime/prosopon-emitter.ts`
currently emits `fs_op` as:

```ts
Intent::Custom { kind: "fs.op", payload: { path, op, content, title, bytes } }
```

Migration (lands in a follow-up PR after this RFC is accepted and implemented):

```ts
// Reads
Intent::FileRead { path, content, bytes, mime }

// Writes — op narrowed to FileWriteKind
Intent::FileWrite { path, op: FileWriteKind, content, title, bytes, mime }
```

`EnvelopeAdapter` on the client side narrows `node_added` with
`Intent::FileWrite` → `ReplayEvent { kind: "fs-op", op: "write" | "create" | "append" | "delete" }`.
The Preview pane reads `FileWrite.content` directly instead of reaching into
`Custom.payload.content`.

## Compatibility

**Additive change.** `Intent` is `#[non_exhaustive]`, so adding two variants is
non-breaking for consumers that already handle `_` in a match arm. Older
consumers that don't yet know `FileRead` / `FileWrite` render them via the
`Intent` `Display` fallback (same behaviour as any unknown future variant).

No schema break:

- `IR_SCHEMA_VERSION` bumps `0.1.0` → `0.2.0` (minor — additive).
- `prosopon-protocol::PROTOCOL_VERSION` stays at `1` (wire format unchanged
  — new Intent types serialize under the same `type`-tagged enum).
- `prosopon-core::scene_schema_json()` + `event_schema_json()` automatically
  emit the new variants via `schemars`; TypeScript bindings
  (`@broomva/prosopon`) regenerate in the same PR.

## Well-known attribute keys

This RFC doesn't add new well-known attrs; compositors can still use the
existing `emphasis`, `semantic_role`, `width_hint`, `glass.variant` keys on
`FileRead` / `FileWrite` nodes without modification.

Two `fs.*` namespaced keys are reserved for future use; not specified here:

- `fs.diff_base` — path of the previous version, for compositors that want
  to render a diff instead of a bare content preview.
- `fs.sensitive` — boolean marker for compositors to redact.

Either may be promoted to a first-class field in a future RFC if usage
pattern crystallizes.

## Implementation checklist

- [ ] Add `FileRead`, `FileWrite`, `FileWriteKind` to `crates/prosopon-core/src/intent.rs`.
- [ ] Serde round-trip test for each variant (pending + resolved states).
- [ ] `crates/prosopon-compositor-text/src/render.rs::render_intent` handles both.
- [ ] Golden-file fixture under `crates/prosopon-compositor-text/tests/fixtures/` for a write-then-resolve sequence.
- [ ] SDK helper in `crates/prosopon-sdk/src/ir.rs` for the common shape:
      `ir::file_write(path, op, content)`, `ir::file_read(path)`.
- [ ] Bump `prosopon_core::IR_SCHEMA_VERSION` to `"0.2.0"` in `lib.rs`.
- [ ] `cargo run -p prosopon-cli -- schema scene > schemas/scene.json` +
      `schema event > schemas/event.json` regenerated and committed.
- [ ] `@broomva/prosopon` TS bindings regenerated via `bun run generate:types`.
- [ ] Update RFC-0001 "Process" row to include `FileRead`, `FileWrite`.

## Follow-up (separate PR after this one lands)

- `broomva.tech` emitter migration in `prosopon-emitter.ts` — swap
  `Custom { kind: "fs.op" }` → typed variants.
- `EnvelopeAdapter` branch for `node_added.intent.type === "file_write" | "file_read"`.
- `PreviewPane` reads `FileWrite.content` / `FileRead.content` directly.

## Open questions (deferred)

- **Diff rendering.** Should the IR carry `previous_content` for compositors
  that want a built-in diff view? Punt to a future RFC — most callers can
  fetch the prior version on their own side.
- **Line-range reads.** Some reads only want a slice (`FileRead` with
  `range: Range<usize>`). Unclear if that's worth first-class typing vs.
  letting it ride on `ToolCall { name: "fs.read_range" }` for now.
- **Permissions / ACL.** A `permissions` field on `FileWrite` would let the
  compositor render "write (sudo)" or "write (ro)" tags. No consumer asks
  for this today; defer until one does.

## Non-goals

This RFC is not a filesystem protocol. It describes how agents surface
filesystem *effects* to compositors — not how those effects interact with
the underlying OS, sandbox, or capability gate. Gate checks live in
`aios-policy` (host-side) and in `life-kernel-gate::StaticPolicyGate`
(kernel-side); the agent has already passed those checks by the time a
`FileWrite` intent leaves the runtime.
