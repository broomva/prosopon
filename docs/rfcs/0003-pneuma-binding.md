# RFC-0003 — Pneuma Binding

- **Status:** Provisional (upstream trait pending)
- **Shipped in:** v0.1.0 (local mirror only)
- **Owns:** `prosopon-pneuma`

---

## Context

The Pneuma trait family (designed in the Life Agent OS, specified in
`core/life/docs/specs/pneuma-trait-surface.md`) formalizes inter-boundary
observation/control flow across the RCS hierarchy. Each boundary has a
`Pneuma<B: Boundary>` implementation with associated types:

- `Signal` — upward-flowing payload.
- `Aggregate` — upward-readable snapshot.
- `Directive` — downward control.

Examples already planned upstream:

| Impl | Boundary |
|---|---|
| `lago-journal` | `Pneuma<L0ToL1>` |
| `autonomic-controller` | `Pneuma<L1ToL2>` |
| `egri` | `Pneuma<L2ToL3>` |
| `bstack-policy` | `Pneuma<L3ToExternal>` |
| `life-plexus` | `Pneuma<D0ToD1>` (horizontal) |
| `sensorium` | `Pneuma<ExternalToL0>` (world → plant) |

Prosopon is the missing outward-facing sibling of Sensorium.

## Decision

**Prosopon implements `Pneuma<B = L0ToExternal>`** — the plant-level boundary
where agent activity becomes visible to human observers. It is the mirror of
Sensorium.

```
  Sensorium : Pneuma<ExternalToL0>   (world → plant; the eyes)
  Prosopon  : Pneuma<L0ToExternal>   (plant → observer; the face)
```

## Associated type mapping

| Pneuma concept | Prosopon type | Semantics |
|---|---|---|
| `Signal` | `ProsoponEvent` | Outward-flowing event stream — scene resets, patches, signals, chunks. |
| `Aggregate` | `Scene` | Current renderable snapshot, pollable by upper levels. |
| `Directive` | `ActionKind` | User-originated actions from a compositor, flowing back in. |
| `SubstrateProfile` | `prosopon_pneuma::SubstrateProfile` | Default: `classical_silicon` with warp 1.0. |

## Boundary markers

This crate defines three markers locally, mirroring the spec:

```rust
pub struct Vertical;      impl Axis for Vertical {}
pub struct L0ToExternal;  impl Boundary for L0ToExternal { type A = Vertical; const NAME = "l0_to_external"; }
pub struct ExternalToL0;  impl Boundary for ExternalToL0 { type A = Vertical; const NAME = "external_to_l0"; }
```

`ExternalToL0` is included purely so downstream code can reference the mirror
boundary symbolically; Sensorium owns the actual implementation.

## Why L0 and not L3?

The first instinct is `L3ToExternal` ("governance exposes to the world"). But:

1. `L3ToExternal` is already reserved upstream for `bstack-policy` — it carries
   governance directives (setpoints, gates, profiles) outward to external
   systems.
2. What Prosopon renders is overwhelmingly L0 plant activity: tool calls,
   prose, decisions, live signals. Not governance.
3. Matching Sensorium's `ExternalToL0` at the same boundary produces the
   cleanest architectural pair — world→plant, plant→world — and frees `L3`
   for its existing purpose.

Upper levels (L1/L2/L3) MAY surface their state through Prosopon by emitting
`ProsoponEvent`s; they do not need their own `LnToExternal` boundaries unless
they require a *separate* outward channel (e.g. a privileged governance
console).

## Migration plan (upstream land)

When `aios_protocol::pneuma` exists in the Life workspace:

1. Delete the local `Pneuma`, `Boundary`, `Axis`, `Vertical`, `Horizontal`,
   `L0ToExternal`, `ExternalToL0`, `SubstrateProfile`, `SubstrateKind`,
   `PneumaError` from `prosopon-pneuma/src/lib.rs`.
2. Replace with `pub use aios_protocol::pneuma::*;`.
3. Keep the Prosopon-specific type aliases (`ProsoponSignal = ProsoponEvent`,
   etc.) in place.
4. Add `aios-protocol` to `prosopon-pneuma/Cargo.toml`.
5. Bump `prosopon-pneuma` minor version (0.2.0).
6. Update this RFC status from Provisional → Accepted.

The change is a single commit affecting only `prosopon-pneuma`. No other crate
depends on the local definitions.

## Coexistence during transition

Until upstream lands, the local mirror is authoritative for Prosopon's build.
When downstream Life crates want to interoperate (e.g. arcan emitting
ProsoponEvents through a Pneuma channel), they can either:

- Use the local mirror directly (fine for an internal experiment).
- Wait for the upstream land to ship a cohesive integration.

Both paths are supported.

## Invariants to preserve

- `L0ToExternal::NAME == "l0_to_external"` — serialized substrate registry
  entries may reference this string.
- `ProsoponSignal = ProsoponEvent` — never wrap `ProsoponEvent` in an additional
  envelope at the Pneuma layer; use `prosopon-protocol::Envelope` for wire
  framing instead.
- `ProsoponDirective = ActionKind` — the narrow contract that "only user-origin
  actions flow inward through Prosopon" is load-bearing; other inward
  directives should use their own Pneuma boundary.
