import type { Intent } from "../runtime/types";

export function Progress({ intent }: { intent: Extract<Intent, { type: "progress" }> }) {
  const pct = clamp(intent.pct ?? 0, 0, 1);
  return (
    <div className="pgl-progress">
      <div className="pgl-progress-track">
        <div className="pgl-progress-fill" style={{ width: `${(pct * 100).toFixed(0)}%` }} />
      </div>
      <span className="pgl-dim pgl-mono">{(pct * 100).toFixed(0)}%</span>
      {intent.label ? <span className="pgl-dim">{intent.label}</span> : null}
    </div>
  );
}

function clamp(n: number, lo: number, hi: number) {
  return n < lo ? lo : n > hi ? hi : n;
}
