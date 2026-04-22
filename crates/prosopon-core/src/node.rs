//! The `Node` tree — the spatial skeleton of the IR.
//!
//! A `Node` is the unit of composition: it carries an `Intent`, optional `Binding`s,
//! optional `ActionSlot`s, and arbitrary child nodes. Patches describe incremental
//! updates so compositors never have to re-render unchanged subtrees.

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::ActionSlot;
use crate::ids::NodeId;
use crate::intent::Intent;
use crate::lifecycle::Lifecycle;
use crate::signal::Binding;
use crate::value::Value;

/// A node in the Prosopon IR tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Node {
    pub id: NodeId,
    pub intent: Intent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<Binding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActionSlot>,
    /// Free-form attribute bag for hints the compositor MAY use. Well-known keys:
    /// `emphasis` (`"low" | "normal" | "high"`), `semantic_role` (e.g. `"error"`,
    /// `"success"`), `width_hint` (fraction 0..=1), `voice` (TTS voice id).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attrs: IndexMap<String, Value>,
    #[serde(default)]
    pub lifecycle: Lifecycle,
}

impl Node {
    /// Create a node with the given intent and a freshly-minted id.
    #[must_use]
    pub fn new(intent: Intent) -> Self {
        Self {
            id: NodeId::new(),
            intent,
            children: Vec::new(),
            bindings: Vec::new(),
            actions: Vec::new(),
            attrs: IndexMap::new(),
            lifecycle: Lifecycle::now(),
        }
    }

    /// Builder: assign an explicit id.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<NodeId>) -> Self {
        self.id = id.into();
        self
    }

    /// Builder: add a child node.
    #[must_use]
    pub fn child(mut self, n: Node) -> Self {
        self.children.push(n);
        self
    }

    /// Builder: replace all children.
    #[must_use]
    pub fn children(mut self, ns: impl IntoIterator<Item = Node>) -> Self {
        self.children = ns.into_iter().collect();
        self
    }

    /// Builder: add a binding.
    #[must_use]
    pub fn bind(mut self, b: Binding) -> Self {
        self.bindings.push(b);
        self
    }

    /// Builder: add an interactive action slot.
    #[must_use]
    pub fn action(mut self, a: ActionSlot) -> Self {
        self.actions.push(a);
        self
    }

    /// Builder: set an attribute.
    #[must_use]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }

    /// Builder: replace the lifecycle wholesale.
    #[must_use]
    pub fn lifecycle(mut self, lc: Lifecycle) -> Self {
        self.lifecycle = lc;
        self
    }

    /// Depth-first iterator over `(depth, &node)`.
    pub fn iter_depth_first(&self) -> impl Iterator<Item = (usize, &Node)> + '_ {
        let mut stack: Vec<(usize, &Node)> = vec![(0, self)];
        std::iter::from_fn(move || {
            let (d, n) = stack.pop()?;
            for c in n.children.iter().rev() {
                stack.push((d + 1, c));
            }
            Some((d, n))
        })
    }

    /// Find a node by id, recursively.
    #[must_use]
    pub fn find(&self, id: &NodeId) -> Option<&Node> {
        if &self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    /// Find a node by id (mutable).
    pub fn find_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        if &self.id == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }
}

/// An incremental update to a node — used by `ProsoponEvent::NodeUpdated`.
///
/// All fields are optional; unset fields leave existing values untouched.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct NodePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<Intent>,
    /// Attribute updates. `None` values remove the key.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attrs: IndexMap<String, Option<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildrenPatch>,
}

impl NodePatch {
    /// Apply this patch in-place to a node.
    pub fn apply_to(&self, node: &mut Node) {
        if let Some(i) = &self.intent {
            node.intent = i.clone();
        }
        for (k, v) in &self.attrs {
            match v {
                Some(v) => {
                    node.attrs.insert(k.clone(), v.clone());
                }
                None => {
                    node.attrs.shift_remove(k);
                }
            }
        }
        if let Some(lc) = &self.lifecycle {
            node.lifecycle = lc.clone();
        }
        if let Some(cp) = &self.children {
            cp.apply_to(&mut node.children);
        }
    }
}

/// How a patch modifies a node's children.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ChildrenPatch {
    /// Replace the entire children list.
    Replace { children: Vec<Node> },
    /// Append new children to the end.
    Append { children: Vec<Node> },
    /// Remove children by id.
    Remove { ids: Vec<NodeId> },
    /// Reorder children to match `order`. Any ids not present are dropped.
    Reorder { order: Vec<NodeId> },
}

impl ChildrenPatch {
    fn apply_to(&self, children: &mut Vec<Node>) {
        match self {
            Self::Replace { children: cs } => *children = cs.clone(),
            Self::Append { children: cs } => children.extend(cs.iter().cloned()),
            Self::Remove { ids } => children.retain(|c| !ids.contains(&c.id)),
            Self::Reorder { order } => {
                let mut map: IndexMap<NodeId, Node> =
                    children.drain(..).map(|c| (c.id.clone(), c)).collect();
                *children = order.iter().filter_map(|id| map.shift_remove(id)).collect();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Intent;

    fn prose(s: &str) -> Node {
        Node::new(Intent::Prose { text: s.into() })
    }

    #[test]
    fn builder_chain() {
        let n = prose("hello")
            .attr("emphasis", "high")
            .child(prose("child-a"))
            .child(prose("child-b"));
        assert_eq!(n.children.len(), 2);
        assert_eq!(n.attrs["emphasis"], "high");
    }

    #[test]
    fn depth_first_iteration() {
        let n = prose("root")
            .child(prose("a").child(prose("a1")))
            .child(prose("b"));
        let labels: Vec<String> = n
            .iter_depth_first()
            .map(|(d, node)| match &node.intent {
                Intent::Prose { text } => format!("{d}:{text}"),
                _ => String::new(),
            })
            .collect();
        assert_eq!(labels, vec!["0:root", "1:a", "2:a1", "1:b"]);
    }

    #[test]
    fn patch_updates_attrs() {
        let mut n = prose("x").attr("k1", "v1");
        let p = NodePatch {
            attrs: {
                let mut m = IndexMap::new();
                m.insert("k1".into(), Some(Value::from("v2")));
                m.insert("k2".into(), Some(Value::from("v2b")));
                m
            },
            ..NodePatch::default()
        };
        p.apply_to(&mut n);
        assert_eq!(n.attrs["k1"], "v2");
        assert_eq!(n.attrs["k2"], "v2b");
    }

    #[test]
    fn children_patch_reorder() {
        let mut cs = vec![
            prose("a").with_id("a"),
            prose("b").with_id("b"),
            prose("c").with_id("c"),
        ];
        let p = ChildrenPatch::Reorder {
            order: vec!["c".into(), "a".into()],
        };
        p.apply_to(&mut cs);
        assert_eq!(cs.len(), 2);
        assert_eq!(cs[0].id.as_str(), "c");
        assert_eq!(cs[1].id.as_str(), "a");
    }
}
