/**
 * AUTO-GENERATED — do not edit by hand.
 *
 * Source of truth: core/prosopon/crates/prosopon-core (Rust).
 * Regenerate with `bun run generate` from packages/prosopon-ts/.
 *
 * Generated on: 2026-04-24T01:17:55.398Z
 */
/* eslint-disable */
/* biome-ignore-all */

/**
 * Visual density preference. Compositors interpret this per-surface.
 */
export type Density = "compact" | "comfortable" | "spacious";
/**
 * Overall presentation bias for a scene.
 */
export type IntentProfile = "balanced" | "dense_technical" | "ambient_monitor" | "cinematic" | "conversational";
/**
 * The shape of surface a compositor renders onto.
 */
export type SurfaceKind = "text" | "two_d" | "three_d" | "shader" | "audio" | "spatial" | "tactile";
/**
 * What the action *means*. Compositors choose representation; agents react by semantics.
 */
export type ActionKind =
  | {
      kind: "submit";
      payload: unknown;
    }
  | {
      kind: "inspect";
      target: string;
    }
  | {
      kind: "focus";
      target: string;
    }
  | {
      args?: {
        [k: string]: unknown | undefined;
      };
      command: string;
      kind: "invoke";
    }
  | {
      comment?: string | null;
      kind: "feedback";
      valence: Valence;
    }
  | {
      kind: "choose";
      option_id: string;
    }
  | {
      kind: "input";
      value: unknown;
    }
  | {
      accepted: boolean;
      kind: "confirm";
    };
/**
 * Qualitative feedback direction.
 */
export type Valence = "positive" | "neutral" | "negative";
/**
 * Whether this slot should be offered to the user right now.
 */
export type Visibility = "visible" | "muted" | "hidden";
/**
 * What part of a node a binding updates.
 */
export type BindTarget =
  | {
      key: string;
      kind: "attr";
    }
  | {
      kind: "intent_slot";
      path: string;
    }
  | {
      id: string;
      kind: "child_content";
    };
/**
 * Optional transformation applied to the signal value before binding.
 *
 * Transforms are intentionally limited to pure, deterministic functions so compositors can execute them safely without a scripting runtime.
 */
export type Transform =
  | {
      kind: "identity";
    }
  | {
      kind: "format";
      template: string;
    }
  | {
      kind: "clamp";
      max: number;
      min: number;
    }
  | {
      decimals: number;
      kind: "round";
    }
  | {
      kind: "percent";
    };
/**
 * The semantic type of a node.
 *
 * See module docs for the design philosophy. Compositors SHOULD handle every variant gracefully; falling back to rendering via `Intent::Prose` of the variant's `Display` is acceptable when no better representation is available.
 */
export type Intent =
  | {
      text: string;
      type: "prose";
    }
  | {
      lang: string;
      source: string;
      type: "code";
    }
  | {
      source: string;
      type: "math";
    }
  | {
      id: string;
      kind: string;
      label?: string | null;
      type: "entity_ref";
    }
  | {
      href: string;
      label?: string | null;
      type: "link";
    }
  | {
      anchor?: string | null;
      source: string;
      type: "citation";
    }
  | {
      display?: SignalDisplay & string;
      topic: string;
      type: "signal";
    }
  | {
      id: string;
      kind: StreamKind;
      type: "stream";
    }
  | {
      options: ChoiceOption[];
      prompt: string;
      type: "choice";
    }
  | {
      message: string;
      severity?: Severity & string;
      type: "confirm";
    }
  | {
      default?: unknown;
      input: InputKind;
      prompt: string;
      type: "input";
    }
  | {
      args: unknown;
      name: string;
      stream?: string | null;
      type: "tool_call";
    }
  | {
      payload: unknown;
      success: boolean;
      type: "tool_result";
    }
  | {
      label?: string | null;
      pct?: number | null;
      type: "progress";
    }
  | {
      /**
       * Byte count. Compositors may compute from `content.len()` if the emitter doesn't supply it; supplying it authoritatively is preferred for multi-byte encodings.
       */
      bytes?: number | null;
      /**
       * File content once the read completes. Absent while in-flight.
       */
      content?: string | null;
      /**
       * MIME type hint — enables syntax highlighting and preview-mode switching. Compositors SHOULD assume `text/plain` when absent.
       */
      mime?: string | null;
      /**
       * Absolute or workspace-relative path. Compositors MAY render it as a clickable target. No scheme is implied; the emitter decides.
       */
      path: string;
      type: "file_read";
    }
  | {
      /**
       * Byte count — authoritative when supplied, else derive from `content.len()`.
       */
      bytes?: number | null;
      /**
       * The full body written. Absent until the write resolves. SHOULD be absent for `FileWriteKind::Delete`.
       */
      content?: string | null;
      /**
       * MIME type hint, as in `FileRead`.
       */
      mime?: string | null;
      /**
       * The kind of write. Constrained via `FileWriteKind`.
       */
      op: FileWriteKind;
      /**
       * Target path — see `FileRead::path`.
       */
      path: string;
      /**
       * Optional human-readable title the agent chose for this artifact. Useful when `path` is a synthetic workspace path and `title` is the user-facing name the compositor should surface.
       */
      title?: string | null;
      type: "file_write";
    }
  | {
      layout: GroupKind;
      type: "group";
    }
  | {
      collapsible?: boolean;
      title?: string | null;
      type: "section";
    }
  | {
      type: "divider";
    }
  | {
      projection: Projection;
      topic: string;
      type: "field";
    }
  | {
      frame: SpatialFrame;
      /**
       * @minItems 3
       * @maxItems 3
       */
      position: [number, number, number];
      type: "locus";
    }
  | {
      kind: FormationKind;
      topic: string;
      type: "formation";
    }
  | {
      alt?: string;
      type: "image";
      uri: string;
    }
  | {
      stream?: string | null;
      type: "audio";
      uri?: string | null;
      voice?: string | null;
    }
  | {
      poster?: string | null;
      type: "video";
      uri: string;
    }
  | {
      type: "empty";
    }
  | {
      kind: string;
      payload: unknown;
      type: "custom";
    };
/**
 * How a `Signal` intent is surfaced.
 */
export type SignalDisplay = "inline" | "sparkline" | "badge";
/**
 * The content of a `Stream` channel.
 */
export type StreamKind = "text" | "audio" | "binary" | "jsonl";
/**
 * Severity level for confirmation dialogs, warnings, and alerts.
 */
export type Severity = "info" | "notice" | "warning" | "danger";
/**
 * Shape of an `Input` intent.
 */
export type InputKind =
  | {
      kind: "text";
      multiline?: boolean;
    }
  | {
      kind: "number";
      max?: number | null;
      min?: number | null;
    }
  | {
      kind: "boolean";
    }
  | {
      kind: "date";
    }
  | {
      kind: "json";
    };
/**
 * What kind of filesystem write a `FileWrite` intent represents.
 *
 * `#[non_exhaustive]` so future variants (e.g. `Patch` for diff-based writes) land additively.
 */
export type FileWriteKind = "create" | "write" | "append" | "delete";
/**
 * How a `Group` lays out its children.
 */
export type GroupKind = "list" | "grid" | "sequence" | "parallel" | "stack";
/**
 * How a field is projected into the compositor's surface.
 */
export type Projection = "heatmap" | "contour" | "volume" | "summary";
/**
 * Frame of reference for a spatial locus.
 */
export type SpatialFrame = "viewer" | "world" | "geo";
/**
 * A coherent grouping of agents — renders as a cluster, swarm, or quorum.
 */
export type FormationKind = "quorum" | "swarm" | "geometric" | "stigmergy";
/**
 * Attention priority of a node.
 */
export type Priority = "ambient" | "normal" | "urgent" | "blocking";
/**
 * Discrete lifecycle state of a node.
 */
export type NodeStatus =
  | {
      kind: "active";
    }
  | {
      kind: "pending";
    }
  | {
      kind: "resolved";
    }
  | {
      kind: "failed";
      reason: string;
    }
  | {
      kind: "decaying";
      progress: number;
    };
/**
 * A concrete signal value pushed onto the bus at a given instant.
 *
 * Compositors MAY cache last-known values to render bindings when a signal has not yet emitted.
 */
export type SignalValue =
  | (
      | {
          kind: "scalar";
        }
      | (
          | {
              kind: "time_series";
            }
          | TimePoint[]
        )
      | (
          | {
              kind: "vector";
            }
          | number[]
        )
      | {
          kind: "event";
          payload: unknown;
        }
    )
  | undefined;
/**
 * A single event in the Prosopon stream.
 */
export type ProsoponEvent =
  | {
      scene: Scene;
      type: "scene_reset";
    }
  | {
      node: Node;
      parent: string;
      type: "node_added";
    }
  | {
      id: string;
      patch: NodePatch;
      type: "node_updated";
    }
  | {
      id: string;
      type: "node_removed";
    }
  | {
      topic: string;
      ts: string;
      type: "signal_changed";
      value: SignalValue | undefined;
    }
  | {
      chunk: StreamChunk;
      id: string;
      type: "stream_chunk";
    }
  | {
      kind: ActionKind;
      slot: string;
      source: string;
      type: "action_emitted";
    }
  | {
      ts: string;
      type: "heartbeat";
    };
/**
 * How a patch modifies a node's children.
 */
export type ChildrenPatch =
  | {
      children: Node[];
      op: "replace";
    }
  | {
      children: Node[];
      op: "append";
    }
  | {
      ids: string[];
      op: "remove";
    }
  | {
      op: "reorder";
      order: string[];
    };
/**
 * Payload variants carried in a stream chunk.
 */
export type ChunkPayload =
  | {
      encoding: "text";
      text: string;
    }
  | {
      data: string;
      encoding: "b64";
      mime?: string | null;
    }
  | {
      encoding: "json";
      value: unknown;
    };

/**
 * Combined Scene + ProsoponEvent schema for TS code-gen.
 */
/**
 * A complete snapshot of "what to render."
 */
export interface Scene {
  hints?: SceneHints;
  id: string;
  root: Node;
  /**
   * Last-known value for each referenced signal topic. Compositors that support push-based updates MAY subscribe and ignore this cache; compositors that only pull SHOULD treat this as the authoritative initial value.
   */
  signals?: {
    [k: string]: SignalValue | undefined;
  };
}
/**
 * Compositor-steering hints carried with every scene.
 */
export interface SceneHints {
  /**
   * Visual density preference.
   */
  density?: Density & string;
  /**
   * Overall intent profile — tells compositors whether to bias dense+technical or sparse+ambient presentations.
   */
  intent_profile?: IntentProfile & string;
  /**
   * BCP-47 locale (e.g. `"en-US"`, `"es-CO"`). Compositors localize formatting.
   */
  locale?: string | null;
  /**
   * Ordered preferred surfaces, most preferred first. A compositor MAY skip a scene if its surface is not listed; compositors SHOULD respect the first match.
   */
  preferred_surfaces?: SurfaceKind[];
  /**
   * Rough viewport budget — hints for truncation/layout. `None` = unbounded.
   */
  viewport?: Viewport | null;
}
/**
 * Rough viewport budget. Compositors with bounded surfaces (terminals, small screens) SHOULD respect these; surfaces without a natural viewport (audio, spatial) MAY ignore.
 */
export interface Viewport {
  cols: number;
  rows: number;
}
/**
 * A node in the Prosopon IR tree.
 */
export interface Node {
  actions?: ActionSlot[];
  /**
   * Free-form attribute bag for hints the compositor MAY use. Well-known keys: `emphasis` (`"low" | "normal" | "high"`), `semantic_role` (e.g. `"error"`, `"success"`), `width_hint` (fraction 0..=1), `voice` (TTS voice id).
   */
  attrs?: {
    [k: string]: unknown | undefined;
  };
  bindings?: Binding[];
  children?: Node[];
  id: string;
  intent: Intent;
  lifecycle?: Lifecycle;
}
/**
 * An interactive slot attached to a node.
 *
 * Slots are *declarative* — the node says "this action is available," the compositor decides how to present it (button, keybind, voice command, gesture), and the agent reacts to the emitted `ActionEvent`.
 */
export interface ActionSlot {
  enabled?: boolean;
  id: string;
  kind: ActionKind;
  label?: string | null;
  visibility?: Visibility & string;
}
/**
 * Binds a live signal to a rendering slot on a node.
 *
 * The compositor is responsible for subscribing to the signal and applying the (optional) transform before updating the target.
 */
export interface Binding {
  source: SignalRef;
  target: BindTarget;
  transform?: Transform | null;
}
/**
 * A reference to a live value. Supports nested paths for structured signals: `SignalRef { topic: "plexus.state", path: ["quorum", "ratio"] }` addresses the `ratio` field inside the `quorum` object inside the `plexus.state` signal.
 */
export interface SignalRef {
  path?: string[];
  topic: string;
}
/**
 * One of the options presented by a `Choice` intent.
 */
export interface ChoiceOption {
  default?: boolean;
  description?: string | null;
  /**
   * Stable identifier used in the corresponding `ActionKind::Choose`.
   */
  id: string;
  label: string;
}
/**
 * Temporal and attentional state of a node.
 */
export interface Lifecycle {
  /**
   * When this node was first emitted by the agent.
   */
  created_at: string;
  /**
   * Optional expiration. Compositors SHOULD remove or fade past this time.
   */
  expires_at?: string | null;
  /**
   * Attention priority — shapes placement, color, and notification behavior.
   */
  priority?: Priority & string;
  /**
   * Current status in the node's lifecycle.
   */
  status?: NodeStatus;
}
/**
 * Single point in a time-series signal.
 */
export interface TimePoint {
  t: string;
  v: unknown;
}
/**
 * An incremental update to a node — used by `ProsoponEvent::NodeUpdated`.
 *
 * All fields are optional; unset fields leave existing values untouched.
 */
export interface NodePatch {
  /**
   * Attribute updates. `None` values remove the key.
   */
  attrs?: {
    [k: string]: unknown | undefined;
  };
  children?: ChildrenPatch | null;
  intent?: Intent | null;
  lifecycle?: Lifecycle | null;
}
/**
 * A chunk of streaming payload.
 */
export interface StreamChunk {
  /**
   * True if this is the last chunk of the stream.
   */
  final_?: boolean;
  /**
   * Encoded payload. Interpretation depends on the `StreamKind` of the originating `Stream` intent.
   */
  payload: ChunkPayload;
  /**
   * Sequence number (monotonic per stream). Compositors drop out-of-order chunks.
   */
  seq: number;
}

