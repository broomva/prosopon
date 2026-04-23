// src/runtime/transport.ts
// WebSocket client with reconnect backoff. Delivers decoded Envelope frames to
// subscribers. On WS close, reconnects after exponential backoff.

import type { Envelope } from "./types";

export type TransportState = "connecting" | "open" | "reconnecting" | "closed";

export interface Transport {
  onEnvelope(cb: (env: Envelope) => void): () => void;
  onState(cb: (state: TransportState) => void): () => void;
  send(frame: unknown): void;
  close(): void;
}

const BACKOFF_MS = [250, 500, 1000, 2000, 5000] as const;

export function connectTransport(baseUrl: string): Transport {
  const envCbs = new Set<(env: Envelope) => void>();
  const stateCbs = new Set<(s: TransportState) => void>();
  let ws: WebSocket | null = null;
  let state: TransportState = "connecting";
  let attempt = 0;
  let closed = false;

  function setState(s: TransportState) {
    state = s;
    for (const cb of stateCbs) cb(s);
  }

  function connect() {
    if (closed) return;
    ws = new WebSocket(`${baseUrl.replace(/^http/, "ws")}/ws`);
    ws.onopen = () => {
      attempt = 0;
      setState("open");
    };
    ws.onmessage = (ev) => {
      try {
        const envelope = JSON.parse(ev.data) as Envelope;
        for (const cb of envCbs) cb(envelope);
      } catch (e) {
        console.warn("prosopon-glass: dropping malformed envelope", e);
      }
    };
    ws.onclose = () => {
      ws = null;
      if (closed) return;
      setState("reconnecting");
      const delay = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)] ?? 5000;
      attempt += 1;
      setTimeout(connect, delay);
    };
    ws.onerror = () => ws?.close();
  }

  connect();

  return {
    onEnvelope(cb) {
      envCbs.add(cb);
      return () => {
        envCbs.delete(cb);
      };
    },
    onState(cb) {
      stateCbs.add(cb);
      cb(state);
      return () => {
        stateCbs.delete(cb);
      };
    },
    send(frame) {
      if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(frame));
    },
    close() {
      closed = true;
      ws?.close();
      setState("closed");
    },
  };
}
