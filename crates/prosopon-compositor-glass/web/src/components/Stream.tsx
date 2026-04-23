import type { Intent } from "../runtime/types";

export function Stream({ intent }: { intent: Extract<Intent, { type: "stream" }> }) {
  return (
    <div className="pgl-mono pgl-dim" data-stream-id={intent.id}>
      ⟳ stream:{intent.id} ({intent.kind})
    </div>
  );
}
