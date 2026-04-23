// src/actions/emit.ts
// Helper for components to build and dispatch ActionEmitted frames.

import type { Transport } from "../runtime/transport";

export function makeActionEmitter(transport: Transport, sessionId: () => string) {
  let seq = 1;
  return (payload: { slot: string; source: string; kind: unknown }) => {
    transport.send({
      version: 1,
      session_id: sessionId(),
      seq: seq++,
      ts: new Date().toISOString(),
      event: {
        type: "action_emitted",
        slot: payload.slot,
        source: payload.source,
        kind: payload.kind,
      },
    });
  };
}
