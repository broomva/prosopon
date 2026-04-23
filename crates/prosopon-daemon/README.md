# prosopon-daemon

Shared HTTP + WebSocket + SSE transport for the Prosopon ecosystem.

Every compositor (glass, text-web, field-shader) registers a
`SurfaceBundle` (a named `include_dir::Dir` of static assets). Every
emitter (arcan, lago, vigil) publishes `prosopon_protocol::Envelope`
values into the shared `EnvelopeFanout`. Browsers connect to one
endpoint — `/ws` (bidirectional) or `/events` (SSE) — and the daemon
fans every envelope out to every subscriber.

## Routes

| Route                  | Purpose                                 |
|------------------------|-----------------------------------------|
| `GET /`                | Surface bundle's `index.html` (or a minimal fallback). |
| `GET /assets/{*path}`  | Static assets from the surface bundle.  |
| `GET /ws`              | WebSocket envelope stream.              |
| `GET /events`          | SSE envelope stream.                    |
| `GET /schema/scene`    | JSON schema for `prosopon_core::Scene`. |
| `GET /schema/event`    | JSON schema for `prosopon_core::ProsoponEvent`. |

## Usage

```rust,ignore
use prosopon_daemon::{DaemonConfig, DaemonServer};

let server = DaemonServer::bind(DaemonConfig {
    addr: "127.0.0.1:4321".parse()?,
    surface: None, // or Some(glass_surface())
}).await?;
server.serve().await?;
```

See `docs/rfcs/0002-compositor-contract.md` and the BRO-768 plan.
