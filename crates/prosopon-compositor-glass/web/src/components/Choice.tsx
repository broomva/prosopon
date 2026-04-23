import { useRegistry } from "../registry/context";
import type { Intent } from "../runtime/types";

export function Choice({
  intent,
  nodeId,
}: {
  intent: Extract<Intent, { type: "choice" }>;
  nodeId: string;
}) {
  const { emitAction } = useRegistry();
  return (
    <div className="pgl-card pgl-flex-col">
      <strong>{intent.prompt}</strong>
      <div className="pgl-flex-col">
        {intent.options.map((o) => (
          <button
            key={o.id}
            type="button"
            onClick={() =>
              emitAction({ slot: o.id, source: nodeId, kind: { type: "choose", option_id: o.id } })
            }
          >
            {o.default ? "● " : "○ "}
            {o.label}
            {o.description ? <span className="pgl-dim"> — {o.description}</span> : null}
          </button>
        ))}
      </div>
    </div>
  );
}
