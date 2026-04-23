import type { Intent } from "../runtime/types";

export function ToolCall({ intent }: { intent: Extract<Intent, { type: "tool_call" }> }) {
  return (
    <div className="pgl-tool-call">
      <span style={{ color: "var(--pgl-accent)" }}>⚙ </span>
      <strong>{intent.name}</strong>({jsonPreview(intent.args)})
    </div>
  );
}

function jsonPreview(v: unknown): string {
  const s = JSON.stringify(v);
  return s.length > 80 ? `${s.slice(0, 79)}…` : s;
}
