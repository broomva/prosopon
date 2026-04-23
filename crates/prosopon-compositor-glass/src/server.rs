//! axum server exposing:
//!   GET  /                 — embedded index.html
//!   GET  /assets/...       — embedded static assets
//!   GET  /ws               — WebSocket, bidi envelope stream
//!   GET  /events           — SSE, one-way JSONL envelope stream
//!   GET  /schema/scene     — prosopon-core scene schema (application/json)
//!   GET  /schema/event     — prosopon-core event schema
//!
//! The server is designed so the downstream `prosopon-daemon` (BRO-768) can
//! reuse it verbatim by constructing the router and adding its own middleware.

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::header;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures::stream::Stream;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::assets::serve_asset;
use crate::fanout::EnvelopeFanout;

/// Server configuration.
pub struct GlassServerConfig {
    pub addr: SocketAddr,
}

/// Runtime state shared by every request handler.
#[derive(Clone)]
struct AppState {
    fanout: Arc<EnvelopeFanout>,
}

/// Bound-but-not-yet-serving HTTP server.
pub struct GlassServer {
    listener: TcpListener,
    fanout: Arc<EnvelopeFanout>,
    local_addr: SocketAddr,
}

impl GlassServer {
    /// Bind a TCP listener. Returns immediately; the caller drives `serve()`.
    ///
    /// # Errors
    /// Returns the underlying `std::io::Error` if the listener cannot bind.
    pub async fn bind(config: GlassServerConfig) -> std::io::Result<Self> {
        let listener = TcpListener::bind(config.addr).await?;
        let local_addr = listener.local_addr()?;
        Ok(Self {
            listener,
            fanout: Arc::new(EnvelopeFanout::new()),
            local_addr,
        })
    }

    /// Clone a fanout handle for registering a compositor.
    #[must_use]
    pub fn fanout(&self) -> EnvelopeFanout {
        (*self.fanout).clone()
    }

    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Run the server until the task is cancelled.
    ///
    /// # Errors
    /// Returns any I/O error surfaced by axum's serve loop.
    pub async fn serve(self) -> std::io::Result<()> {
        let state = AppState {
            fanout: self.fanout,
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

async fn index() -> Response<Body> {
    serve_asset("index.html")
}

async fn asset(axum::extract::Path(path): axum::extract::Path<String>) -> Response<Body> {
    serve_asset(&path)
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
                                tracing::warn!(target: "prosopon::glass", "encode failed: {e}");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(frame.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(crate::fanout::FanoutError::Lagged(n)) => {
                        tracing::warn!(target: "prosopon::glass", "ws client lagged {n}; closing");
                        return;
                    }
                    Err(_) => return,
                }
            }
            msg = socket.recv() => {
                // Client→server: ActionEmitted envelopes. Not implemented in v0.2;
                // we ACK by ignoring. BRO-778 will wire these into the runtime.
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
