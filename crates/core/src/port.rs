use async_trait::async_trait;

use crate::error::CoreError;
use crate::model::{Article, Vocabulary};

#[async_trait]
pub trait FetcherPort: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Article, CoreError>;
}

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn exists(&self, url: &str) -> Result<bool, CoreError>;
    async fn save_article(&self, article: &Article) -> Result<(), CoreError>;
    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError>;

    // Query methods for integration crate
    async fn get_all_vocab(&self) -> Result<Vec<Vocabulary>, CoreError>;
    async fn search_vocab(&self, query: &str) -> Result<Vec<Vocabulary>, CoreError>;
    async fn get_today_vocab(&self) -> Result<Vec<Vocabulary>, CoreError>;
    async fn get_random_vocab(&self) -> Result<Option<Vocabulary>, CoreError>;
}

#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError>;
}
