// tests/goldens.test.tsx
// Cross-surface golden snapshots. The same fixture scenes drive vitest
// outerHTML snapshots here and insta text-compositor snapshots on the Rust
// side. If an intent's rendered shape changes in either surface, both
// goldens fail and force a conscious review.

import { render } from "@testing-library/preact";
import { describe, expect, it } from "vitest";
import { NodeView } from "../src/components/Node";
import { RegistryContext } from "../src/registry/context";
import { SignalBus } from "../src/runtime/signal-bus";
import type { Scene } from "../src/runtime/types";

import demo from "./fixtures/demo_scene.json";
import stream from "./fixtures/streaming_tokens.json";
import toolFlow from "./fixtures/tool_flow.json";

function renderScene(scene: Scene): string {
  const bus = new SignalBus();
  const { container } = render(
    <RegistryContext.Provider value={{ scene, bus, emitAction: () => {} }}>
      <NodeView node={scene.root} />
    </RegistryContext.Provider>,
  );
  return container.outerHTML;
}

describe("glass compositor goldens", () => {
  it.each([
    ["demo_scene", demo as unknown as Scene],
    ["tool_flow", toolFlow as unknown as Scene],
    ["streaming_tokens", stream as unknown as Scene],
  ])("%s matches snapshot", (_name, scene) => {
    expect(renderScene(scene)).toMatchSnapshot();
  });
});
