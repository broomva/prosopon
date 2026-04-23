import type { SignalValue } from "../runtime/types";

export function previewSignal(v: SignalValue): string {
  switch (v.type) {
    case "scalar":
      return String(v.value);
    case "series":
      return `[${v.points.length} pts]`;
    case "categorical":
      return v.label;
    case "vector":
      return `[${v.components.join(", ")}]`;
    default:
      return "<?>";
  }
}
