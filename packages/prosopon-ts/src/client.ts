/**
 * Browser-side client — subscribes to a Prosopon envelope stream over
 * WebSocket (primary) with SSE fallback for environments that can't
 * upgrade.
 *
 * The client owns transport management (connect, reconnect, close); it does
 * NOT fold envelopes into a Scene. Callers pipe into their state store via
 * `applyEvent()` from `./apply-event`.
 */

import { decode, type Envelope, ProtocolVersionMismatch } from "./codec";

export type ProsoponTransport = "ws" | "sse";

export interface ProsoponClientOptions {
  /** Base URL of the transport endpoint. For WS pass wss:/ws:; for SSE pass http(s):/events. */
  url: string;
  /** Which transport to try first. Default: `ws`. */
  preferred?: ProsoponTransport;
  /**
   * When true (default), if the preferred transport fails to open within
   * `fallbackAfterMs` or errors immediately, the other transport is tried.
   * Set false to disable automatic fallback.
   */
  autoFallback?: boolean;
  fallbackAfterMs?: number;
  /** Maximum reconnect attempts before giving up. Default: infinite. */
  maxReconnects?: number;
  /** Base backoff (ms) — actual backoff is exponential with jitter. Default 500. */
  reconnectBaseMs?: number;
}

export type EnvelopeHandler = (envelope: Envelope) => void;
export type ErrorHandler = (err: Error) => void;
export type StateHandler = (state: ProsoponClientState) => void;

export type ProsoponClientState =
  | "idle"
  | "connecting"
  | "open"
  | "reconnecting"
  | "closed"
  | "error";

export class ProsoponClient {
  private readonly opts: Required<ProsoponClientOptions>;
  private handlers: EnvelopeHandler[] = [];
  private errorHandlers: ErrorHandler[] = [];
  private stateHandlers: StateHandler[] = [];
  private ws: WebSocket | null = null;
  private sse: EventSource | null = null;
  private currentTransport: ProsoponTransport | null = null;
  private reconnects = 0;
  private state: ProsoponClientState = "idle";
  private closed = false;

  constructor(options: ProsoponClientOptions) {
    this.opts = {
      url: options.url,
      preferred: options.preferred ?? "ws",
      autoFallback: options.autoFallback ?? true,
      fallbackAfterMs: options.fallbackAfterMs ?? 3000,
      maxReconnects: options.maxReconnects ?? Number.POSITIVE_INFINITY,
      reconnectBaseMs: options.reconnectBaseMs ?? 500,
    };
  }

  onEnvelope(handler: EnvelopeHandler): () => void {
    this.handlers.push(handler);
    return () => {
      this.handlers = this.handlers.filter((h) => h !== handler);
    };
  }

  onError(handler: ErrorHandler): () => void {
    this.errorHandlers.push(handler);
    return () => {
      this.errorHandlers = this.errorHandlers.filter((h) => h !== handler);
    };
  }

  onState(handler: StateHandler): () => void {
    this.stateHandlers.push(handler);
    handler(this.state);
    return () => {
      this.stateHandlers = this.stateHandlers.filter((h) => h !== handler);
    };
  }

  connect(): void {
    this.closed = false;
    this.open(this.opts.preferred);
  }

  close(): void {
    this.closed = true;
    this.teardown();
    this.setState("closed");
  }

  // ------------------------------------------------------------------ private

  private open(transport: ProsoponTransport): void {
    if (this.closed) return;
    this.teardown();
    this.currentTransport = transport;
    this.setState(this.reconnects === 0 ? "connecting" : "reconnecting");

    if (transport === "ws") {
      this.openWs();
    } else {
      this.openSse();
    }
  }

  private openWs(): void {
    try {
      const ws = new WebSocket(this.toWsUrl(this.opts.url));
      this.ws = ws;
      let opened = false;

      const fallbackTimer = setTimeout(() => {
        if (!opened && this.opts.autoFallback) {
          ws.close();
          this.open("sse");
        }
      }, this.opts.fallbackAfterMs);

      ws.addEventListener("open", () => {
        opened = true;
        clearTimeout(fallbackTimer);
        this.reconnects = 0;
        this.setState("open");
      });

      ws.addEventListener("message", (ev) => {
        this.ingest(typeof ev.data === "string" ? ev.data : "");
      });

      ws.addEventListener("error", (ev) => {
        clearTimeout(fallbackTimer);
        this.emitError(new Error(`WebSocket error: ${String(ev)}`));
      });

      ws.addEventListener("close", () => {
        clearTimeout(fallbackTimer);
        if (!opened && this.opts.autoFallback) {
          this.open("sse");
        } else {
          this.scheduleReconnect();
        }
      });
    } catch (err) {
      if (this.opts.autoFallback) {
        this.open("sse");
      } else {
        this.emitError(err instanceof Error ? err : new Error(String(err)));
      }
    }
  }

  private openSse(): void {
    try {
      const sse = new EventSource(this.opts.url);
      this.sse = sse;

      sse.addEventListener("open", () => {
        this.reconnects = 0;
        this.setState("open");
      });

      // The Rust daemon emits envelopes under the `envelope` event name;
      // we also subscribe to the default `message` event for broader compat.
      const onMessage = (ev: MessageEvent): void => {
        this.ingest(ev.data);
      };
      sse.addEventListener("envelope", onMessage as EventListener);
      sse.addEventListener("message", onMessage as EventListener);

      sse.addEventListener("error", () => {
        // EventSource auto-reconnects; we just surface the state change.
        this.setState("reconnecting");
      });
    } catch (err) {
      this.emitError(err instanceof Error ? err : new Error(String(err)));
    }
  }

  private ingest(payload: string): void {
    if (!payload) return;
    try {
      const env = decode("json", payload);
      for (const h of this.handlers) h(env);
    } catch (err) {
      if (err instanceof ProtocolVersionMismatch) {
        this.emitError(err);
        this.close();
        return;
      }
      this.emitError(err instanceof Error ? err : new Error(String(err)));
    }
  }

  private scheduleReconnect(): void {
    if (this.closed) return;
    if (this.reconnects >= this.opts.maxReconnects) {
      this.setState("closed");
      return;
    }
    const delay = Math.min(
      30_000,
      this.opts.reconnectBaseMs * 2 ** this.reconnects,
    );
    const jitter = Math.random() * 250;
    this.reconnects += 1;
    setTimeout(() => {
      this.open(this.currentTransport ?? this.opts.preferred);
    }, delay + jitter);
  }

  private teardown(): void {
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        /* ignore */
      }
      this.ws = null;
    }
    if (this.sse) {
      try {
        this.sse.close();
      } catch {
        /* ignore */
      }
      this.sse = null;
    }
  }

  private setState(state: ProsoponClientState): void {
    this.state = state;
    for (const h of this.stateHandlers) h(state);
  }

  private emitError(err: Error): void {
    this.setState("error");
    for (const h of this.errorHandlers) h(err);
  }

  private toWsUrl(url: string): string {
    if (url.startsWith("http://")) return `ws://${url.slice(7)}`;
    if (url.startsWith("https://")) return `wss://${url.slice(8)}`;
    return url;
  }
}
