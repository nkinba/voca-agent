#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parsing error: {0}")]
    Parse(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
