import type { Intent } from "../runtime/types";

export function Video({ intent }: { intent: Extract<Intent, { type: "video" }> }) {
  return (
    <video poster={intent.poster} controls style={{ maxWidth: "100%" }}>
      <source src={intent.uri} />
      <track kind="captions" />
    </video>
  );
}
