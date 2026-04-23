// src/registry/context.ts
import { createContext } from "preact";
import { useContext } from "preact/hooks";
import type { SignalBus } from "../runtime/signal-bus";
import type { Scene, SignalValue } from "../runtime/types";

export interface ActionEmitPayload {
  slot: string;
  source: string;
  kind: unknown;
}

export interface RegistryCtx {
  scene: Scene;
  bus: SignalBus;
  emitAction: (payload: ActionEmitPayload) => void;
}

// biome-ignore lint/style/noNonNullAssertion: provider asserts this in layout
export const RegistryContext = createContext<RegistryCtx>(null!);

export function useRegistry(): RegistryCtx {
  const ctx = useContext(RegistryContext);
  if (!ctx) throw new Error("useRegistry outside of RegistryContext");
  return ctx;
}

export function useSignalSnapshot(): Record<string, SignalValue> {
  const { scene } = useRegistry();
  return scene.signals;
}
