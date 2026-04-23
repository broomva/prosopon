// src/app.tsx
// Top-level App — owns the scene store, signal bus, and transport; wires them
// into the RegistryContext provider for all descendant components.
//
// IMPORTANT: the provider's `scene` must re-evaluate on every scene-store
// update. We mirror store.signal into useState via Signal.subscribe so the
// component re-renders whenever apply() writes s.value.

import { useEffect, useMemo, useState } from "preact/hooks";
import { makeActionEmitter } from "./actions/emit";
import { NodeView } from "./components/Node";
import { RegistryContext } from "./registry/context";
import { createSceneStore } from "./runtime/scene-store";
import { SignalBus } from "./runtime/signal-bus";
import { connectTransport, type Transport, type TransportState } from "./runtime/transport";
import type { Scene } from "./runtime/types";

const EMPTY_SCENE: Scene = {
  id: "empty",
  root: {
    id: "root",
    intent: { type: "prose", text: "Waiting for envelopes…" },
    children: [],
    bindings: [],
    actions: [],
    attrs: {},
    lifecycle: { priority: "normal", status: { type: "active" } },
  },
  signals: {},
  hints: {},
};

export function App() {
  const store = useMemo(() => createSceneStore(EMPTY_SCENE), []);
  const bus = useMemo(() => new SignalBus(), []);
  const [state, setState] = useState<TransportState>("connecting");
  const [lastSession, setLastSession] = useState("");
  const [transport, setTransport] = useState<Transport | null>(null);
  // Mirror the scene signal into React state so the Provider value
  // re-evaluates on every scene-store update.
  const [scene, setScene] = useState<Scene>(store.signal.value);

  useEffect(() => {
    const unsubscribe = store.signal.subscribe((next) => setScene(next));
    return unsubscribe;
  }, [store]);

  useEffect(() => {
    const base = typeof window !== "undefined" ? window.location.origin : "";
    const t = connectTransport(base);
    setTransport(t);
    const offState = t.onState(setState);
    const offEnv = t.onEnvelope((env) => {
      setLastSession(env.session_id);
      store.apply(env.event);
      if (env.event.type === "signal_changed") bus.publish(env.event.topic, env.event.value);
    });
    return () => {
      offState();
      offEnv();
      t.close();
    };
  }, [store, bus]);

  const emitAction = transport ? makeActionEmitter(transport, () => lastSession) : () => {};

  return (
    <div className="pgl-shell">
      <header
        className="pgl-flex-row"
        style={{ justifyContent: "space-between", alignItems: "baseline" }}
      >
        <strong>Prosopon · Glass</strong>
        <span className="pgl-dim pgl-mono">ws: {state}</span>
      </header>
      <main>
        <RegistryContext.Provider value={{ scene, bus, emitAction }}>
          <NodeView node={scene.root} />
        </RegistryContext.Provider>
      </main>
    </div>
  );
}
