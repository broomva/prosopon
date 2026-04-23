import type { Intent } from "../runtime/types";

export function Image({ intent }: { intent: Extract<Intent, { type: "image" }> }) {
  return <img src={intent.uri} alt={intent.alt ?? ""} style={{ maxWidth: "100%" }} />;
}
