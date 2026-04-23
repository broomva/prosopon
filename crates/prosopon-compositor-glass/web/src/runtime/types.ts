// src/runtime/types.ts
// Mirror of prosopon-core's JSON shape. Hand-written for v0.2; will be
// auto-generated from scene_schema_json() in a future ticket.

export type Topic = string;
export type NodeId = string;
export type StreamId = string;
export type ActionId = string;
export type SceneId = string;

export type SignalValue =
  | { type: "scalar"; value: number | string | boolean | null }
  | { type: "series"; points: Array<{ ts: string; value: number }> }
  | { type: "categorical"; label: string }
  | { type: "vector"; components: number[] };

export type Intent =
  | { type: "prose"; text: string }
  | { type: "code"; lang: string; source: string }
  | { type: "math"; source: string }
  | { type: "entity_ref"; kind: string; id: string; label?: string }
  | { type: "link"; href: string; label?: string }
  | { type: "citation"; source: string; anchor?: string }
  | { type: "signal"; topic: Topic; display?: "inline" | "sparkline" | "badge" }
  | { type: "stream"; id: StreamId; kind: "text" | "audio" | "binary" | "jsonl" }
  | { type: "choice"; prompt: string; options: Array<ChoiceOption> }
  | { type: "confirm"; message: string; severity?: Severity }
  | { type: "input"; prompt: string; input: InputKind; default?: unknown }
  | { type: "tool_call"; name: string; args: unknown; stream?: StreamId }
  | { type: "tool_result"; success: boolean; payload: unknown }
  | { type: "progress"; pct?: number; label?: string }
  | { type: "group"; layout: GroupKind }
  | { type: "section"; title?: string; collapsible?: boolean }
  | { type: "divider" }
  | { type: "field"; topic: Topic; projection: Projection }
  | { type: "locus"; frame: SpatialFrame; position: [number, number, number] }
  | { type: "formation"; topic: Topic; kind: FormationKind }
  | { type: "image"; uri: string; alt?: string }
  | { type: "audio"; uri?: string; stream?: StreamId; voice?: string }
  | { type: "video"; uri: string; poster?: string }
  | { type: "empty" }
  | { type: "custom"; kind: string; payload: unknown };

export type GroupKind = "list" | "grid" | "sequence" | "parallel" | "stack";
export type Severity = "info" | "notice" | "warning" | "danger";
export type Projection = "heatmap" | "contour" | "volume" | "summary";
export type SpatialFrame = "viewer" | "world" | "geo";
export type FormationKind = "quorum" | "swarm" | "geometric" | "stigmergy";

export type InputKind =
  | { kind: "text"; multiline?: boolean }
  | { kind: "number"; min?: number; max?: number }
  | { kind: "boolean" }
  | { kind: "date" }
  | { kind: "json" };

export interface ChoiceOption {
  id: string;
  label: string;
  description?: string;
  default?: boolean;
}

export interface Node {
  id: NodeId;
  intent: Intent;
  children: Node[];
  bindings: Binding[];
  actions: ActionSlot[];
  attrs: Record<string, unknown>;
  lifecycle: Lifecycle;
}

export interface Binding {
  source: { topic: Topic; path?: string };
  target:
    | { type: "attr"; key: string }
    | { type: "intent_slot"; path: string }
    | { type: "child_content"; id: NodeId };
  transform?:
    | { type: "identity" }
    | { type: "format"; template: string }
    | { type: "clamp"; min: number; max: number }
    | { type: "round"; places: number }
    | { type: "percent" };
}

export interface ActionSlot {
  id: ActionId;
  kind: ActionKind;
  label?: string;
  enabled: boolean;
  visibility: "hidden" | "visible" | "primary";
}

export type ActionKind =
  | { type: "submit"; form_id?: string }
  | { type: "inspect"; node: NodeId }
  | { type: "focus"; node: NodeId }
  | { type: "invoke"; command: string; args?: unknown }
  | { type: "feedback"; kind: "positive" | "negative" | "neutral" }
  | { type: "choose"; option_id: string }
  | { type: "input"; value: unknown }
  | { type: "confirm"; accepted: boolean };

export interface Lifecycle {
  created_at?: string;
  expires_at?: string;
  priority: "ambient" | "normal" | "urgent" | "blocking";
  status:
    | { type: "active" }
    | { type: "pending" }
    | { type: "resolved" }
    | { type: "failed"; reason: string }
    | { type: "decaying"; half_life_ms: number };
}

export interface Scene {
  id: SceneId;
  root: Node;
  signals: Record<Topic, SignalValue>;
  hints: SceneHints;
}

export interface SceneHints {
  preferred_surfaces?: Array<"text" | "two_d" | "three_d" | "shader" | "audio" | "spatial" | "tactile">;
  intent_profile?: "balanced" | "dense_technical" | "ambient_monitor" | "cinematic" | "conversational";
  locale?: string;
  density?: "compact" | "comfortable" | "spacious";
  viewport?: { cols: number; rows: number };
}

export type StreamChunk = {
  seq: number;
  payload:
    | { encoding: "text"; text: string }
    | { encoding: "b64"; data: string; mime?: string }
    | { encoding: "json"; value: unknown };
  final_?: boolean;
};

export type ProsoponEvent =
  | { type: "scene_reset"; scene: Scene }
  | { type: "node_added"; parent: NodeId; node: Node }
  | { type: "node_updated"; id: NodeId; patch: NodePatch }
  | { type: "node_removed"; id: NodeId }
  | { type: "signal_changed"; topic: Topic; value: SignalValue; ts: string }
  | { type: "stream_chunk"; id: StreamId; chunk: StreamChunk }
  | { type: "action_emitted"; slot: ActionId; source: NodeId; kind: ActionKind }
  | { type: "heartbeat"; ts: string };

export interface NodePatch {
  intent?: Intent;
  attrs?: Record<string, unknown>;
  lifecycle?: Partial<Lifecycle>;
  children?: { op: "replace"; children: Node[] } | { op: "append"; child: Node } | { op: "remove"; id: NodeId };
}

export interface Envelope {
  version: number;
  session_id: string;
  seq: number;
  ts: string;
  event: ProsoponEvent;
}
