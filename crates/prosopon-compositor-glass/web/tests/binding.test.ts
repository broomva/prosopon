import { describe, expect, it } from "vitest";
import { hydrateIntent } from "../src/util/binding";
import type { Binding, Intent, SignalValue } from "../src/runtime/types";

describe("binding hydration", () => {
  it("resolves Progress.pct from attr:pct binding", () => {
    const intent: Intent = { type: "progress", pct: 0, label: "scoring" };
    const bindings: Binding[] = [
      {
        source: { topic: "job.pct" },
        target: { type: "attr", key: "pct" },
      },
    ];
    const signals: Record<string, SignalValue> = {
      "job.pct": { type: "scalar", value: 0.42 },
    };
    const out = hydrateIntent(intent, bindings, signals);
    expect(out).toEqual({ type: "progress", pct: 0.42, label: "scoring" });
  });

  it("intent_slot binding resolves via path", () => {
    const intent: Intent = { type: "prose", text: "placeholder" };
    const bindings: Binding[] = [
      {
        source: { topic: "greeting" },
        target: { type: "intent_slot", path: "text" },
      },
    ];
    const signals: Record<string, SignalValue> = {
      greeting: { type: "scalar", value: "hello" },
    };
    const out = hydrateIntent(intent, bindings, signals);
    expect(out).toEqual({ type: "prose", text: "hello" });
  });

  it("missing signal leaves intent unchanged", () => {
    const intent: Intent = { type: "progress", pct: 0.1 };
    const bindings: Binding[] = [
      { source: { topic: "missing" }, target: { type: "attr", key: "pct" } },
    ];
    const out = hydrateIntent(intent, bindings, {});
    expect(out).toEqual(intent);
  });
});
