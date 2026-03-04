use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("HTML parse error: {0}")]
    HtmlParse(String),

    #[error("CSS parse error: {0}")]
    CssParse(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("SRI verification failed (expected {expected}, got {got})")]
    SRI { expected: String, got: String },

    #[error("Security violation: {0}")]
    Security(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
