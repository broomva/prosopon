# @broomva/prosopon

TypeScript bindings for [Prosopon](https://github.com/broomva/prosopon) — the Life Agent OS display server. Wire-compatible with `prosopon-protocol` v1.

## Install

```bash
npm install @broomva/prosopon
# or
bun add @broomva/prosopon
```

## What this package provides

| Module | Purpose |
|---|---|
| `.` (default) | Re-exports of IR types + runtime helpers |
| `./codec` | JSON/JSONL envelope encode/decode |
| `./apply-event` | Pure reducer: `applyEvent(scene, event) → scene` |
| `./client` | Browser client — WS primary, SSE fallback |
| `./session` | Server-side session emitter |
| `./ids` | ID factories (NodeId, SceneId, StreamId, Topic) |

## Type layering

```
src/
├── generated/
│   ├── scene.json         ← snapshot of prosopon-core's JSON schema (Scene)
│   ├── event.json         ← snapshot (ProsoponEvent)
│   └── types.ts           ← json-schema-to-typescript output (generated)
├── types.ts               ← re-exports + curated facade over generated types
├── codec.ts               ← Envelope + Codec (JSON, JSONL) matching prosopon-protocol
├── apply-event.ts         ← Pure reducer for all 8 ProsoponEvent variants
├── client.ts              ← WS primary, SSE fallback, reconnect policy
├── session.ts             ← Server emitter (compositor/daemon fanout source)
├── ids.ts                 ← ID factory helpers
└── index.ts
```

The `generated/` folder is produced by compiling the Rust source of truth:

```bash
# from repo root
cargo run -p prosopon-cli -- schema scene > packages/prosopon-ts/src/generated/scene.json
cargo run -p prosopon-cli -- schema event > packages/prosopon-ts/src/generated/event.json
bun run --cwd packages/prosopon-ts generate
```

`json-schema-to-typescript` produces strongly-typed declarations that mirror the Rust enums exactly (including the `snake_case` tag on `Intent` variants).

## Quickstart

### Consume envelopes on the client

```ts
import { ProsoponClient } from "@broomva/prosopon/client";
import { applyEvent } from "@broomva/prosopon/apply-event";
import type { Scene } from "@broomva/prosopon";

const client = new ProsoponClient({
  url: "ws://localhost:9233/ws",
  sseFallback: "/api/events",
});

let scene: Scene = initialScene();

client.onEnvelope((env) => {
  scene = applyEvent(scene, env.event);
  rerender(scene);
});

await client.connect();
```

### Emit envelopes from the server

```ts
import { ProsoponSession } from "@broomva/prosopon/session";

const session = new ProsoponSession({ sessionId: "sess-123" });

// Emit scene reset then stream node adds
yield session.emit({ type: "scene_reset", scene });
yield session.emit({
  type: "node_added",
  parent: "chat",
  node: {
    /* Intent::Prose, Intent::FileWrite, etc. */
  },
});
```

Every `emit` increments the envelope `seq` monotonically and stamps `ts`. Wire with any transport (SSE, WebSocket, HTTP stream).

### Decode raw wire frames

```ts
import { decode } from "@broomva/prosopon/codec";

const envelope = decode(wireString, { codec: "json" });
```

## Compatibility

| Package | IR schema | Protocol |
|---|---|---|
| `0.2.x` | `0.2.0` | `1` |
| `0.1.x` | `0.1.0` | `1` |

`0.2.0` added `Intent::FileRead` and `Intent::FileWrite` via [RFC-0004](https://github.com/broomva/prosopon/blob/main/docs/rfcs/0004-filesystem-intents.md). Additive — `0.1.x` consumers still work (forward-compat via `#[non_exhaustive]` on the Rust side + catch-all branches in the reducer).

## Development

This package lives in the [prosopon monorepo](https://github.com/broomva/prosopon). The same repo ships the Rust crates to crates.io.

```bash
# from repo root
bun install
bun run --cwd packages/prosopon-ts test       # 12/12
bun run --cwd packages/prosopon-ts typecheck
bun run --cwd packages/prosopon-ts build      # emit dist/
```

Or via Turborepo:

```bash
bun run build        # all packages
bun run typecheck
bun run test:js
```

## License

Apache-2.0
