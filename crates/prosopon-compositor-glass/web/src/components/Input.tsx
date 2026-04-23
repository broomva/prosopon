import { useId, useState } from "preact/hooks";
import { useRegistry } from "../registry/context";
import type { Intent } from "../runtime/types";

export function Input({
  intent,
  nodeId,
}: {
  intent: Extract<Intent, { type: "input" }>;
  nodeId: string;
}) {
  const { emitAction } = useRegistry();
  const [value, setValue] = useState<string>(String(intent.default ?? ""));
  const inputId = useId();
  return (
    <div className="pgl-flex-col">
      <label htmlFor={inputId}>{intent.prompt}</label>
      <input
        id={inputId}
        value={value}
        onInput={(e) => setValue((e.currentTarget as HTMLInputElement).value)}
        onKeyDown={(e) => {
          if (e.key === "Enter")
            emitAction({ slot: "input", source: nodeId, kind: { type: "input", value } });
        }}
      />
    </div>
  );
}
