import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";

export function Confirm({
  intent,
  nodeId,
}: {
  intent: Extract<Intent, { type: "confirm" }>;
  nodeId: string;
}) {
  const { emitAction } = useRegistry();
  const color = severityColor(intent.severity ?? "info");
  return (
    <div className="pgl-card" style={{ borderColor: color }}>
      <p style={{ color }}>? {intent.message}</p>
      <div className="pgl-flex-row">
        <button
          type="button"
          onClick={() =>
            emitAction({ slot: "confirm", source: nodeId, kind: { type: "confirm", accepted: true } })
          }
        >
          Confirm
        </button>
        <button
          type="button"
          onClick={() =>
            emitAction({
              slot: "confirm",
              source: nodeId,
              kind: { type: "confirm", accepted: false },
            })
          }
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

function severityColor(s: string): string {
  switch (s) {
    case "danger":
      return "var(--pgl-danger)";
    case "warning":
      return "var(--pgl-warning)";
    case "notice":
      return "var(--pgl-accent)";
    default:
      return "var(--pgl-text-dim)";
  }
}
