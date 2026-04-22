# Surface: Spatial (Vision Pro / Quest)

**Status:** planned for v0.4.0

## What it will be

A volumetric compositor for mixed-reality headsets. Rooms = projects,
constellations = active contexts, agents as pickup-able objects. The Arcan
Glass design language extruded from 2D panels into 3D materials.

## Why this matters

Spatial compositing forces us to answer a question the 2D world papered over:
**where does a piece of agent intent live in space?** The answer, for
Prosopon, is in the `Locus` and `Formation` intents — already part of the v0.1
IR. Building this compositor is the acid test for whether those intents were
correctly specified.

## Design sketch

- **Target platforms** — visionOS (first-class, Swift bridge) and Meta Quest
  (OpenXR + Unity or Godot). Start with visionOS.
- **Coordinate frame** — `SpatialFrame::Viewer` for heads-up elements,
  `Scene` for room-anchored, `World` for geo-anchored AR.
- **Volumetric primitives** — `Locus` = glass orb, `Formation` = swarm mesh,
  `Field` = 3D volume rendering, text as spatial label.
- **Interaction** — eye-gaze selects; hand-pinch acts as `ActionKind::Focus`
  or `Submit`; voice as `ActionKind::Input`.

## Intent → spatial primitive

| Intent | Primitive |
|---|---|
| `Locus` | Glass orb at position, size = priority. |
| `Formation` | Swarm mesh whose motion is parameterized by the topic. |
| `Field { Volume }` | 3D volume rendering. |
| `Section` / `Prose` | Floating panel anchored per `attrs.frame`. |
| `Choice` | Floating selector above the referencing node. |
| `ToolCall` / `ToolResult` | Tether-connected annotation near the emitting agent. |
| `Stream` | Scrolling ribbon across the field of view. |

## Module sketch

```
crates/prosopon-compositor-spatial/
  ├── src/
  │   └── lib.rs           # host-side runtime (state, event application)
  └── visionos/            # Xcode project (Swift + RealityKit)
      ├── ProsoponSpatial/
      │   ├── App.swift
      │   ├── SceneBridge.swift   # maps Prosopon events → RealityKit entities
      │   └── Intents/            # per-variant entity factories
      └── shaders/
```

## Open questions

- **Session persistence.** Should scenes persist across app launches? Probably
  yes, as serialized `Scene` + last-known signals.
- **Multi-user.** Can two people share the same spatial scene? Layered on top
  of Plexus quorum, but the compositor needs to think about per-user viewpoints.
- **Reality anchoring.** Do we use plane detection to pin `SpatialFrame::World`
  objects, or do we require an explicit anchor step?
