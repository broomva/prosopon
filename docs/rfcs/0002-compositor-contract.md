# RFC-0002 — Compositor Contract

- **Status:** Accepted
- **Shipped in:** v0.1.0
- **Owns:** `prosopon-runtime::Compositor`, `prosopon-compositor-*` crates

---

## Motivation

If the IR is the *shared language*, the compositor is the *speaker*. For
polymorphism to be real, the trait every compositor implements must be narrow
enough that new surfaces can join without changing the IR, and total enough
that any scene renders to *some* output on any surface.

## The trait

```rust
pub trait Compositor: Send + 'static {
    fn id(&self) -> CompositorId;
    fn capabilities(&self) -> Capabilities;
    fn apply(&mut self, event: &ProsoponEvent) -> Result<(), CompositorError>;
    fn flush(&mut self) -> Result<(), CompositorError> { Ok(()) }
}
```

- **`id`** — stable string, for logs and error attribution.
- **`capabilities`** — self-description (surfaces, fps, streaming, signal push).
- **`apply`** — receive a single event. MUST handle all known variants; MAY
  produce a placeholder for unknown ones.
- **`flush`** — optional; required for compositors that buffer (double-buffered
  terminal, GPU command-list).

## Totality requirement

Compositors are **total** over `Intent`, `ProsoponEvent`, and
`BindTarget`. This means:

- No `panic!` paths from user-visible scenes.
- No silent drops — unknown variants MUST produce *some* rendered artifact
  (a placeholder, a log line, an empty cell) so a user knows something was there.
- `UnsupportedIntent(name)` is a valid error for truly unhandled variants, but
  should be rare; prefer graceful degradation.

## Capabilities

```rust
pub struct Capabilities {
    pub surfaces: Vec<SurfaceKind>,
    pub max_fps: Option<u32>,
    pub supports_signal_push: bool,
    pub supports_streaming: bool,
}
```

Compositors advertise honestly. Registries MAY match scenes to compositors
using `SceneHints::preferred_surfaces`; if no compositor matches, the runtime
SHOULD route to a fallback text compositor rather than drop the scene.

## Registration + routing

The `Runtime` owns a vector of boxed compositors. On every `submit(event)`:

1. Scene store applies the event (identity layer).
2. Signal bus updates last-known value + notifies subscribers (if `SignalChanged`).
3. Each compositor receives `apply(&event)` in registration order.

Compositors that only handle certain surfaces MAY filter by inspecting
`SceneHints::preferred_surfaces` — but MUST render *something* for every scene
to avoid silent dropouts.

## Error handling

```rust
pub enum CompositorError {
    Io(std::io::Error),
    UnsupportedIntent(String),
    Encoding(String),
    Backend(String),
}
```

Returned errors are attributed by the runtime to the originating `CompositorId`.
A compositor's error does NOT halt the runtime or other compositors; the
runtime SHOULD log and continue.

## Lifecycle + threading

- Compositors are `Send + 'static` and may be moved across threads.
- They are NOT required to be `Sync` — the runtime holds them behind a
  `Mutex` and serializes `apply` calls.
- Internal concurrency is permitted (e.g. a compositor MAY spawn a render thread
  on `apply`), but output ordering MUST match event ordering.

## Performance expectations (guidance, not contract)

| Surface | Target `apply` cost | Notes |
|---|---|---|
| Text | <1ms for small scenes, <10ms for full re-render at 80×24 | Blocking stdout writes acceptable |
| Glass (2D) | <4ms/frame at 60fps | Hydrate signals inline; defer layout to Yoga/WebGPU |
| Field (shader) | <8ms/frame at 60fps | Parameterize shader uniforms from Intent; avoid recompile per event |
| Audio | Real-time — latency matters more than throughput | Buffer ahead by 50ms |
| Spatial | 90fps target — same as native VR runtimes | Pre-compute mesh for static intents |

## Building a new compositor — checklist

1. Create `crates/prosopon-compositor-<surface>/`.
2. Implement `Compositor`.
3. Be total over every `Intent` variant and every `ProsoponEvent` variant
   (including `_` wildcard arms for forward compatibility with
   `#[non_exhaustive]`).
4. Publish accurate `Capabilities`.
5. Add a golden-file test fixture in `tests/` for at least one canonical scene.
6. Write `docs/surfaces/<surface>.md` describing the design choices, the
   mapping from Intent variants to surface-native primitives, and the known
   limitations.

## Open questions

- **Multi-compositor composition.** When two compositors target the same
  surface (e.g. both render to the same terminal), who wins? A later RFC will
  introduce z-order + output routing.
- **Dynamic compositor loading.** Load from a `.so` / `.dylib` at runtime? The
  trait is object-safe, so it's viable, but we haven't specified the ABI.
- **Back-pressure.** What happens when a compositor's backend (e.g. remote
  WebSocket) can't keep up? Currently the runtime does not back-pressure the
  agent; a later RFC will add a capability hint.
