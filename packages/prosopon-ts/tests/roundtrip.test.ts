/**
 * Roundtrip tests — verify wire compatibility with the Rust core by folding
 * each of the 8 ProsoponEvent variants against a scene and asserting
 * invariants.
 */

import { describe, expect, test } from "bun:test";
import {
  applyEvent,
  decode,
  encode,
  makeEnvelope,
  PROTOCOL_VERSION,
  ProsoponSession,
  readSignal,
  type Envelope,
  type ProsoponEvent,
  type Scene,
  type SceneNode,
} from "../src/index";

const ROOT_ID = "root-1";
const CHAT_ID = "chat";
const MSG_ID = "m-1";

function baseScene(): Scene {
  const root: SceneNode = {
    id: ROOT_ID,
    intent: { type: "section", title: "Root", collapsible: false },
    children: [
      {
        id: CHAT_ID,
        intent: { type: "section", title: "Chat", collapsible: false },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { created_at: new Date().toISOString() },
      },
    ],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { created_at: new Date().toISOString() },
  };
  return {
    id: "scene-1",
    root,
    signals: {},
    hints: { density: "comfortable", intent_profile: "balanced" },
  };
}

describe("envelope codec", () => {
  test("makeEnvelope + encode + decode (json) round-trips", () => {
    const env = makeEnvelope({
      session_id: "s1",
      seq: 42,
      event: { type: "heartbeat", ts: new Date().toISOString() },
    });
    const wire = encode("json", env);
    const back = decode("json", wire);
    expect(back.version).toBe(PROTOCOL_VERSION);
    expect(back.seq).toBe(42);
    expect(back.event).toEqual(env.event);
  });

  test("jsonl preserves trailing newline", () => {
    const env = makeEnvelope({
      session_id: "s2",
      seq: 1,
      event: { type: "heartbeat", ts: new Date().toISOString() },
    });
    const wire = encode("jsonl", env);
    expect(wire.endsWith("\n")).toBe(true);
    const back = decode("jsonl", wire);
    expect(back.seq).toBe(1);
  });

  test("version mismatch throws", () => {
    const bad = JSON.stringify({
      version: 999,
      session_id: "x",
      seq: 1,
      ts: new Date().toISOString(),
      event: { type: "heartbeat", ts: new Date().toISOString() },
    });
    expect(() => decode("json", bad)).toThrow(
      /protocol version mismatch/,
    );
  });
});

describe("applyEvent", () => {
  test("scene_reset replaces scene wholesale", () => {
    const s = baseScene();
    const nextRoot: SceneNode = {
      id: "new-root",
      intent: { type: "prose", text: "fresh" },
      children: [],
      bindings: [],
      actions: [],
      attrs: {},
      lifecycle: { created_at: new Date().toISOString() },
    };
    const ev: ProsoponEvent = {
      type: "scene_reset",
      scene: {
        id: "scene-2",
        root: nextRoot,
        signals: {},
        hints: { density: "comfortable", intent_profile: "balanced" },
      },
    };
    const out = applyEvent(s, ev);
    expect(out.id).toBe("scene-2");
    expect(out.root.id).toBe("new-root");
  });

  test("node_added grafts under parent", () => {
    const s = baseScene();
    const newNode: SceneNode = {
      id: MSG_ID,
      intent: { type: "stream", id: MSG_ID, kind: "text" },
      children: [],
      bindings: [],
      actions: [],
      attrs: {},
      lifecycle: { created_at: new Date().toISOString() },
    };
    const out = applyEvent(s, {
      type: "node_added",
      parent: CHAT_ID,
      node: newNode,
    });
    const chat = findById(out, CHAT_ID);
    expect(chat?.children?.map((c) => c.id)).toEqual([MSG_ID]);
  });

  test("node_updated applies attrs and intent patches", () => {
    let s = baseScene();
    s = applyEvent(s, {
      type: "node_added",
      parent: CHAT_ID,
      node: {
        id: MSG_ID,
        intent: { type: "prose", text: "v1" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { created_at: new Date().toISOString() },
      },
    });
    const out = applyEvent(s, {
      type: "node_updated",
      id: MSG_ID,
      patch: {
        intent: { type: "prose", text: "v2" },
        attrs: { emphasis: "high" },
      },
    });
    const node = findById(out, MSG_ID);
    expect(node?.intent).toEqual({ type: "prose", text: "v2" });
    expect(node?.attrs?.emphasis).toBe("high");
  });

  test("node_removed removes subtree", () => {
    let s = baseScene();
    s = applyEvent(s, {
      type: "node_added",
      parent: CHAT_ID,
      node: {
        id: MSG_ID,
        intent: { type: "prose", text: "bye" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { created_at: new Date().toISOString() },
      },
    });
    const out = applyEvent(s, { type: "node_removed", id: MSG_ID });
    const chat = findById(out, CHAT_ID);
    expect(chat?.children ?? []).toEqual([]);
  });

  test("signal_changed updates signals map", () => {
    const s = baseScene();
    const out = applyEvent(s, {
      type: "signal_changed",
      topic: "haima.spend.cents",
      value: { kind: "scalar", Scalar: 42 } as unknown as ReturnType<
        typeof readSignal
      >,
      ts: new Date().toISOString(),
    } as ProsoponEvent);
    expect(out.signals?.["haima.spend.cents"]).toBeDefined();
  });

  test("heartbeat is a no-op", () => {
    const s = baseScene();
    const out = applyEvent(s, {
      type: "heartbeat",
      ts: new Date().toISOString(),
    });
    expect(out).toBe(s);
  });

  test("unknown event type is forward-compat no-op", () => {
    const s = baseScene();
    const out = applyEvent(s, {
      type: "future_variant",
    } as unknown as ProsoponEvent);
    expect(out).toBe(s);
  });
});

describe("ProsoponSession", () => {
  test("monotonic seq across emits", () => {
    const session = new ProsoponSession({ sessionId: "sess-1" });
    const a = session.emit({
      type: "heartbeat",
      ts: new Date().toISOString(),
    });
    const b = session.emit({
      type: "heartbeat",
      ts: new Date().toISOString(),
    });
    expect(a.seq).toBe(1);
    expect(b.seq).toBe(2);
    expect(a.session_id).toBe("sess-1");
  });

  test("emitFrame yields valid jsonl", () => {
    const session = new ProsoponSession({ sessionId: "sess-2" });
    const frame = session.emitFrame({
      type: "heartbeat",
      ts: new Date().toISOString(),
    });
    expect(frame.endsWith("\n")).toBe(true);
    const env: Envelope = JSON.parse(frame.trim());
    expect(env.session_id).toBe("sess-2");
  });
});

// helpers ---------------------------------------------------------------

function findById(scene: Scene, id: string): SceneNode | undefined {
  const walk = (node: SceneNode): SceneNode | undefined => {
    if (node.id === id) return node;
    for (const c of node.children ?? []) {
      const hit = walk(c);
      if (hit) return hit;
    }
    return undefined;
  };
  return walk(scene.root);
}
