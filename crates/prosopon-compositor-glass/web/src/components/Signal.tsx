import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { previewSignal } from "../util/format";

export function Signal({ intent }: { intent: Extract<Intent, { type: "signal" }> }) {
  const { scene } = useRegistry();
  const value = scene.signals[intent.topic];
  return (
    <div className="pgl-mono pgl-dim">
      ~ {intent.topic} = {value ? previewSignal(value) : "<pending>"}
    </div>
  );
}
