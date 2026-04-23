import { describe, expect, it } from "vitest";
import { createSceneStore } from "../src/runtime/scene-store";
import type { ProsoponEvent, Scene } from "../src/runtime/types";

const root: Scene = {
  id: "s1",
  root: {
    id: "root",
    intent: { type: "section", title: "Hello" },
    children: [],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { priority: "normal", status: { type: "active" } },
  },
  signals: {},
  hints: {},
};

describe("scene store", () => {
  it("applies scene_reset by replacing the scene", () => {
    const store = createSceneStore(root);
    const next: Scene = { ...root, id: "s2" };
    store.apply({ type: "scene_reset", scene: next });
    expect(store.scene().id).toBe("s2");
  });

  it("applies node_added under the given parent", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "node_added",
      parent: "root",
      node: {
        id: "c1",
        intent: { type: "prose", text: "child" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { priority: "normal", status: { type: "active" } },
      },
    });
    expect(store.scene().root.children).toHaveLength(1);
    expect(store.scene().root.children[0]?.id).toBe("c1");
  });

  it("applies node_removed by id", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "node_added",
      parent: "root",
      node: {
        id: "c1",
        intent: { type: "prose", text: "child" },
        children: [],
        bindings: [],
        actions: [],
        attrs: {},
        lifecycle: { priority: "normal", status: { type: "active" } },
      },
    });
    store.apply({ type: "node_removed", id: "c1" });
    expect(store.scene().root.children).toHaveLength(0);
  });

  it("signal_changed updates the signals cache", () => {
    const store = createSceneStore(root);
    store.apply({
      type: "signal_changed",
      topic: "t",
      value: { type: "scalar", value: 42 },
      ts: new Date().toISOString(),
    });
    expect(store.scene().signals.t).toEqual({ type: "scalar", value: 42 });
  });

  it("unknown event variant is a silent no-op", () => {
    const store = createSceneStore(root);
    const unknown = { type: "future_variant", foo: "bar" } as unknown as ProsoponEvent;
    expect(() => store.apply(unknown)).not.toThrow();
    expect(store.scene().id).toBe("s1");
  });
});
