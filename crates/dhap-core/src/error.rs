use thiserror::Error;

#[derive(Error, Debug)]
pub enum DhapError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization / Deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Project not found at path: {0}")]
    ProjectNotFound(String),

    #[error("Invalid schema version: expected {expected}, found {found}")]
    InvalidSchemaVersion { expected: u32, found: u32 },

    #[error(
        "Archive security violation: detected illegal path traversal attempting to access '{0}'"
    )]
    PathTraversal(String),

    #[error("Corrupt or invalid project archive: {0}")]
    CorruptArchive(String),

    #[error("Media processing error: {0}")]
    MediaError(String),

    #[error("Frame not found: {0}")]
    FrameNotFound(String),

    #[error("Step not found: {0}")]
    StepNotFound(String),

    #[error("Annotation error: {0}")]
    AnnotationError(String),

    #[error("Export preflight failed: {0}")]
    ExportPreflightFailed(String),

    #[error("Export execution error: {0}")]
    ExportError(String),

    #[error("AI provider error ({provider}): {message}")]
    AiError { provider: String, message: String },
}

pub type DhapResult<T> = Result<T, DhapError>;
