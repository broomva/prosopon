// src/registry/fallback.tsx
import type { IntentProps } from "./intents";

export function Fallback({ intent }: IntentProps) {
  return (
    <div className="pgl-fallback pgl-mono">
      [{intent.type}] — compositor surface upgrade suggested
    </div>
  );
}
