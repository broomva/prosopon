/**
 * @broomva/prosopon — TypeScript bindings for the Prosopon display server.
 *
 * Wire-compatible with `prosopon-protocol` v1 (as implemented in
 * `core/prosopon/crates/prosopon-protocol`).
 */

// IR types — re-exported from the facade over auto-generated output.
export * from "./types";

// Wire layer (envelope + codec)
export {
  type Envelope,
  type Codec,
  type Hello,
  PROTOCOL_VERSION,
  makeEnvelope,
  encode,
  decode,
  ProtocolVersionMismatch,
} from "./codec";

// Pure reducer
export { applyEvent, readSignal } from "./apply-event";

// Browser client (WS primary, SSE fallback)
export {
  ProsoponClient,
  type ProsoponClientOptions,
  type ProsoponClientState,
  type ProsoponTransport,
  type EnvelopeHandler,
  type ErrorHandler,
  type StateHandler,
} from "./client";

// Server-side session emitter
export { ProsoponSession, type ProsoponSessionOptions } from "./session";

// ID helpers
export {
  type NodeId,
  type SceneId,
  type ActionId,
  type StreamId,
  type Topic,
  newNodeId,
  newSceneId,
  newActionId,
  newStreamId,
  topic,
  topicStartsWith,
} from "./ids";
