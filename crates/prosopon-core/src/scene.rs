//! `Scene` — the top-level unit of rendering.
//!
//! A scene is a self-contained "what should the user see right now" — rooted at a
//! single `Node`, paired with a cache of last-known signal values, and carrying
//! compositor hints that describe *how much surface* is available.

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ids::{SceneId, Topic};
use crate::node::Node;
use crate::signal::SignalValue;

/// A complete snapshot of "what to render."
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Scene {
    pub id: SceneId,
    pub root: Node,
    /// Last-known value for each referenced signal topic. Compositors that support
    /// push-based updates MAY subscribe and ignore this cache; compositors that only
    /// pull SHOULD treat this as the authoritative initial value.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub signals: IndexMap<Topic, SignalValue>,
    #[serde(default)]
    pub hints: SceneHints,
}

impl Scene {
    /// Create a scene wrapping a single root node with default hints.
    #[must_use]
    pub fn new(root: Node) -> Self {
        Self {
            id: SceneId::new(),
            root,
            signals: IndexMap::new(),
            hints: SceneHints::default(),
        }
    }

    #[must_use]
    pub fn with_hints(mut self, hints: SceneHints) -> Self {
        self.hints = hints;
        self
    }

    /// Insert or update a signal's last-known value.
    pub fn set_signal(&mut self, topic: impl Into<Topic>, value: SignalValue) {
        self.signals.insert(topic.into(), value);
    }
}

/// Compositor-steering hints carried with every scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct SceneHints {
    /// Ordered preferred surfaces, most preferred first. A compositor MAY skip a scene
    /// if its surface is not listed; compositors SHOULD respect the first match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preferred_surfaces: Vec<SurfaceKind>,
    /// Overall intent profile — tells compositors whether to bias dense+technical or
    /// sparse+ambient presentations.
    #[serde(default)]
    pub intent_profile: IntentProfile,
    /// BCP-47 locale (e.g. `"en-US"`, `"es-CO"`). Compositors localize formatting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Visual density preference.
    #[serde(default)]
    pub density: Density,
    /// Rough viewport budget — hints for truncation/layout. `None` = unbounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
}

/// The shape of surface a compositor renders onto.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceKind {
    /// Plain ANSI / text terminal or plain-text document.
    Text,
    /// 2D raster or vector canvas (browser DOM, native 2D).
    TwoD,
    /// 3D world (WebGPU, native 3D engines).
    ThreeD,
    /// GPU shader program — agents become field/material parameters.
    Shader,
    /// Audio playback surface (speech, ambient music, sonification).
    Audio,
    /// Spatial computing (Vision Pro / Quest / AR).
    Spatial,
    /// Haptic / tactile feedback.
    Tactile,
}

/// Overall presentation bias for a scene.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum IntentProfile {
    /// Default — balanced density and ambience.
    #[default]
    Balanced,
    /// Dense technical output — maximize information per unit surface.
    DenseTechnical,
    /// Ambient monitor — minimize intrusion, prefer peripheral rendering.
    AmbientMonitor,
    /// Cinematic — long-form narrative, one idea per frame.
    Cinematic,
    /// Conversational — paced for human back-and-forth.
    Conversational,
}

/// Visual density preference. Compositors interpret this per-surface.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

/// Rough viewport budget. Compositors with bounded surfaces (terminals, small screens)
/// SHOULD respect these; surfaces without a natural viewport (audio, spatial) MAY ignore.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Viewport {
    pub cols: u32,
    pub rows: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Intent;

    #[test]
    fn scene_roundtrips_json() {
        let s = Scene::new(Node::new(Intent::Prose { text: "hi".into() }));
        let json = serde_json::to_string(&s).unwrap();
        let back: Scene = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, s.id);
    }

    #[test]
    fn hints_default_preserved() {
        let h = SceneHints::default();
        assert!(matches!(h.density, Density::Comfortable));
        assert!(matches!(h.intent_profile, IntentProfile::Balanced));
    }
}
