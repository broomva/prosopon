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
