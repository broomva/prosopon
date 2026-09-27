/**
 * Wire codec — mirrors `prosopon-protocol::Envelope` + `Codec`.
 *
 * Protocol version is pinned to 1. Bump in lockstep with `prosopon-protocol`.
 */

import type { ProsoponEvent } from "./types";

/** Current wire version — MUST equal `prosopon_protocol::PROTOCOL_VERSION`. */
export const PROTOCOL_VERSION = 1 as const;

/**
 * One envelope — wraps a single `ProsoponEvent` with session metadata.
 * Deliberately structurally identical to the Rust `Envelope` so the same JSON
 * round-trips between implementations losslessly.
 */
export interface Envelope {
  version: number;
  session_id: string;
  seq: number;
  ts: string; // RFC3339 / ISO8601 — Rust emits chrono::DateTime<Utc> in this format
  event: ProsoponEvent;
}

export interface NewEnvelopeArgs {
  session_id: string;
  seq: number;
  event: ProsoponEvent;
  ts?: string;
}

export function makeEnvelope(args: NewEnvelopeArgs): Envelope {
  return {
    version: PROTOCOL_VERSION,
    session_id: args.session_id,
    seq: args.seq,
    ts: args.ts ?? new Date().toISOString(),
    event: args.event,
  };
}

export type Codec = "json" | "jsonl";

export function encode(codec: Codec, envelope: Envelope): string {
  const json = JSON.stringify(envelope);
  return codec === "jsonl" ? `${json}\n` : json;
}

/**
 * Decode a single envelope. For `jsonl`, tolerates trailing whitespace /
 * newlines.
 */
export function decode(codec: Codec, payload: string): Envelope {
  const trimmed = codec === "jsonl" ? payload.replace(/\s+$/g, "") : payload;
  const parsed = JSON.parse(trimmed) as Envelope;
  if (parsed.version !== PROTOCOL_VERSION) {
    throw new ProtocolVersionMismatch(PROTOCOL_VERSION, parsed.version);
  }
  return parsed;
}

export class ProtocolVersionMismatch extends Error {
  constructor(
    public readonly expected: number,
    public readonly actual: number,
  ) {
    super(`protocol version mismatch: expected ${expected}, got ${actual}`);
    this.name = "ProtocolVersionMismatch";
  }
}

/** `Hello` handshake frame — used by agents and compositors to declare role + capabilities. */
export interface Hello {
  max_version: number;
  agent: string;
  role: "agent" | "compositor";
  capabilities: {
    surfaces?: string[];
    codecs?: Codec[];
    supports_signal_push?: boolean;
    supports_streaming?: boolean;
  };
}
