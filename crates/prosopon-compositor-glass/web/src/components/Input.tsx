import type { Intent } from "../runtime/types";
import { useRegistry } from "../registry/context";
import { useState } from "preact/hooks";

export function Input({
  intent,
  nodeId,
}: {
  intent: Extract<Intent, { type: "input" }>;
  nodeId: string;
}) {
  const { emitAction } = useRegistry();
  const [value, setValue] = useState<string>(String(intent.default ?? ""));
  return (
    <div className="pgl-flex-col">
      <label>{intent.prompt}</label>
      <input
        value={value}
        onInput={(e) => setValue((e.currentTarget as HTMLInputElement).value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") emitAction({ slot: "input", source: nodeId, kind: { type: "input", value } });
        }}
      />
    </div>
  );
}
