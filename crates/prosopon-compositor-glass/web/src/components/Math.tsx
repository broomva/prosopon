import type { Intent } from "../runtime/types";

export function MathComponent({ intent }: { intent: Extract<Intent, { type: "math" }> }) {
  return <code className="pgl-mono pgl-dim">{intent.source}</code>;
}
