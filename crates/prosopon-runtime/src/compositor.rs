//! The compositor trait — the contract every rendering backend implements.
//!
//! This is where surfaces plug in. A compositor consumes [`ProsoponEvent`]s and
//! produces surface-specific output (ANSI text, 2D pixels, 3D meshes, shader
//! uniforms, audio samples, spatial mesh updates, haptic patterns, …).
//!
//! Implementations MUST be total over `Intent`: even unknown variants should produce
//! *some* output (e.g. a placeholder) rather than panic or skip silently.

use prosopon_core::{ProsoponEvent, SurfaceKind};
use thiserror::Error;

/// Stable identifier for a compositor instance — useful for multi-compositor setups
/// where errors need to attribute to the originating backend.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CompositorId(pub String);

impl CompositorId {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl std::fmt::Display for CompositorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A compositor — the "RenderObject layer" of the Prosopon stack.
///
/// Compositors are driven imperatively by the runtime: every event is delivered via
/// [`Compositor::apply`]. Compositors may buffer internally and flush on their own
/// cadence.
pub trait Compositor: Send + 'static {
    /// Identifier, for logs and error attribution.
    fn id(&self) -> CompositorId;

    /// Advertised capabilities.
    fn capabilities(&self) -> Capabilities;

    /// Receive an event. Errors MAY be recoverable; the runtime decides.
    ///
    /// # Errors
    /// Returns [`CompositorError`] on any rendering or encoding failure.
    fn apply(&mut self, event: &ProsoponEvent) -> Result<(), CompositorError>;

    /// Flush any buffered output. Default is a no-op for compositors that render
    /// inline; implementations that batch MUST override this.
    ///
    /// # Errors
    /// Returns [`CompositorError`] on flush failure.
    fn flush(&mut self) -> Result<(), CompositorError> {
        Ok(())
    }
}

/// Self-described compositor capabilities.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Capabilities {
    /// Surfaces this compositor renders to.
    pub surfaces: Vec<SurfaceKind>,
    /// Max sustained frame rate this compositor can push (None = effectively unlimited
    /// / event-driven).
    pub max_fps: Option<u32>,
    /// Whether the compositor subscribes to signal pushes (vs. polling the scene).
    pub supports_signal_push: bool,
    /// Whether streaming chunks render incrementally (e.g. token-by-token text).
    pub supports_streaming: bool,
}

/// Errors produced by compositor backends.
#[derive(Debug, Error)]
pub enum CompositorError {
    /// IO error from the underlying sink (stdout, file, socket).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The compositor does not know how to render this intent variant.
    #[error("unsupported intent: {0}")]
    UnsupportedIntent(String),

    /// Encoding/rendering failure — backend-specific.
    #[error("encoding error: {0}")]
    Encoding(String),

    /// Backend-specific error carrying a preformatted message. Use when no other
    /// variant fits; surface the real cause in the message.
    #[error("{0}")]
    Backend(String),
}
