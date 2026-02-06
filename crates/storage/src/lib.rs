use async_trait::async_trait;
use sqlx::SqlitePool;
use voca_core::error::CoreError;
use voca_core::model::{Article, Vocabulary};
use voca_core::port::StoragePort;

const CREATE_ARTICLES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS articles (
    url TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    published_at DATETIME NOT NULL,
    collected_at DATETIME NOT NULL
)
"#;

const CREATE_VOCABULARIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS vocabularies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL,
    definition TEXT NOT NULL,
    context_sentence TEXT NOT NULL,
    source_url TEXT NOT NULL,
    FOREIGN KEY (source_url) REFERENCES articles(url)
)
"#;

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(db_url: &str) -> Result<Self, CoreError> {
        let pool = SqlitePool::connect(db_url)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        sqlx::query(CREATE_ARTICLES_TABLE)
            .execute(&pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        sqlx::query(CREATE_VOCABULARIES_TABLE)
            .execute(&pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StoragePort for SqliteStorage {
    async fn exists(&self, url: &str) -> Result<bool, CoreError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles WHERE url = ?")
            .bind(url)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(count.0 > 0)
    }

    async fn save_article(&self, article: &Article) -> Result<(), CoreError> {
        let source = format!("{:?}", article.source);

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO articles (url, title, content, source, published_at, collected_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&article.url)
        .bind(&article.title)
        .bind(&article.content)
        .bind(&source)
        .bind(article.published_at)
        .bind(article.collected_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            INSERT INTO vocabularies (word, definition, context_sentence, source_url)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&vocab.word)
        .bind(&vocab.definition)
        .bind(&vocab.context_sentence)
        .bind(&vocab.source_url)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_all_vocab(&self) -> Result<Vec<Vocabulary>, CoreError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT word, definition, context_sentence, source_url FROM vocabularies",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(word, definition, context_sentence, source_url)| Vocabulary {
                word,
                definition,
                context_sentence,
                source_url,
            })
            .collect())
    }

    async fn search_vocab(&self, query: &str) -> Result<Vec<Vocabulary>, CoreError> {
        let pattern = format!("%{}%", query);
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT word, definition, context_sentence, source_url
            FROM vocabularies
            WHERE word LIKE ? OR definition LIKE ?
            "#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(word, definition, context_sentence, source_url)| Vocabulary {
                word,
                definition,
                context_sentence,
                source_url,
            })
            .collect())
    }

    async fn get_today_vocab(&self) -> Result<Vec<Vocabulary>, CoreError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT v.word, v.definition, v.context_sentence, v.source_url
            FROM vocabularies v
            JOIN articles a ON v.source_url = a.url
            WHERE date(a.collected_at) = date('now')
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(word, definition, context_sentence, source_url)| Vocabulary {
                word,
                definition,
                context_sentence,
                source_url,
            })
            .collect())
    }

    async fn get_random_vocab(&self) -> Result<Option<Vocabulary>, CoreError> {
        let row: Option<(String, String, String, String)> = sqlx::query_as(
            "SELECT word, definition, context_sentence, source_url FROM vocabularies ORDER BY RANDOM() LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(row.map(|(word, definition, context_sentence, source_url)| Vocabulary {
            word,
            definition,
            context_sentence,
            source_url,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use voca_core::model::SourceType;

    #[tokio::test]
    async fn test_article_crud() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        let article = Article {
            url: "https://example.com/test".to_string(),
            title: "Test Article".to_string(),
            content: "This is test content.".to_string(),
            source: SourceType::RSS,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };

        assert!(!storage
            .exists(&article.url)
            .await
            .expect("exists check failed"));

        storage
            .save_article(&article)
            .await
            .expect("save_article failed");

        assert!(storage
            .exists(&article.url)
            .await
            .expect("exists check failed"));
    }

    #[tokio::test]
    async fn test_vocabulary_crud() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        let article = Article {
            url: "https://example.com/vocab-test".to_string(),
            title: "Vocab Test Article".to_string(),
            content: "Content with vocabulary.".to_string(),
            source: SourceType::Manual,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };

        storage
            .save_article(&article)
            .await
            .expect("save_article failed");

        let vocab = Vocabulary {
            word: "vocabulary".to_string(),
            definition: "A collection of words".to_string(),
            context_sentence: "Content with vocabulary.".to_string(),
            source_url: article.url.clone(),
        };

        storage.save_vocab(&vocab).await.expect("save_vocab failed");
    }

    #[tokio::test]
    async fn test_duplicate_article_ignored() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        let article = Article {
            url: "https://example.com/duplicate".to_string(),
            title: "Original Title".to_string(),
            content: "Original content.".to_string(),
            source: SourceType::Youtube,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };

        storage
            .save_article(&article)
            .await
            .expect("first save failed");

        let duplicate = Article {
            url: "https://example.com/duplicate".to_string(),
            title: "Different Title".to_string(),
            content: "Different content.".to_string(),
            source: SourceType::RSS,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };

        storage
            .save_article(&duplicate)
            .await
            .expect("duplicate save should not error");
    }

    #[tokio::test]
    async fn test_get_all_vocab() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        let article = Article {
            url: "https://example.com/test".to_string(),
            title: "Test".to_string(),
            content: "Test content.".to_string(),
            source: SourceType::RSS,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };
        storage.save_article(&article).await.unwrap();

        let vocab1 = Vocabulary {
            word: "test".to_string(),
            definition: "A trial".to_string(),
            context_sentence: "This is a test.".to_string(),
            source_url: article.url.clone(),
        };
        let vocab2 = Vocabulary {
            word: "example".to_string(),
            definition: "A sample".to_string(),
            context_sentence: "This is an example.".to_string(),
            source_url: article.url.clone(),
        };

        storage.save_vocab(&vocab1).await.unwrap();
        storage.save_vocab(&vocab2).await.unwrap();

        let all = storage.get_all_vocab().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_search_vocab() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        let article = Article {
            url: "https://example.com/search".to_string(),
            title: "Search Test".to_string(),
            content: "Content.".to_string(),
            source: SourceType::RSS,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };
        storage.save_article(&article).await.unwrap();

        let vocab = Vocabulary {
            word: "serendipity".to_string(),
            definition: "Finding good things by chance".to_string(),
            context_sentence: "It was serendipity.".to_string(),
            source_url: article.url.clone(),
        };
        storage.save_vocab(&vocab).await.unwrap();

        let results = storage.search_vocab("serendip").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "serendipity");

        let no_results = storage.search_vocab("xyz").await.unwrap();
        assert!(no_results.is_empty());
    }

    #[tokio::test]
    async fn test_get_random_vocab() {
        let storage = SqliteStorage::new("sqlite::memory:")
            .await
            .expect("Failed to create storage");

        // Empty DB should return None
        let none = storage.get_random_vocab().await.unwrap();
        assert!(none.is_none());

        let article = Article {
            url: "https://example.com/random".to_string(),
            title: "Random Test".to_string(),
            content: "Content.".to_string(),
            source: SourceType::RSS,
            published_at: Utc::now(),
            collected_at: Utc::now(),
        };
        storage.save_article(&article).await.unwrap();

        let vocab = Vocabulary {
            word: "random".to_string(),
            definition: "By chance".to_string(),
            context_sentence: "Random selection.".to_string(),
            source_url: article.url.clone(),
        };
        storage.save_vocab(&vocab).await.unwrap();

        let some = storage.get_random_vocab().await.unwrap();
        assert!(some.is_some());
        assert_eq!(some.unwrap().word, "random");
    }
}
