//! Error types for the Prosopon core crate.

use thiserror::Error;

use crate::ids::NodeId;

/// Errors produced by core IR operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("invalid intent payload: {0}")]
    InvalidIntent(String),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("schema violation: {0}")]
    SchemaViolation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
