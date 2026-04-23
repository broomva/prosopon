import type { Intent } from "../runtime/types";

export function Citation({ intent }: { intent: Extract<Intent, { type: "citation" }> }) {
  return (
    <cite className="pgl-dim pgl-mono">
      cite: {intent.source}
      {intent.anchor ? `#${intent.anchor}` : ""}
    </cite>
  );
}
