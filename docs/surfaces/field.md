# Surface: Field (GPU shader)

**Status:** planned for v0.3.0

## What it will be

A GPU compositor that renders Prosopon scenes as living fields. Intents become
physical parameters of a reaction-diffusion / pheromone-field simulation;
agents appear as attractors; quorum appears as resonance; stigmergic traces
appear as decaying trails.

This is the compositor that *proves* the polymorphism thesis — the same IR
that renders as a terminal list on `text` becomes a visible force-field on
`field`. No new IR needed.

## Why this is the right move

The Plexus substrate (`Pneuma<D0ToD1>`) already exposes typed signals with
half-lives, gradients, quorum detection, and stigmergic traces. These are
*literally* the building blocks of GPU field simulations. A field compositor
doesn't simulate the agent system — it *is* the agent system's visualization.

## Design sketch

- **wgpu** for cross-platform GPU (Metal on macOS, Vulkan on Linux, D3D12 on
  Windows, WebGPU in browsers).
- **Compute shader** implements reaction-diffusion; fragment shader
  color-grades the result.
- **Uniforms** driven directly from Prosopon events:
  - `Intent::Field { topic, projection }` → bind buffer from signal cache.
  - `Intent::Locus { position }` → attractor at that 3D position.
  - `Intent::Formation { kind, topic }` → simulation template variant.
  - `SignalChanged { topic, value }` → buffer update.
- **Text overlay** for non-spatial Intents (Prose, ToolCall) as labeled
  annotations over the field — so the compositor is still *total* and doesn't
  silently drop non-spatial nodes.

## Visual language

| Intent | Visual |
|---|---|
| `Field { Heatmap }` | 2D heatmap with reaction-diffusion animation driven by signal values. |
| `Field { Volume }` | 3D volumetric rendering of the same source. |
| `Locus` | Glowing attractor; pulse frequency = priority. |
| `Formation { Quorum }` | Resonance pattern — concentric ripples; amplitude = ratio. |
| `Formation { Swarm }` | Particle flow aligned with local gradient. |
| `Formation { Stigmergy }` | Decaying trails; half-life from signal metadata. |
| `Signal` (non-field) | Small gauge orbiting the node's spatial position. |
| `ToolCall` | Annotated ring with tool name on axis. |
| `Prose` | Text overlay with line anchored to node's locus (or viewport-stable). |

## Module sketch

```
crates/prosopon-compositor-field/
  ├── src/
  │   ├── lib.rs          # Compositor impl
  │   ├── pipeline.rs     # wgpu setup
  │   ├── simulation.rs   # reaction-diffusion step
  │   ├── uniforms.rs     # IR → GPU buffer translation
  │   └── overlay.rs      # text labels for non-field intents
  └── shaders/
      ├── simulate.wgsl
      └── render.wgsl
```

## Research references

- **wlroots compositor architecture** — multiple backends, render to any
  platform surface.
- **Godot RenderingServer** — opaque renderer abstraction; we need the same
  insulation between simulation and surface.
- **Plexus paper P6 (horizontal composition stability)** — once published,
  its mathematical form constrains the simulation parameters used here.

## Open questions

- **Temporal coupling.** How tightly does the GPU simulation tick with the
  Prosopon event stream? Per-event step is expensive; free-running simulation
  with periodic sync is smoother but harder to reason about.
- **Human-in-the-loop.** If a human clicks on an attractor, how does that map
  back to an `ActionEmitted`? Probably as a `Focus { target }` on the node
  whose Locus is closest.
- **Scale.** How do we show 1,000 agents simultaneously without the frame
  budget collapsing? Likely tiled compute + LOD.
