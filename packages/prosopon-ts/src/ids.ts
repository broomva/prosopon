/**
 * Identifier helpers — Prosopon IDs are transparent UUID strings in the Rust
 * core. We mirror the minting behaviour (v4 UUIDs by default) so hand-rolled
 * TS agents produce IDs wire-compatible with Rust-emitted IDs.
 */

import { randomUUID } from "node:crypto";

type Branded<K, Brand> = K & { readonly __brand: Brand };

export type NodeId = Branded<string, "NodeId">;
export type SceneId = Branded<string, "SceneId">;
export type ActionId = Branded<string, "ActionId">;
export type StreamId = Branded<string, "StreamId">;
export type Topic = Branded<string, "Topic">;

function mint<B>(): Branded<string, B> {
  // Browsers expose crypto.randomUUID() globally; Node/Bun do too via the
  // imported `randomUUID` above. This tries global first so the code works
  // identically in edge runtimes without a `node:crypto` shim.
  const g = (globalThis as { crypto?: { randomUUID?(): string } }).crypto;
  if (g?.randomUUID) return g.randomUUID() as Branded<string, B>;
  return randomUUID() as Branded<string, B>;
}

export function newNodeId(raw?: string): NodeId {
  return (raw ?? mint<"NodeId">()) as NodeId;
}

export function newSceneId(raw?: string): SceneId {
  return (raw ?? mint<"SceneId">()) as SceneId;
}

export function newActionId(raw?: string): ActionId {
  return (raw ?? mint<"ActionId">()) as ActionId;
}

export function newStreamId(raw?: string): StreamId {
  return (raw ?? mint<"StreamId">()) as StreamId;
}

export function topic(name: string): Topic {
  return name as Topic;
}

/** Topic namespace-prefix matching (dot-aware), mirrors Rust `Topic::starts_with_namespace`. */
export function topicStartsWith(t: Topic, prefix: string): boolean {
  const s = t as unknown as string;
  if (s === prefix) return true;
  return s.startsWith(`${prefix}.`);
}
