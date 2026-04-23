// src/util/binding.ts
// Resolve Bindings against a signal cache and return a hydrated Intent clone.
// Mirrors prosopon-compositor-text::hydrate_intent (RFC-0001 §Binding resolution).

import type { Binding, Intent, SignalValue } from "../runtime/types";

export function hydrateIntent(
  intent: Intent,
  bindings: readonly Binding[],
  signals: Readonly<Record<string, SignalValue>>,
): Intent {
  if (bindings.length === 0) return intent;
  let out: Intent = structuredClone(intent);
  for (const b of bindings) {
    const value = signals[b.source.topic];
    if (!value) continue;
    if (value.type !== "scalar") continue;
    const raw = value.value;
    if (b.target.type === "attr") {
      out = applyAttrBinding(out, b.target.key, raw);
    } else if (b.target.type === "intent_slot") {
      out = applyIntentSlotBinding(out, b.target.path, raw);
    }
    // child_content is handled at component level — not here.
  }
  return out;
}

function applyAttrBinding(intent: Intent, key: string, raw: unknown): Intent {
  if (intent.type === "progress" && key === "pct" && typeof raw === "number") {
    return { ...intent, pct: raw };
  }
  // Generic fallback: unhandled attr bindings leave the intent alone.
  return intent;
}

function applyIntentSlotBinding(intent: Intent, path: string, raw: unknown): Intent {
  if (intent.type === "prose" && path === "text" && typeof raw === "string") {
    return { ...intent, text: raw };
  }
  if (intent.type === "progress" && path === "pct" && typeof raw === "number") {
    return { ...intent, pct: raw };
  }
  return intent;
}
