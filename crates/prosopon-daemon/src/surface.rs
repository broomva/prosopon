/// A named asset bundle a compositor registers with the daemon.
pub struct SurfaceBundle {
    pub name: &'static str,
    pub assets: Option<&'static include_dir::Dir<'static>>,
}
