import type { Intent } from "../runtime/types";

export function Audio({ intent }: { intent: Extract<Intent, { type: "audio" }> }) {
  if (intent.uri) {
    return (
      <audio controls>
        <source src={intent.uri} />
        <track kind="captions" />
      </audio>
    );
  }
  return <div className="pgl-mono pgl-dim">♪ live audio (voice: {intent.voice ?? "default"})</div>;
}
