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
