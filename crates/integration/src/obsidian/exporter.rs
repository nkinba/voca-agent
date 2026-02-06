use std::path::{Path, PathBuf};

use chrono::Local;
use tera::{Context, Tera};
use tracing::info;
use voca_core::model::Vocabulary;

use crate::error::IntegrationError;

const VOCABULARY_TEMPLATE: &str = r#"---
tag: #toefl #voca
date: {{ today }}
source: {{ article_url }}
---
# {{ word }}
**Definition:** {{ definition }}

> {{ context_sentence }}

[YouGlish로 발음 듣기](https://youglish.com/pronounce/{{ word }}/english?)
"#;

pub struct MarkdownExporter {
    output_path: PathBuf,
    tera: Tera,
}

impl MarkdownExporter {
    pub fn new(obsidian_path: &Path) -> Result<Self, IntegrationError> {
        let mut tera = Tera::default();
        tera.add_raw_template("vocabulary.md", VOCABULARY_TEMPLATE)?;

        Ok(Self {
            output_path: obsidian_path.to_path_buf(),
            tera,
        })
    }

    pub fn export(&self, vocab: &Vocabulary) -> Result<PathBuf, IntegrationError> {
        let mut context = Context::new();
        context.insert("today", &Local::now().format("%Y-%m-%d").to_string());
        context.insert("article_url", &vocab.source_url);
        context.insert("word", &vocab.word);
        context.insert("definition", &vocab.definition);
        context.insert("context_sentence", &vocab.context_sentence);

        let content = self.tera.render("vocabulary.md", &context)?;

        // Sanitize filename (remove special characters)
        let safe_word = vocab
            .word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        let filename = format!("{}.md", safe_word);
        let file_path = self.output_path.join(&filename);

        std::fs::write(&file_path, content)?;

        info!(word = %vocab.word, path = %file_path.display(), "Exported vocabulary to Obsidian");

        Ok(file_path)
    }

    pub fn export_batch(&self, vocabs: &[Vocabulary]) -> Result<Vec<PathBuf>, IntegrationError> {
        let mut paths = Vec::with_capacity(vocabs.len());
        for vocab in vocabs {
            let path = self.export(vocab)?;
            paths.push(path);
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_export_vocabulary() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MarkdownExporter::new(temp_dir.path()).unwrap();

        let vocab = Vocabulary {
            word: "serendipity".to_string(),
            definition: "The occurrence of events by chance in a happy way".to_string(),
            context_sentence: "It was pure serendipity that we met.".to_string(),
            source_url: "https://example.com/article".to_string(),
        };

        let path = exporter.export(&vocab).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "serendipity.md");

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# serendipity"));
        assert!(content.contains("**Definition:** The occurrence of events"));
        assert!(content.contains("> It was pure serendipity"));
        assert!(content.contains("youglish.com/pronounce/serendipity"));
        assert!(content.contains("tag: #toefl #voca"));
    }

    #[test]
    fn test_export_batch() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MarkdownExporter::new(temp_dir.path()).unwrap();

        let vocabs = vec![
            Vocabulary {
                word: "ephemeral".to_string(),
                definition: "Lasting for a very short time".to_string(),
                context_sentence: "Fame is ephemeral.".to_string(),
                source_url: "https://example.com".to_string(),
            },
            Vocabulary {
                word: "ubiquitous".to_string(),
                definition: "Present everywhere".to_string(),
                context_sentence: "Smartphones are ubiquitous.".to_string(),
                source_url: "https://example.com".to_string(),
            },
        ];

        let paths = exporter.export_batch(&vocabs).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths[0].exists());
        assert!(paths[1].exists());
    }

    #[test]
    fn test_sanitize_filename() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MarkdownExporter::new(temp_dir.path()).unwrap();

        let vocab = Vocabulary {
            word: "test/word:with*special".to_string(),
            definition: "Test".to_string(),
            context_sentence: "Test.".to_string(),
            source_url: "https://example.com".to_string(),
        };

        let path = exporter.export(&vocab).unwrap();
        let filename = path.file_name().unwrap().to_str().unwrap();
        // Should not contain special characters
        assert!(!filename.contains('/'));
        assert!(!filename.contains(':'));
        assert!(!filename.contains('*'));
    }
}
