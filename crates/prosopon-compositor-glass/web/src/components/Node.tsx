// src/components/Node.tsx
// Generic Node wrapper — handles lifecycle, actions, children. Each intent
// component handles its own body; this wrapper supplies priority styling and
// action buttons.

import type { Node as ProsoponNode } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { hydrateIntent } from "../util/binding";
import { renderIntent } from "../registry/intents";

export function NodeView({ node }: { node: ProsoponNode }) {
  const { scene } = useRegistry();
  const hydrated = hydrateIntent(node.intent, node.bindings, scene.signals);
  const priorityClass = priorityClassFor(node.lifecycle.priority);

  return (
    <div className={`pgl-node ${priorityClass}`} data-node-id={node.id}>
      {renderIntent(hydrated, node.id)}
      {node.children.map((child) => (
        <NodeView key={child.id} node={child} />
      ))}
      <ActionBar node={node} />
    </div>
  );
}

function ActionBar({ node }: { node: ProsoponNode }) {
  const { emitAction } = useRegistry();
  const visible = node.actions.filter((a) => a.visibility !== "hidden");
  if (visible.length === 0) return null;
  return (
    <div className="pgl-flex-row" style={{ marginTop: "var(--pgl-space-2)" }}>
      {visible.map((a) => (
        <button
          key={a.id}
          type="button"
          disabled={!a.enabled}
          onClick={() => emitAction({ slot: a.id, source: node.id, kind: a.kind })}
        >
          {a.label ?? a.kind.type}
        </button>
      ))}
    </div>
  );
}

function priorityClassFor(p: ProsoponNode["lifecycle"]["priority"]): string {
  switch (p) {
    case "blocking":
      return "pgl-prio-blocking";
    case "urgent":
      return "pgl-prio-urgent";
    case "ambient":
      return "pgl-prio-ambient";
    default:
      return "";
  }
}
