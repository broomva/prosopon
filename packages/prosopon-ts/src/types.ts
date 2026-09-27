/**
 * Facade over the auto-generated types — re-exports with stable names and
 * a couple of ergonomic helpers. If `src/generated/types.ts` is regenerated
 * and an export name changes upstream, adjust here rather than at every
 * call-site.
 */

export type {
  // Scene root
  Scene,
  SceneHints,
  Viewport,
  Density,
  IntentProfile,
  SurfaceKind,
  // Nodes
  Node as SceneNode,
  NodePatch,
  ChildrenPatch,
  // Intent — the IR's semantic layer
  Intent,
  GroupKind,
  SignalDisplay,
  StreamKind,
  ChoiceOption,
  InputKind,
  Projection,
  SpatialFrame,
  FormationKind,
  // Lifecycle
  Lifecycle,
  NodeStatus,
  Priority,
  Severity,
  // Actions (compositor → agent)
  ActionSlot,
  ActionKind,
  Valence,
  Visibility,
  // Signals (reactive)
  Binding,
  BindTarget,
  Transform,
  SignalRef,
  SignalValue,
  TimePoint,
  // Events + stream payload
  ProsoponEvent,
  StreamChunk,
  ChunkPayload,
} from "./generated/types";
