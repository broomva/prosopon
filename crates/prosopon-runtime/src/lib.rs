//! # prosopon-runtime
//!
//! The reactive runtime: a **signal bus**, a **compositor registry**, and a
//! **scene store** that applies incoming events to an in-memory scene.
//!
//! ## Mental model (Flutter-inspired)
//!
//! `prosopon-core` defines the *Widget* layer (the IR agents emit). This crate owns
//! the *Element* layer — the identity/diff substrate that maintains a compositor's
//! in-memory scene and pushes it out through backends. Actual pixel/text/audio
//! production is the *RenderObject* layer and lives in `prosopon-compositor-*`.
//!
//! ## What lives here
//!
//! - [`SignalBus`] — publish/subscribe over `Topic`s, last-value cached.
//! - [`SceneStore`] — applies [`ProsoponEvent`] to a local [`Scene`] and fans out
//!   notifications.
//! - [`Compositor`] — the backend trait. Implementations live in sibling crates.
//! - [`Runtime`] — glue: owns a `SceneStore` + `SignalBus` + a set of compositors.

#![forbid(unsafe_code)]

pub mod bus;
pub mod compositor;
pub mod store;

pub use bus::{SignalBus, SignalSubscriber};
pub use compositor::{Capabilities, Compositor, CompositorError, CompositorId};
pub use store::{SceneStore, SceneStoreError, StoreEvent};

use std::sync::Arc;
use tokio::sync::Mutex;

use prosopon_core::{ProsoponEvent, Scene, SignalValue, Topic};

/// Top-level runtime: a scene store wired to a signal bus and a list of compositors.
pub struct Runtime {
    store: Arc<Mutex<SceneStore>>,
    bus: Arc<SignalBus>,
    compositors: Arc<Mutex<Vec<Box<dyn Compositor>>>>,
}

impl Runtime {
    /// Create a fresh runtime with an empty scene and no compositors.
    #[must_use]
    pub fn new(initial: Scene) -> Self {
        Self {
            store: Arc::new(Mutex::new(SceneStore::new(initial))),
            bus: Arc::new(SignalBus::new()),
            compositors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a compositor. Ownership moves to the runtime.
    pub async fn register_compositor(&self, c: Box<dyn Compositor>) {
        let mut guard = self.compositors.lock().await;
        guard.push(c);
    }

    /// Clone an `Arc` to the signal bus for agent-side publishing.
    #[must_use]
    pub fn bus(&self) -> Arc<SignalBus> {
        Arc::clone(&self.bus)
    }

    /// Submit an event into the runtime. Updates the scene, fans out to
    /// registered compositors, and updates the signal cache when applicable.
    ///
    /// # Errors
    /// Returns `RuntimeError::Compositor` if any compositor fails.
    pub async fn submit(&self, event: ProsoponEvent) -> Result<(), RuntimeError> {
        if let ProsoponEvent::SignalChanged { topic, value, .. } = &event {
            self.bus.publish(topic.clone(), value.clone()).await;
        }
        {
            let mut store = self.store.lock().await;
            store.apply(event.clone())?;
        }
        let mut compositors = self.compositors.lock().await;
        for c in compositors.iter_mut() {
            c.apply(&event)
                .map_err(|e| RuntimeError::Compositor(c.id(), e))?;
        }
        Ok(())
    }

    /// Read-only access to the current scene.
    pub async fn snapshot(&self) -> Scene {
        self.store.lock().await.scene().clone()
    }

    /// Inject a signal value without wrapping it in a `ProsoponEvent`.
    ///
    /// Useful for publishing high-frequency updates that compositors may down-sample.
    pub async fn publish_signal(&self, topic: Topic, value: SignalValue) {
        self.bus.publish(topic, value).await;
    }
}

/// Top-level runtime error type.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("scene store error: {0}")]
    Store(#[from] SceneStoreError),

    #[error("compositor `{0}` failed: {1}")]
    Compositor(CompositorId, CompositorError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node};

    fn scene() -> Scene {
        Scene::new(Node::new(Intent::Prose {
            text: "init".into(),
        }))
    }

    #[tokio::test]
    async fn runtime_applies_reset() {
        let rt = Runtime::new(scene());
        let new_scene = Scene::new(Node::new(Intent::Prose {
            text: "reset".into(),
        }));
        rt.submit(ProsoponEvent::SceneReset {
            scene: new_scene.clone(),
        })
        .await
        .unwrap();
        let snap = rt.snapshot().await;
        assert_eq!(snap.id, new_scene.id);
    }
}
