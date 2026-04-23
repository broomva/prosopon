import type { Intent } from "../runtime/types";

export function EntityRef({ intent }: { intent: Extract<Intent, { type: "entity_ref" }> }) {
  const label = intent.label ?? `${intent.kind}:${intent.id}`;
  return (
    <span className="pgl-mono" style={{ color: "var(--pgl-accent)" }}>
      → {label}
    </span>
  );
}
