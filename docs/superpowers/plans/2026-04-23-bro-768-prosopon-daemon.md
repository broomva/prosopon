# BRO-768 · prosopon-daemon Implementation Plan

**Goal:** Extract the HTTP / WebSocket / SSE / envelope-fanout primitives from `prosopon-compositor-glass` into a reusable `prosopon-daemon` crate. All existing behaviour preserved; glass becomes a thin layer that registers its asset bundle with a `DaemonServer`. Future crates (`prosopon-lago`, `prosopon-vigil`, `arcan-prosopon`) connect to the same daemon without each vendoring an axum server.

**Nature:** Pure refactor — no new features, no new tests expected, all 55 workspace + 32 TS tests must remain green.

**Architecture:**
- `prosopon-daemon` (new crate, lib + optional bin):
  - `EnvelopeFanout`, `EnvelopeReceiver`, `FanoutError` — moved verbatim from glass.
  - `DaemonServer { listener, fanout, surface }` — owns the axum router.
  - `SurfaceBundle { name, assets }` — a name + optional `&'static include_dir::Dir<'static>`. Bundle-less mode is valid (daemon-only, no static UI).
  - Routes: `/`, `/assets/{*path}`, `/ws`, `/events`, `/schema/{scene,event}` — unchanged from what Task 10 shipped.
- `prosopon-compositor-glass`:
  - Depends on `prosopon-daemon`.
  - Re-exports `EnvelopeFanout` for back-compat.
  - Deletes `src/fanout.rs` and `src/server.rs`. Adds `src/surface.rs` exposing `glass_surface() -> SurfaceBundle`.
  - `GlassServer` is removed — `prosopon-glass` CLI uses `DaemonServer` directly.

---

## Task 1: Scaffold `prosopon-daemon` + move fanout

**Files:**
- Create: `crates/prosopon-daemon/Cargo.toml`
- Create: `crates/prosopon-daemon/src/lib.rs`
- Create: `crates/prosopon-daemon/src/fanout.rs` (moved from glass)
- Modify: root `Cargo.toml` — add member + workspace dep entry.

- [ ] Add to workspace. Edit root `Cargo.toml` members list, insert `"crates/prosopon-daemon",` after `"crates/prosopon-compositor-glass"`. Under `[workspace.dependencies]` add:
  ```toml
  prosopon-daemon = { path = "crates/prosopon-daemon", version = "0.2.0" }
  ```

- [ ] Write `crates/prosopon-daemon/Cargo.toml`:
  ```toml
  [package]
  name = "prosopon-daemon"
  version = "0.2.0"
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true
  authors.workspace = true
  description = "Prosopon HTTP/WebSocket/SSE daemon — envelope fanout + axum server shared by every compositor"
  repository.workspace = true
  keywords = ["prosopon", "agents", "daemon", "websocket", "sse"]
  categories.workspace = true
  readme = "README.md"

  [lib]
  path = "src/lib.rs"

  [[bin]]
  name = "prosopon-daemon"
  path = "src/bin/prosopon-daemon.rs"

  [dependencies]
  prosopon-core = { workspace = true }
  prosopon-protocol = { workspace = true }

  axum = { workspace = true }
  tokio = { workspace = true }
  tokio-stream = { workspace = true }
  futures = { workspace = true }
  tower = { workspace = true }
  tower-http = { workspace = true }

  serde = { workspace = true }
  serde_json = { workspace = true }
  thiserror = { workspace = true }
  tracing = { workspace = true }
  tracing-subscriber = { workspace = true }

  include_dir = { workspace = true }
  mime_guess = { workspace = true }
  async-stream = "0.3"
  clap = { workspace = true }
  anyhow = { workspace = true }

  [dev-dependencies]
  tokio = { workspace = true }
  tokio-tungstenite = "0.26"
  pretty_assertions = { workspace = true }
  chrono = { workspace = true }
  ```

- [ ] Create `src/lib.rs`:
  ```rust
  //! # prosopon-daemon
  //!
  //! Reusable HTTP + WebSocket + SSE transport for the Prosopon ecosystem.
  //! Every compositor (glass, text-web, field-shader) registers an optional
  //! asset bundle; every emitter (arcan, lago, vigil) publishes envelopes
  //! into a shared [`EnvelopeFanout`]. The daemon is the single endpoint
  //! browsers connect to.
  //!
  //! See `docs/rfcs/0002-compositor-contract.md` and the BRO-768 plan.

  #![forbid(unsafe_code)]

  pub mod fanout;
  pub mod server;
  pub mod surface;

  pub use fanout::{EnvelopeFanout, EnvelopeReceiver, FanoutError};
  pub use server::{DaemonConfig, DaemonServer};
  pub use surface::SurfaceBundle;

  /// Version of this daemon crate.
  pub const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");
  ```

- [ ] Copy the contents of `crates/prosopon-compositor-glass/src/fanout.rs` verbatim to `crates/prosopon-daemon/src/fanout.rs`. (Do NOT delete the glass copy yet — Task 2 does that.)

- [ ] Create a stub `src/server.rs` + `src/surface.rs` + `src/bin/prosopon-daemon.rs` (just enough to compile):
  ```rust
  // src/server.rs
  pub struct DaemonConfig;
  pub struct DaemonServer;
  ```
  ```rust
  // src/surface.rs
  /// A named asset bundle a compositor registers with the daemon.
  pub struct SurfaceBundle {
      pub name: &'static str,
      pub assets: Option<&'static include_dir::Dir<'static>>,
  }
  ```
  ```rust
  // src/bin/prosopon-daemon.rs
  fn main() {
      eprintln!("prosopon-daemon — stub, see Task 2");
  }
  ```

- [ ] Write `crates/prosopon-daemon/README.md` (~20 lines describing purpose).

- [ ] `cargo check -p prosopon-daemon` — must compile. `cargo check --workspace` — must also compile (glass still has its own fanout, no conflict yet).

- [ ] Commit:
  ```
  feat(daemon): scaffold prosopon-daemon crate + move EnvelopeFanout (BRO-768)

  Empty server stubs; real extraction lands next. Fanout moved byte-for-byte
  from prosopon-compositor-glass so the diff is reviewable. Glass still
  uses its own copy at this commit — Task 2 re-wires.
  ```

---

## Task 2: Move server + wire glass to depend on daemon

**Files:**
- Modify: `crates/prosopon-daemon/src/server.rs` (full implementation)
- Modify: `crates/prosopon-daemon/src/surface.rs` (keep simple)
- Modify: `crates/prosopon-daemon/src/bin/prosopon-daemon.rs` (minimal binary)
- Delete: `crates/prosopon-compositor-glass/src/server.rs`
- Delete: `crates/prosopon-compositor-glass/src/fanout.rs`
- Create: `crates/prosopon-compositor-glass/src/surface.rs` (`glass_surface()`)
- Modify: `crates/prosopon-compositor-glass/src/lib.rs` (re-exports)
- Modify: `crates/prosopon-compositor-glass/src/compositor.rs` (import EnvelopeFanout from daemon)
- Modify: `crates/prosopon-compositor-glass/src/bin/prosopon-glass.rs` (use DaemonServer)
- Delete: `crates/prosopon-compositor-glass/src/assets.rs` (superseded by daemon's asset serving)
- Modify: `crates/prosopon-compositor-glass/Cargo.toml` (depend on daemon, drop axum-direct deps no longer needed)
- Modify: `crates/prosopon-compositor-glass/tests/server_handshake.rs` (adjust to use DaemonServer API)

### Server implementation

Replace `crates/prosopon-daemon/src/server.rs` with the server code previously in glass, generalized to accept an optional `SurfaceBundle`:

```rust
//! axum server exposing:
//!   GET  /                 — embedded index.html (from surface bundle, if any)
//!   GET  /assets/{*path}   — embedded static assets (from surface bundle)
//!   GET  /ws               — WebSocket, bidi envelope stream
//!   GET  /events           — SSE, one-way envelope stream
//!   GET  /schema/scene     — prosopon-core scene schema
//!   GET  /schema/event     — prosopon-core event schema
//!
//! Every compositor crate constructs a [`DaemonServer`] with its own
//! [`SurfaceBundle`] and calls [`DaemonServer::serve`]. Future enhancements
//! (multi-surface routing, auth, rate limiting) belong here.

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures::stream::Stream;
use include_dir::Dir;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::fanout::EnvelopeFanout;
use crate::surface::SurfaceBundle;

/// Daemon configuration.
pub struct DaemonConfig {
    pub addr: SocketAddr,
    /// Asset bundle to serve. `None` = no UI, WS/SSE + schema only.
    pub surface: Option<SurfaceBundle>,
}

#[derive(Clone)]
struct AppState {
    fanout: Arc<EnvelopeFanout>,
    assets: Option<&'static Dir<'static>>,
    surface_name: &'static str,
}

pub struct DaemonServer {
    listener: TcpListener,
    fanout: Arc<EnvelopeFanout>,
    assets: Option<&'static Dir<'static>>,
    surface_name: &'static str,
    local_addr: SocketAddr,
}

impl DaemonServer {
    /// Bind a listener. Returns immediately.
    ///
    /// # Errors
    /// Returns any I/O error encountered while binding.
    pub async fn bind(config: DaemonConfig) -> std::io::Result<Self> {
        let listener = TcpListener::bind(config.addr).await?;
        let local_addr = listener.local_addr()?;
        Ok(Self {
            listener,
            fanout: Arc::new(EnvelopeFanout::new()),
            assets: config.surface.as_ref().and_then(|s| s.assets),
            surface_name: config.surface.map(|s| s.name).unwrap_or("daemon"),
            local_addr,
        })
    }

    #[must_use]
    pub fn fanout(&self) -> EnvelopeFanout {
        (*self.fanout).clone()
    }

    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Serve until cancelled.
    ///
    /// # Errors
    /// Surfaces any I/O error from axum's serve loop.
    pub async fn serve(self) -> std::io::Result<()> {
        let state = AppState {
            fanout: self.fanout,
            assets: self.assets,
            surface_name: self.surface_name,
        };
        let router = Router::new()
            .route("/", get(index))
            .route("/assets/{*path}", get(asset))
            .route("/ws", get(ws_upgrade))
            .route("/events", get(sse_stream))
            .route("/schema/scene", get(schema_scene))
            .route("/schema/event", get(schema_event))
            .layer(TraceLayer::new_for_http())
            .with_state(state);
        axum::serve(self.listener, router).await
    }
}

const FALLBACK_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>prosopon-daemon</title></head>
<body><h1>prosopon-daemon</h1><p>No surface bundle registered. Connect a client to <code>/ws</code> or <code>/events</code>.</p></body></html>"#;

async fn index(State(state): State<AppState>) -> Response<Body> {
    match state.assets.and_then(|d| d.get_file("index.html")) {
        Some(file) => html_response(file.contents()),
        None => html_response(FALLBACK_HTML.as_bytes()),
    }
}

async fn asset(
    State(state): State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    let Some(dir) = state.assets else {
        return not_found();
    };
    match dir.get_file(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref())
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            let mut resp = Response::new(Body::from(file.contents()));
            *resp.headers_mut() = headers;
            resp
        }
        None => not_found(),
    }
}

fn html_response(bytes: &'static [u8]) -> Response<Body> {
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    resp
}

fn not_found() -> Response<Body> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::NOT_FOUND;
    resp
}

async fn ws_upgrade(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| ws_session(state, socket))
}

async fn ws_session(state: AppState, mut socket: WebSocket) {
    let mut rx = state.fanout.subscribe();
    loop {
        tokio::select! {
            maybe_env = rx.recv() => {
                match maybe_env {
                    Ok(envelope) => {
                        let frame = match serde_json::to_string(&envelope) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::warn!(target: "prosopon::daemon", surface = state.surface_name, "encode failed: {e}");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(frame.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(crate::fanout::FanoutError::Lagged(n)) => {
                        tracing::warn!(target: "prosopon::daemon", "client lagged {n}; closing");
                        return;
                    }
                    Err(_) => return,
                }
            }
            msg = socket.recv() => {
                match msg {
                    None | Some(Err(_)) => return,
                    Some(Ok(Message::Close(_))) => return,
                    Some(Ok(_)) => continue,
                }
            }
        }
    }
}

async fn sse_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let mut rx = state.fanout.subscribe();
    let stream = async_stream::stream! {
        while let Ok(envelope) = rx.recv().await {
            let data = serde_json::to_string(&envelope).unwrap_or_default();
            yield Ok(SseEvent::default().event("envelope").data(data));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn schema_scene() -> Response {
    let body = prosopon_core::scene_schema_json();
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

async fn schema_event() -> Response {
    let body = prosopon_core::event_schema_json();
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}
```

### Surface module

`crates/prosopon-daemon/src/surface.rs`:

```rust
//! A `SurfaceBundle` is a named static asset bundle (an `include_dir::Dir`)
//! that a compositor crate hands to the daemon. The daemon serves `/` and
//! `/assets/{*path}` from the bundle. When `assets` is `None`, the daemon
//! serves a minimal fallback page.

use include_dir::Dir;

/// Asset bundle + identifier.
pub struct SurfaceBundle {
    pub name: &'static str,
    pub assets: Option<&'static Dir<'static>>,
}
```

### Daemon binary

`crates/prosopon-daemon/src/bin/prosopon-daemon.rs`:

```rust
//! Standalone daemon binary. Starts a DaemonServer with no surface bundle —
//! useful for headless deployments where the UI lives elsewhere (e.g. a
//! browser-hosted bundle connecting over CORS). For a UI-attached daemon,
//! use a compositor-specific binary such as `prosopon-glass`.

use clap::{Parser, Subcommand};
use prosopon_daemon::{DaemonConfig, DaemonServer};

#[derive(Parser)]
#[command(name = "prosopon-daemon", about = "Prosopon daemon — HTTP/WS/SSE transport")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Serve {
        #[arg(long, default_value = "127.0.0.1:4321")]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve { addr } => {
            let config = DaemonConfig {
                addr: addr.parse()?,
                surface: None,
            };
            let server = DaemonServer::bind(config).await?;
            println!("prosopon-daemon serving at http://{}/", server.local_addr());
            server.serve().await?;
            Ok(())
        }
    }
}
```

### Glass rewiring

Modify `crates/prosopon-compositor-glass/Cargo.toml`:
- Add `prosopon-daemon = { workspace = true }`.
- Remove direct deps that are only used by the moved code: `axum`, `tower`, `tower-http`, `include_dir` (keep `include_dir` — glass STILL needs it for `include_dir!(...)` to embed the bundle), `mime_guess` (drop), `async-stream` (drop).
  - Actually, keep `axum` only if glass still imports axum types (it shouldn't after this refactor). Verify post-change.

Delete:
- `crates/prosopon-compositor-glass/src/server.rs`
- `crates/prosopon-compositor-glass/src/fanout.rs`
- `crates/prosopon-compositor-glass/src/assets.rs`

Create `crates/prosopon-compositor-glass/src/surface.rs`:

```rust
//! Glass asset bundle — the Preact web bundle embedded at Rust compile time.
//! Register with a [`prosopon_daemon::DaemonServer`] via [`glass_surface()`].

use include_dir::{Dir, include_dir};
use prosopon_daemon::SurfaceBundle;

static GLASS_WEB: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

#[must_use]
pub fn glass_surface() -> SurfaceBundle {
    SurfaceBundle {
        name: "glass",
        assets: Some(&GLASS_WEB),
    }
}
```

Update `crates/prosopon-compositor-glass/src/lib.rs`:

```rust
//! # prosopon-compositor-glass
//!
//! 2D web compositor for Prosopon — Arcan Glass-styled Preact bundle.
//!
//! Register with a [`prosopon_daemon::DaemonServer`]:
//!
//! ```no_run
//! use prosopon_compositor_glass::{GlassCompositor, glass_surface};
//! use prosopon_daemon::{DaemonConfig, DaemonServer};
//!
//! # async fn run() -> anyhow::Result<()> {
//! let server = DaemonServer::bind(DaemonConfig {
//!     addr: "127.0.0.1:4321".parse()?,
//!     surface: Some(glass_surface()),
//! }).await?;
//! let _compositor = GlassCompositor::new(server.fanout());
//! server.serve().await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod compositor;
pub mod surface;

pub use compositor::{GlassCompositor, GlassCompositorBuilder};
pub use surface::glass_surface;

// Back-compat re-exports so consumers don't break.
pub use prosopon_daemon::{EnvelopeFanout, EnvelopeReceiver, FanoutError};

pub const COMPOSITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
```

Update `crates/prosopon-compositor-glass/src/compositor.rs` — change the import:

```rust
use prosopon_daemon::EnvelopeFanout;
```

(Drop `use crate::fanout::EnvelopeFanout;`.)

Update `crates/prosopon-compositor-glass/src/bin/prosopon-glass.rs` — replace `GlassServer` with `DaemonServer`:

```rust
use clap::{Parser, Subcommand};
use prosopon_compositor_glass::{GlassCompositor, glass_surface};
use prosopon_core::ProsoponEvent;
use prosopon_daemon::{DaemonConfig, DaemonServer};
use prosopon_runtime::Compositor;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prosopon-glass", about = "Prosopon glass compositor server")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Serve {
        #[arg(long, default_value = "127.0.0.1:4321")]
        addr: String,
        #[arg(long)]
        fixture: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve { addr, fixture } => {
            let config = DaemonConfig {
                addr: addr.parse()?,
                surface: Some(glass_surface()),
            };
            let server = DaemonServer::bind(config).await?;
            println!("prosopon-glass serving at http://{}/", server.local_addr());
            let mut compositor = GlassCompositor::new(server.fanout());
            if let Some(path) = fixture {
                replay_fixture(&mut compositor, &path)?;
            }
            server.serve().await?;
            Ok(())
        }
    }
}

fn replay_fixture(c: &mut GlassCompositor, path: &std::path::Path) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        let event: ProsoponEvent = serde_json::from_str(line)
            .map_err(|e| anyhow::anyhow!("line {}: {e}", n + 1))?;
        c.apply(&event)?;
    }
    Ok(())
}
```

### Test update

Update `crates/prosopon-compositor-glass/tests/server_handshake.rs` — replace `GlassServer` / `GlassServerConfig` with `DaemonServer` / `DaemonConfig`:

```rust
//! Integration test: start a DaemonServer with glass surface, connect via WS,
//! send a SceneReset through the compositor, and assert it arrives on the wire.

use futures::StreamExt;
use prosopon_compositor_glass::{GlassCompositor, glass_surface};
use prosopon_core::{Intent, Node, ProsoponEvent, Scene};
use prosopon_daemon::{DaemonConfig, DaemonServer};
use prosopon_runtime::Compositor;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn ws_client_receives_envelopes() {
    let server = DaemonServer::bind(DaemonConfig {
        addr: "127.0.0.1:0".parse().unwrap(),
        surface: Some(glass_surface()),
    })
    .await
    .expect("bind succeeds");
    let url = format!("ws://{}/ws", server.local_addr());
    let mut compositor = GlassCompositor::new(server.fanout());
    let serve = tokio::spawn(server.serve());

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let (mut ws, _resp) = connect_async(&url).await.expect("ws connect");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let scene = Scene::new(Node::new(Intent::Prose { text: "hello".into() }));
    compositor
        .apply(&ProsoponEvent::SceneReset { scene })
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .expect("got a message in time")
        .expect("stream not closed")
        .expect("ws frame");
    let text = msg.into_text().unwrap().to_string();
    assert!(text.contains("\"scene_reset\""), "frame was: {text}");

    serve.abort();
}
```

### Verify

- `cargo check --workspace` — clean.
- `cargo test --workspace` — 55 tests pass (same count as before; 2 glass totality + 1 handshake reconfigured to DaemonServer all still present).
- `make smoke` — exit 0.
- `cargo run -p prosopon-compositor-glass --bin prosopon-glass -- serve --addr 127.0.0.1:4321` (manual, optional) — browser loads.

Add `prosopon-tungstenite` fallback note: if `cargo check` errors on unused axum/tower deps in glass Cargo.toml, remove them — they should be fully gone from glass after this task. Keep only `prosopon-core`, `prosopon-protocol`, `prosopon-runtime`, `prosopon-daemon`, `tokio`, `serde`, `serde_json`, `thiserror`, `tracing`, `include_dir`, `clap`, `anyhow`, `tracing-subscriber` for glass.

### Commit

```
refactor(daemon,glass): extract HTTP/WS/SSE server to prosopon-daemon (BRO-768)

GlassServer → DaemonServer. EnvelopeFanout moved to prosopon-daemon; glass
re-exports for back-compat. Glass now ships only a SurfaceBundle (glass_surface)
and the GlassCompositor; all transport lives in the daemon. Unlocks arcan-prosopon
(BRO-773), prosopon-lago (BRO-771), prosopon-vigil (BRO-772) — each will
construct a DaemonServer directly instead of embedding its own axum stack.

All 55 workspace tests + 32 TS tests remain green. No behaviour change.
```

---

## Task 3: Docs, CHANGELOG, PR

- [ ] Update `ARCHITECTURE.md` crate dependency graph: add `prosopon-daemon` between `prosopon-protocol` and `prosopon-compositor-glass`.

- [ ] Update `docs/surfaces/glass.md` to mention the daemon dependency.

- [ ] Prepend to `CHANGELOG.md`:
  ```
  ## [0.2.0-alpha.2] — 2026-04-23

  ### Added
  - `prosopon-daemon` crate — shared HTTP/WebSocket/SSE transport layer.
    Every compositor now registers a `SurfaceBundle` with a `DaemonServer`
    instead of vendoring its own axum stack. Unblocks `arcan-prosopon`
    (BRO-773), `prosopon-lago` (BRO-771), `prosopon-vigil` (BRO-772).

  ### Changed
  - `prosopon-compositor-glass` now depends on `prosopon-daemon` for
    `EnvelopeFanout`, `EnvelopeReceiver`, and the axum server. Re-exports
    preserved for back-compat (`use prosopon_compositor_glass::EnvelopeFanout`
    still works). `GlassServer` removed in favour of `DaemonServer`.
  ```

- [ ] Check BRO-768 in `PLANS.md`.

- [ ] `make smoke` / full test pass.

- [ ] Commit docs (second commit on this branch):
  ```
  docs(daemon): architecture + CHANGELOG 0.2.0-alpha.2
  ```

- [ ] Archive the plan:
  ```
  git add -f docs/superpowers/plans/2026-04-23-bro-768-prosopon-daemon.md
  git commit -m "docs(plans): archive BRO-768 implementation plan"
  ```

- [ ] Push and open PR:
  ```
  git push -u origin bro-768-prosopon-daemon
  gh pr create --base bro-767-glass-compositor \
    --title "BRO-768: prosopon-daemon — shared HTTP/WS/SSE transport" \
    --body "..."
  ```

  PR is stacked on top of BRO-767 — merge base is `bro-767-glass-compositor`.

- [ ] Post Linear comment on BRO-768 with PR URL.
