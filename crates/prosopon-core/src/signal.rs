//! Reactive signal primitives — bindings from live topics to IR slots.
//!
//! A **signal** is a live value keyed by a `Topic`. A **binding** tells a compositor
//! "when this topic changes, update that part of the node." This is the reactivity
//! substrate on which all "living text" and field-visualization effects compose.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Topic;
use crate::ids::NodeId;
use crate::value::Value;

/// A reference to a live value. Supports nested paths for structured signals:
/// `SignalRef { topic: "plexus.state", path: ["quorum", "ratio"] }` addresses
/// the `ratio` field inside the `quorum` object inside the `plexus.state` signal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SignalRef {
    pub topic: Topic,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,
}

impl SignalRef {
    #[must_use]
    pub fn topic(topic: impl Into<Topic>) -> Self {
        Self {
            topic: topic.into(),
            path: Vec::new(),
        }
    }

    #[must_use]
    pub fn at(mut self, segment: impl Into<String>) -> Self {
        self.path.push(segment.into());
        self
    }
}

impl From<Topic> for SignalRef {
    fn from(topic: Topic) -> Self {
        Self::topic(topic)
    }
}

/// A concrete signal value pushed onto the bus at a given instant.
///
/// Compositors MAY cache last-known values to render bindings when a signal
/// has not yet emitted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SignalValue {
    Scalar(Value),
    TimeSeries(Vec<TimePoint>),
    Vector(Vec<f32>),
    Event { payload: Value },
}

impl SignalValue {
    /// Returns a preview string suitable for logging.
    #[must_use]
    pub fn preview(&self) -> String {
        match self {
            Self::Scalar(v) => v.to_string(),
            Self::TimeSeries(ts) => format!("timeseries[{}]", ts.len()),
            Self::Vector(v) => format!("vec[{}]", v.len()),
            Self::Event { .. } => "event".to_string(),
        }
    }
}

/// Single point in a time-series signal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TimePoint {
    pub t: chrono::DateTime<chrono::Utc>,
    pub v: Value,
}

/// Binds a live signal to a rendering slot on a node.
///
/// The compositor is responsible for subscribing to the signal and applying
/// the (optional) transform before updating the target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Binding {
    pub source: SignalRef,
    pub target: BindTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
}

/// What part of a node a binding updates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BindTarget {
    /// Update a named attribute on the node (e.g. `"pct"`, `"label"`).
    Attr { key: String },
    /// Update a named slot inside the node's `Intent` payload (e.g. `"progress.pct"`).
    IntentSlot { path: String },
    /// Replace the content of a child node entirely.
    ChildContent { id: NodeId },
}

/// Optional transformation applied to the signal value before binding.
///
/// Transforms are intentionally limited to pure, deterministic functions so
/// compositors can execute them safely without a scripting runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Transform {
    /// Pass the value through unchanged.
    Identity,
    /// Format the value into a template string. `{}` is replaced with the value.
    Format { template: String },
    /// Clamp a numeric value into `[min, max]`.
    Clamp { min: f64, max: f64 },
    /// Round a numeric value to `decimals` places.
    Round { decimals: u8 },
    /// Multiply by 100 and append `"%"`.
    Percent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signalref_path_builder() {
        let s = SignalRef::topic(Topic::new("plexus.state"))
            .at("quorum")
            .at("ratio");
        assert_eq!(s.topic.as_str(), "plexus.state");
        assert_eq!(s.path, vec!["quorum".to_string(), "ratio".to_string()]);
    }

    #[test]
    fn signal_value_preview() {
        assert_eq!(SignalValue::Scalar(serde_json::json!(42)).preview(), "42");
        assert_eq!(SignalValue::Vector(vec![1.0, 2.0]).preview(), "vec[2]");
    }

    #[test]
    fn binding_serializes_with_tag() {
        let b = Binding {
            source: SignalRef::topic(Topic::new("arcan.load")),
            target: BindTarget::Attr { key: "pct".into() },
            transform: Some(Transform::Percent),
        };
        let json = serde_json::to_value(&b).unwrap();
        assert_eq!(json["target"]["kind"], "attr");
        assert_eq!(json["transform"]["kind"], "percent");
    }
}
