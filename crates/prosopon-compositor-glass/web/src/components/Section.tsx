import type { Intent } from "../runtime/types";

export function Section({ intent }: { intent: Extract<Intent, { type: "section" }> }) {
  if (!intent.title) return null;
  return (
    <div className="pgl-section">
      <h2 className="pgl-section-title">{intent.title}</h2>
      <div className="pgl-divider" />
    </div>
  );
}
