/**
 * Pure reducer — `applyEvent(scene, event) → scene`.
 *
 * Mirrors the semantics of `prosopon_runtime::SceneStore` in Rust. Given a
 * current scene and an incoming ProsoponEvent, returns the updated scene.
 * Never mutates its inputs. Unknown / future event variants are passed
 * through as no-ops (forward-compat).
 *
 * This is intentionally tiny and dependency-free so it runs identically in
 * the browser, Node, Bun, and edge runtimes.
 */

import type {
  ChildrenPatch,
  NodePatch,
  ProsoponEvent,
  Scene,
  SceneNode,
  SignalValue,
} from "./types";

/**
 * Core reducer. Returns a fresh Scene on change; returns the input reference
 * on no-op so React + Zustand users can cheaply detect "nothing happened".
 */
export function applyEvent(scene: Scene, event: ProsoponEvent): Scene {
  // ProsoponEvent is tagged by `type` (snake_case). This dispatch matches
  // the Rust enum variants exactly.
  const ev = event as ProsoponEventNarrowed;
  switch (ev.type) {
    case "scene_reset":
      return ev.scene;

    case "node_added":
      return replaceNodeById(scene, ev.parent, (node) => ({
        ...node,
        children: [...(node.children ?? []), ev.node],
      }));

    case "node_updated":
      return replaceNodeById(scene, ev.id, (node) =>
        applyPatch(node, ev.patch),
      );

    case "node_removed":
      return removeNodeById(scene, ev.id);

    case "signal_changed":
      return {
        ...scene,
        signals: {
          ...(scene.signals ?? {}),
          [ev.topic]: ev.value,
        },
      };

    case "stream_chunk":
      // Stream chunks are handled by callers (typically: append to a text
      // buffer keyed by ev.id). The reducer itself can't know which node
      // owns the stream without additional bookkeeping, so we pass-through.
      // Callers should use `subscribeStream` in client.ts to get a stream
      // of chunks + their fanout target.
      return scene;

    case "action_emitted":
      // compositor → agent flow; not a scene update
      return scene;

    case "heartbeat":
      return scene;

    default:
      // Unknown event — forward-compat: ignore.
      return scene;
  }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

type ProsoponEventNarrowed = ProsoponEvent & { type: string };

function applyPatch(node: SceneNode, patch: NodePatch): SceneNode {
  let next = node;
  if (patch.intent !== undefined && patch.intent !== null) {
    next = { ...next, intent: patch.intent };
  }
  if (patch.attrs) {
    const attrs = { ...(next.attrs ?? {}) };
    for (const [k, v] of Object.entries(patch.attrs)) {
      if (v === null || v === undefined) {
        delete attrs[k];
      } else {
        attrs[k] = v;
      }
    }
    next = { ...next, attrs };
  }
  if (patch.lifecycle !== undefined && patch.lifecycle !== null) {
    next = { ...next, lifecycle: patch.lifecycle };
  }
  if (patch.children !== undefined && patch.children !== null) {
    next = {
      ...next,
      children: applyChildrenPatch(next.children ?? [], patch.children),
    };
  }
  return next;
}

function applyChildrenPatch(
  current: SceneNode[],
  patch: ChildrenPatch,
): SceneNode[] {
  const p = patch as ChildrenPatchNarrowed;
  switch (p.op) {
    case "replace":
      return p.children;
    case "append":
      return [...current, ...p.children];
    case "remove": {
      const removeSet = new Set(p.ids);
      return current.filter((c) => !removeSet.has(c.id));
    }
    case "reorder": {
      const byId = new Map(current.map((c) => [c.id, c]));
      return p.order
        .map((id) => byId.get(id))
        .filter((c): c is SceneNode => c !== undefined);
    }
    default:
      return current;
  }
}

type ChildrenPatchNarrowed =
  | { op: "replace"; children: SceneNode[] }
  | { op: "append"; children: SceneNode[] }
  | { op: "remove"; ids: string[] }
  | { op: "reorder"; order: string[] };

function replaceNodeById(
  scene: Scene,
  id: string,
  transform: (node: SceneNode) => SceneNode,
): Scene {
  const newRoot = replaceInTree(scene.root, id, transform);
  if (newRoot === scene.root) return scene;
  return { ...scene, root: newRoot };
}

function replaceInTree(
  node: SceneNode,
  id: string,
  transform: (node: SceneNode) => SceneNode,
): SceneNode {
  if (node.id === id) return transform(node);
  const children = node.children ?? [];
  let changed = false;
  const next: SceneNode[] = new Array(children.length);
  for (let i = 0; i < children.length; i++) {
    const c = children[i] as SceneNode;
    const nc = replaceInTree(c, id, transform);
    if (nc !== c) changed = true;
    next[i] = nc;
  }
  return changed ? { ...node, children: next } : node;
}

function removeNodeById(scene: Scene, id: string): Scene {
  if (scene.root.id === id) {
    // Removing the root leaves a scene with no content; mirror Rust behaviour
    // by returning the scene unchanged and logging a warning.
    // eslint-disable-next-line no-console
    console.warn(
      `[prosopon] ignoring node_removed for root id=${id} — not permitted`,
    );
    return scene;
  }
  const newRoot = filterTree(scene.root, id);
  if (newRoot === scene.root) return scene;
  return { ...scene, root: newRoot };
}

function filterTree(node: SceneNode, removeId: string): SceneNode {
  const children = node.children ?? [];
  const kept: SceneNode[] = [];
  let changed = false;
  for (const c of children) {
    if (c.id === removeId) {
      changed = true;
      continue;
    }
    const nc = filterTree(c, removeId);
    if (nc !== c) changed = true;
    kept.push(nc);
  }
  return changed ? { ...node, children: kept } : node;
}

/**
 * Utility — pull the current value of a signal topic off a scene.
 * Returns undefined if the signal has never been pushed.
 */
export function readSignal(scene: Scene, topic: string): SignalValue | undefined {
  return scene.signals?.[topic];
}
