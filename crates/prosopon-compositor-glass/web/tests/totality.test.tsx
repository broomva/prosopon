import { render } from "@testing-library/preact";
import type { ComponentChildren } from "preact";
import { describe, expect, it } from "vitest";
import { RegistryContext } from "../src/registry/context";
import { renderIntent } from "../src/registry/intents";
import { SignalBus } from "../src/runtime/signal-bus";
import type { Intent, Scene } from "../src/runtime/types";

const EMPTY_SCENE: Scene = {
  id: "s",
  root: {
    id: "root",
    intent: { type: "empty" },
    children: [],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { priority: "normal", status: { type: "active" } },
  },
  signals: {},
  hints: {},
};

function wrap(ui: ComponentChildren) {
  const bus = new SignalBus();
  return render(
    <RegistryContext.Provider value={{ scene: EMPTY_SCENE, bus, emitAction: () => {} }}>
      {ui}
    </RegistryContext.Provider>,
  );
}

const INTENTS: Intent[] = [
  { type: "prose", text: "hi" },
  { type: "code", lang: "rust", source: "fn main() {}" },
  { type: "math", source: "E=mc^2" },
  { type: "entity_ref", kind: "concept", id: "x", label: "X" },
  { type: "link", href: "https://example.org" },
  { type: "citation", source: "RFC-0001" },
  { type: "signal", topic: "t" },
  { type: "stream", id: "s", kind: "text" },
  { type: "choice", prompt: "?", options: [{ id: "a", label: "A" }] },
  { type: "confirm", message: "sure?" },
  { type: "input", prompt: "name", input: { kind: "text" } },
  { type: "tool_call", name: "search", args: {} },
  { type: "tool_result", success: true, payload: "ok" },
  { type: "progress", pct: 0.5 },
  { type: "section", title: "S" },
  { type: "divider" },
  { type: "image", uri: "/x.png" },
  { type: "audio", voice: "default" },
  { type: "video", uri: "/x.mp4" },
  { type: "empty" },
  { type: "custom", kind: "x", payload: null },
];

describe("intent registry totality", () => {
  for (const intent of INTENTS) {
    it(`renders ${intent.type} without throwing`, () => {
      expect(() => wrap(renderIntent(intent, "n1"))).not.toThrow();
    });
  }
});
