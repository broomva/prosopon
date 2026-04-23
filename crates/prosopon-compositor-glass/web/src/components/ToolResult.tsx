import type { Intent } from "../runtime/types";

export function ToolResult({ intent }: { intent: Extract<Intent, { type: "tool_result" }> }) {
  const cls = intent.success ? "pgl-tool-result-ok" : "pgl-tool-result-err";
  const marker = intent.success ? "✓" : "✗";
  const s = JSON.stringify(intent.payload);
  return (
    <div className={`pgl-mono ${cls}`}>
      {marker} {s.length > 120 ? `${s.slice(0, 119)}…` : s}
    </div>
  );
}
