// src/runtime/scene-store.ts
// Minimal SceneStore: applies events to a local Scene and exposes a reactive
// signal. Mirrors prosopon-runtime::SceneStore. Forward-compatible: unknown
// event variants are no-op rather than throw.

import { type Signal, signal } from "@preact/signals-core";
import type { Node, NodeId, NodePatch, ProsoponEvent, Scene } from "./types";

export interface SceneStore {
  /** Current scene (snapshot via signal.value). */
  readonly scene: () => Scene;
  /** Reactive signal — subscribe in Preact components via .value. */
  readonly signal: Signal<Scene>;
  /** Apply a single event. Unknown variants are silent no-ops. */
  apply(event: ProsoponEvent): void;
}

export function createSceneStore(initial: Scene): SceneStore {
  const s = signal<Scene>(initial);

  const apply = (event: ProsoponEvent) => {
    switch (event.type) {
      case "scene_reset":
        s.value = event.scene;
        return;
      case "node_added": {
        const next = deepClone(s.value);
        const parent = findNode(next.root, event.parent);
        if (parent) parent.children.push(event.node);
        s.value = next;
        return;
      }
      case "node_updated": {
        const next = deepClone(s.value);
        const target = findNode(next.root, event.id);
        if (target) applyPatch(target, event.patch);
        s.value = next;
        return;
      }
      case "node_removed": {
        const next = deepClone(s.value);
        removeNode(next.root, event.id);
        s.value = next;
        return;
      }
      case "signal_changed": {
        const next = deepClone(s.value);
        next.signals[event.topic] = event.value;
        s.value = next;
        return;
      }
      case "stream_chunk":
      case "action_emitted":
      case "heartbeat":
        // Pass-through — compositors handle directly via the transport layer.
        return;
      default:
        // Forward-compatible: future variants become no-ops.
        return;
    }
  };

  return {
    scene: () => s.value,
    signal: s,
    apply,
  };
}

function findNode(root: Node, id: NodeId): Node | undefined {
  if (root.id === id) return root;
  for (const child of root.children) {
    const found = findNode(child, id);
    if (found) return found;
  }
  return undefined;
}

function removeNode(root: Node, id: NodeId): boolean {
  const idx = root.children.findIndex((c) => c.id === id);
  if (idx >= 0) {
    root.children.splice(idx, 1);
    return true;
  }
  for (const child of root.children) {
    if (removeNode(child, id)) return true;
  }
  return false;
}

function applyPatch(node: Node, patch: NodePatch): void {
  if (patch.intent) node.intent = patch.intent;
  if (patch.attrs) Object.assign(node.attrs, patch.attrs);
  if (patch.lifecycle) Object.assign(node.lifecycle, patch.lifecycle);
  if (patch.children) {
    if (patch.children.op === "replace") node.children = patch.children.children;
    else if (patch.children.op === "append") node.children.push(patch.children.child);
    else if (patch.children.op === "remove") removeNode(node, patch.children.id);
  }
}

function deepClone<T>(v: T): T {
  return structuredClone(v);
}
