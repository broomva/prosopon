//! Embeds the compiled TypeScript bundle from `web/dist/` via `include_dir!`.
//! At runtime, HTTP GETs for any path inside the bundle are served with the
//! appropriate MIME type. Missing `web/dist/` (pre-`bun run build`) is tolerated
//! — the server returns a helpful HTML shell telling the operator to build.

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use include_dir::{Dir, include_dir};

static WEB: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

const FALLBACK_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>prosopon-glass</title></head>
<body><h1>prosopon-compositor-glass</h1>
<p>The web bundle is missing. From the crate root, run:</p>
<pre>cd web &amp;&amp; bun install &amp;&amp; bun run build</pre>
<p>Then restart this server.</p></body></html>"#;

pub fn serve_asset(path: &str) -> Response<Body> {
    let normalized = path.trim_start_matches('/');
    let normalized = if normalized.is_empty() {
        "index.html"
    } else {
        normalized
    };
    match WEB.get_file(normalized) {
        Some(file) => {
            let mime = mime_guess::from_path(normalized).first_or_octet_stream();
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref())
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            let body = Body::from(file.contents());
            let mut resp = Response::new(body);
            *resp.headers_mut() = headers;
            resp
        }
        None if normalized == "index.html" => {
            let mut resp = Response::new(Body::from(FALLBACK_HTML));
            resp.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            resp
        }
        None => {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
    }
}
