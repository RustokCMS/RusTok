use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ScriptError {
    #[error("Compilation failed: {0}")]
    Compilation(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Script aborted: {0}")]
    Aborted(String),

    #[error("Timeout: script exceeded {limit_ms}ms")]
    Timeout { limit_ms: u64 },

    #[error("Operation limit exceeded: {limit} operations")]
    OperationLimit { limit: u64 },

    #[error("Script not found: {name}")]
    NotFound { name: String },

    #[error("Max call depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid trigger: {0}")]
    InvalidTrigger(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),
}

pub type ScriptResult<T> = Result<T, ScriptError>;
