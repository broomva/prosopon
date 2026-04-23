// Lightweight code block — no syntax highlight in v0.2 (Shiki pull-in deferred).
import type { Intent } from "../runtime/types";

export function Code({ intent }: { intent: Extract<Intent, { type: "code" }> }) {
  return (
    <pre className="pgl-code" data-lang={intent.lang}>
      <code>{intent.source}</code>
    </pre>
  );
}
