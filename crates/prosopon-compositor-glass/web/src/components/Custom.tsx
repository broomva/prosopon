import type { Intent } from "../runtime/types";

export function Custom({ intent }: { intent: Extract<Intent, { type: "custom" }> }) {
  return (
    <div className="pgl-fallback pgl-mono">
      [{intent.kind}] {JSON.stringify(intent.payload)}
    </div>
  );
}
