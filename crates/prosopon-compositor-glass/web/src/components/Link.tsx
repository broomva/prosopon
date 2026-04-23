import type { Intent } from "../runtime/types";

export function Link({ intent }: { intent: Extract<Intent, { type: "link" }> }) {
  return (
    <a href={intent.href} style={{ color: "var(--pgl-accent)" }} rel="noopener">
      {intent.label ?? intent.href}
    </a>
  );
}
