import type { Intent } from "../runtime/types";

export function Prose({ intent }: { intent: Extract<Intent, { type: "prose" }> }) {
  return <p className="pgl-prose">{intent.text}</p>;
}
