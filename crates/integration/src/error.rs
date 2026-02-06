use voca_core::error::CoreError;

#[derive(thiserror::Error, Debug)]
pub enum IntegrationError {
    #[error("Template error: {0}")]
    Template(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Storage error: {0}")]
    Storage(#[from] CoreError),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for IntegrationError {
    fn from(e: std::io::Error) -> Self {
        IntegrationError::Io(e.to_string())
    }
}

impl From<tera::Error> for IntegrationError {
    fn from(e: tera::Error) -> Self {
        IntegrationError::Template(e.to_string())
    }
}

impl From<serde_json::Error> for IntegrationError {
    fn from(e: serde_json::Error) -> Self {
        IntegrationError::Serialization(e.to_string())
    }
}
