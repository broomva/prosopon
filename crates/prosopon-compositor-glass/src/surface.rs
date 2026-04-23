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
