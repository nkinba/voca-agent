use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    RSS,
    Manual,
    Youtube,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub content: String,
    pub source: SourceType,
    pub published_at: DateTime<Utc>,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    pub word: String,
    pub definition: String,
    pub context_sentence: String,
    pub source_url: String,
}
