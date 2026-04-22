//! Stable, serializable identifiers for Prosopon entities.
//!
//! All IDs are strings with macro-generated constructors. We use strings instead of
//! raw `Uuid` so:
//!   - agents can mint human-readable IDs for debugging (`"tool-call-7"`)
//!   - IDs round-trip through JSON without custom deserializers
//!   - compositors can preserve IDs verbatim across surfaces

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Mint a new random v4 UUID-based identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4().to_string())
            }

            /// Wrap an existing string without validation.
            #[must_use]
            pub fn from_raw(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            /// Borrow the inner string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

define_id!(NodeId, "Unique identifier for a node in a scene graph.");
define_id!(SceneId, "Unique identifier for a scene.");
define_id!(
    ActionId,
    "Unique identifier for an interactive action slot."
);
define_id!(
    StreamId,
    "Unique identifier for a streaming output channel (tokens, audio, etc.)."
);

/// A symbolic name for a reactive signal topic.
///
/// Topics are dotted namespaces (e.g. `plexus.load`, `arcan.tool.exec`, `haima.balance`).
/// They are intentionally free-form so agents can introduce new topics without schema
/// changes; compositors route by prefix matching.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Topic(pub String);

impl Topic {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Split the topic into dot-separated segments.
    pub fn segments(&self) -> std::str::Split<'_, char> {
        self.0.split('.')
    }

    /// True if this topic is under the given namespace prefix (dot-aware).
    #[must_use]
    pub fn starts_with_namespace(&self, prefix: &str) -> bool {
        if self.0 == prefix {
            return true;
        }
        self.0.starts_with(&format!("{prefix}."))
    }
}

impl From<&str> for Topic {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_ids_are_unique_by_default() {
        let a = NodeId::new();
        let b = NodeId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn ids_roundtrip_json() {
        let id = NodeId::from_raw("tool-call-7");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"tool-call-7\"");
        let back: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn topic_namespace_matching() {
        let t = Topic::new("plexus.load.cpu");
        assert!(t.starts_with_namespace("plexus"));
        assert!(t.starts_with_namespace("plexus.load"));
        assert!(t.starts_with_namespace("plexus.load.cpu"));
        assert!(!t.starts_with_namespace("plex"));
        assert!(!t.starts_with_namespace("plexus.loader"));
    }

    #[test]
    fn topic_segments() {
        let t = Topic::new("arcan.tool.exec");
        let segs: Vec<_> = t.segments().collect();
        assert_eq!(segs, vec!["arcan", "tool", "exec"]);
    }
}
