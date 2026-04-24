/**
 * Server-side emitter — `ProsoponSession`. Helper for agent code running on
 * Node / Bun / edge that needs to emit envelopes into a transport (SSE, WS,
 * prosopon-daemon fanout, etc.).
 *
 * The session owns (sessionId, seq). Every `emit()` mints a new envelope with
 * the next monotonic seq. Callers supply the event; we handle versioning,
 * timestamps, and encoding.
 */

import { type Envelope, type Codec, encode, makeEnvelope } from "./codec";
import type { ProsoponEvent } from "./types";

export interface ProsoponSessionOptions {
  /** Stable session identifier. Generate with crypto.randomUUID() if absent. */
  sessionId: string;
  /** Starting sequence number (default 1, matches Rust `seq` semantics). */
  startSeq?: number;
}

export class ProsoponSession {
  readonly sessionId: string;
  private seq: number;

  constructor(options: ProsoponSessionOptions) {
    this.sessionId = options.sessionId;
    this.seq = options.startSeq ?? 1;
  }

  /**
   * Build the next envelope wrapping `event`. Increments the session's
   * sequence counter. Does NOT send anything — callers pipe the returned
   * envelope into their transport of choice.
   */
  emit(event: ProsoponEvent): Envelope {
    const envelope = makeEnvelope({
      session_id: this.sessionId,
      seq: this.seq,
      event,
    });
    this.seq += 1;
    return envelope;
  }

  /**
   * Emit + encode in one step. Convenience for SSE/JSONL writers.
   * Returns the wire bytes ready to enqueue into a stream.
   */
  emitFrame(event: ProsoponEvent, codec: Codec = "jsonl"): string {
    return encode(codec, this.emit(event));
  }
}
