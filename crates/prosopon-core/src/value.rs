//! Dynamic value type for IR payloads.
//!
//! We alias `serde_json::Value` instead of defining a bespoke type because:
//!   1. every compositor and wire codec already speaks JSON fluently
//!   2. agents emit values from LLM outputs that are JSON-native
//!   3. adding a custom type costs ergonomics without buying new expressiveness
//!
//! For strongly-typed payloads, define your own domain struct and `serde::Serialize` it
//! into this alias via `serde_json::to_value`.

pub use serde_json::Value;
pub use serde_json::json;
