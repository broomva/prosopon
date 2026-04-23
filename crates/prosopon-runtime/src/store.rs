//! Scene store — applies incoming events to an in-memory [`Scene`].
//!
//! The store is the Element-layer analogue: it owns identity, applies patches, and
//! can emit diff summaries for compositors that want to avoid full re-renders.

use thiserror::Error;

use prosopon_core::{NodeId, ProsoponEvent, Scene};

/// Errors produced when applying events to the scene.
#[derive(Debug, Error, PartialEq)]
pub enum SceneStoreError {
    #[error("parent node `{0}` not found")]
    ParentNotFound(NodeId),

    #[error("node `{0}` not found")]
    NodeNotFound(NodeId),
}

/// The in-memory scene store.
#[derive(Debug)]
pub struct SceneStore {
    scene: Scene,
}

impl SceneStore {
    /// Create a store wrapping `initial`.
    #[must_use]
    pub fn new(initial: Scene) -> Self {
        Self { scene: initial }
    }

    /// Borrow the current scene.
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Apply an event to the scene. Returns a summary of what changed.
    ///
    /// # Errors
    /// Returns [`SceneStoreError`] when referenced nodes are missing.
    pub fn apply(&mut self, event: ProsoponEvent) -> Result<StoreEvent, SceneStoreError> {
        match event {
            ProsoponEvent::SceneReset { scene } => {
                let prev = std::mem::replace(&mut self.scene, scene);
                Ok(StoreEvent::Reset { previous: Box::new(prev) })
            }
            ProsoponEvent::NodeAdded { parent, node } => {
                let parent_node = self
                    .scene
                    .root
                    .find_mut(&parent)
                    .ok_or_else(|| SceneStoreError::ParentNotFound(parent.clone()))?;
                let id = node.id.clone();
                parent_node.children.push(node);
                Ok(StoreEvent::Added { parent, id })
            }
            ProsoponEvent::NodeUpdated { id, patch } => {
                let node = self
                    .scene
                    .root
                    .find_mut(&id)
                    .ok_or_else(|| SceneStoreError::NodeNotFound(id.clone()))?;
                patch.apply_to(node);
                Ok(StoreEvent::Updated { id })
            }
            ProsoponEvent::NodeRemoved { id } => {
                if remove_node(&mut self.scene.root, &id) {
                    Ok(StoreEvent::Removed { id })
                } else {
                    Err(SceneStoreError::NodeNotFound(id))
                }
            }
            ProsoponEvent::SignalChanged { topic, value, .. } => {
                self.scene.signals.insert(topic.clone(), value);
                Ok(StoreEvent::SignalUpdated { topic })
            }
            // Stream chunks and action events are pass-through — compositors handle.
            ProsoponEvent::StreamChunk { .. }
            | ProsoponEvent::ActionEmitted { .. }
            | ProsoponEvent::Heartbeat { .. } => Ok(StoreEvent::Passthrough),
            // Forwards-compatible fallback: new `ProsoponEvent` variants added upstream
            // are acknowledged as a passthrough rather than rejected — the store can
            // remain on an older minor version without breaking agents.
            _ => Ok(StoreEvent::Passthrough),
        }
    }
}

/// Summary of what an apply() did.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreEvent {
    /// The scene was wholesale replaced. `previous` is boxed to keep the
    /// enum tag-word small; `StoreEvent::Reset` is rare.
    Reset { previous: Box<Scene> },
    Added { parent: NodeId, id: NodeId },
    Updated { id: NodeId },
    Removed { id: NodeId },
    SignalUpdated { topic: prosopon_core::Topic },
    Passthrough,
}

fn remove_node(root: &mut prosopon_core::Node, id: &NodeId) -> bool {
    if let Some(idx) = root.children.iter().position(|c| &c.id == id) {
        root.children.remove(idx);
        return true;
    }
    for c in &mut root.children {
        if remove_node(c, id) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use prosopon_core::{Intent, Node, NodePatch};

    fn scene() -> Scene {
        Scene::new(
            Node::new(Intent::Section {
                title: None,
                collapsible: false,
            })
            .with_id("root"),
        )
    }

    #[test]
    fn applies_node_added() {
        let mut store = SceneStore::new(scene());
        let child = Node::new(Intent::Prose { text: "hi".into() }).with_id("c1");
        let res = store
            .apply(ProsoponEvent::NodeAdded {
                parent: NodeId::from_raw("root"),
                node: child,
            })
            .unwrap();
        assert!(matches!(res, StoreEvent::Added { .. }));
        assert_eq!(store.scene().root.children.len(), 1);
    }

    #[test]
    fn patches_update_attrs() {
        let mut store = SceneStore::new(scene());
        store
            .apply(ProsoponEvent::NodeAdded {
                parent: NodeId::from_raw("root"),
                node: Node::new(Intent::Prose { text: "hi".into() }).with_id("c1"),
            })
            .unwrap();

        let patch = NodePatch {
            intent: Some(Intent::Prose { text: "updated".into() }),
            ..Default::default()
        };
        store
            .apply(ProsoponEvent::NodeUpdated {
                id: NodeId::from_raw("c1"),
                patch,
            })
            .unwrap();

        let child = store.scene().root.find(&NodeId::from_raw("c1")).unwrap();
        assert_eq!(
            child.intent,
            Intent::Prose {
                text: "updated".into()
            }
        );
    }

    #[test]
    fn missing_parent_errors() {
        let mut store = SceneStore::new(scene());
        let res = store.apply(ProsoponEvent::NodeAdded {
            parent: NodeId::from_raw("nope"),
            node: Node::new(Intent::Prose { text: "x".into() }),
        });
        assert!(matches!(res, Err(SceneStoreError::ParentNotFound(_))));
    }
}
