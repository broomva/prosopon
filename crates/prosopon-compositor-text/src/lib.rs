//! # prosopon-compositor-text
//!
//! Reference text compositor — renders the Prosopon IR to an ANSI-capable terminal
//! (or a plain string for testing). This is the lowest-common-denominator compositor:
//! every agent's output should be viewable through it.
//!
//! ## Design
//!
//! - **Streaming-first.** Stream chunks (token-by-token text, tool call progress)
//!   render incrementally.
//! - **Pretext-inspired.** Rendering is a pure function of (scene × viewport-width),
//!   so the same IR renders identically to any write target.
//! - **Signal-aware.** Bindings on `attr:"pct"` etc. are read from the scene's signal
//!   cache at render time.
//!
//! ## Usage
//!
//! ```no_run
//! use prosopon_compositor_text::{TextCompositor, TextTarget};
//! use prosopon_runtime::Compositor;
//! use prosopon_core::{Scene, Node, Intent, ProsoponEvent};
//!
//! let mut c = TextCompositor::new(TextTarget::stdout(), 80);
//! let scene = Scene::new(Node::new(Intent::Prose { text: "hi".into() }));
//! c.apply(&ProsoponEvent::SceneReset { scene }).unwrap();
//! c.flush().unwrap();
//! ```

#![forbid(unsafe_code)]

mod render;
mod target;

pub use render::{RenderOptions, render_scene};
pub use target::TextTarget;

use std::io::Write;

use prosopon_core::{ProsoponEvent, Scene, SurfaceKind};
use prosopon_runtime::{Capabilities, Compositor, CompositorError, CompositorId};

/// A text-surface compositor.
pub struct TextCompositor {
    id: CompositorId,
    target: TextTarget,
    width: u16,
    scene: Option<Scene>,
    options: RenderOptions,
}

impl TextCompositor {
    /// Create a compositor writing to `target` with `width` columns.
    #[must_use]
    pub fn new(target: TextTarget, width: u16) -> Self {
        Self {
            id: CompositorId::new("prosopon-compositor-text"),
            target,
            width,
            scene: None,
            options: RenderOptions::default(),
        }
    }

    /// Override default render options.
    #[must_use]
    pub fn with_options(mut self, options: RenderOptions) -> Self {
        self.options = options;
        self
    }

    /// Replace the identifier — useful when running multiple text compositors (e.g.
    /// one for stdout + one for a log sink).
    #[must_use]
    pub fn with_id(mut self, id: CompositorId) -> Self {
        self.id = id;
        self
    }

    /// Render the current scene to the target. Idempotent — safe to call anytime.
    ///
    /// # Errors
    /// Returns a [`CompositorError`] on io failure.
    pub fn render_current(&mut self) -> Result<(), CompositorError> {
        if let Some(scene) = &self.scene {
            let output = render_scene(scene, self.width, &self.options);
            self.target.write_all(output.as_bytes())?;
            self.target.write_all(b"\n")?;
            self.target.flush()?;
        }
        Ok(())
    }
}

impl Compositor for TextCompositor {
    fn id(&self) -> CompositorId {
        self.id.clone()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            surfaces: vec![SurfaceKind::Text],
            max_fps: Some(60),
            supports_signal_push: true,
            supports_streaming: true,
        }
    }

    fn apply(&mut self, event: &ProsoponEvent) -> Result<(), CompositorError> {
        match event {
            ProsoponEvent::SceneReset { scene } => {
                self.scene = Some(scene.clone());
                self.render_current()?;
            }
            ProsoponEvent::SignalChanged { topic, value, .. } => {
                if let Some(scene) = &mut self.scene {
                    scene.signals.insert(topic.clone(), value.clone());
                    // Re-render on signal change — signals with bindings may have changed.
                    self.render_current()?;
                }
            }
            ProsoponEvent::NodeAdded { .. }
            | ProsoponEvent::NodeUpdated { .. }
            | ProsoponEvent::NodeRemoved { .. } => {
                // The runtime's SceneStore applied this; we get the new scene via
                // SceneReset from the runtime, OR re-render from our cached scene.
                // For standalone use, a full render is acceptable.
                self.render_current()?;
            }
            ProsoponEvent::StreamChunk { id: _, chunk } => match &chunk.payload {
                prosopon_core::ChunkPayload::Text { text } => {
                    self.target.write_all(text.as_bytes())?;
                    if chunk.final_ {
                        self.target.write_all(b"\n")?;
                    }
                    self.target.flush()?;
                }
                prosopon_core::ChunkPayload::Json { .. }
                | prosopon_core::ChunkPayload::B64 { .. } => {
                    // Text compositor ignores non-text streams; other compositors
                    // (audio, binary) handle them.
                }
            },
            ProsoponEvent::ActionEmitted { .. } | ProsoponEvent::Heartbeat { .. } => {
                // No visible effect.
            }
            _ => {
                // Forwards-compatible: future event variants render as a no-op here.
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CompositorError> {
        self.target.flush().map_err(CompositorError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node};

    #[test]
    fn renders_prose_through_compositor() {
        let buf = TextTarget::in_memory();
        let mut c = TextCompositor::new(buf.clone(), 40);
        let scene = Scene::new(Node::new(Intent::Prose {
            text: "hello, prosopon".into(),
        }));
        c.apply(&ProsoponEvent::SceneReset { scene }).unwrap();
        let out = buf.captured();
        assert!(out.contains("hello, prosopon"));
    }

    #[test]
    fn stream_chunks_append() {
        let buf = TextTarget::in_memory();
        let mut c = TextCompositor::new(buf.clone(), 40);
        c.apply(&ProsoponEvent::SceneReset {
            scene: Scene::new(Node::new(Intent::Empty)),
        })
        .unwrap();
        let chunk = prosopon_core::StreamChunk {
            seq: 1,
            payload: prosopon_core::ChunkPayload::Text { text: "tok".into() },
            final_: false,
        };
        c.apply(&ProsoponEvent::StreamChunk {
            id: prosopon_core::StreamId::from_raw("s1"),
            chunk,
        })
        .unwrap();
        assert!(buf.captured().contains("tok"));
    }
}
