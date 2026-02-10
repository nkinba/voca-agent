use rand::seq::SliceRandom;
use serde::Serialize;
use thiserror::Error;
use tracing::{info, warn};
use voca_core::model::Vocabulary;

/// Number of words to select for daily notification
const DEFAULT_WORD_COUNT: usize = 3;

/// Telegram Bot API base URL
const TELEGRAM_API_BASE: &str = "https://api.telegram.org";

#[derive(Error, Debug)]
pub enum NotifyError {
    #[error("Telegram API error: {0}")]
    TelegramApi(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Configuration missing: {0}")]
    ConfigMissing(String),
}

/// Telegram message request payload
#[derive(Serialize)]
struct SendMessageRequest<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

/// Telegram API response
#[derive(serde::Deserialize)]
struct TelegramResponse {
    ok: bool,
    description: Option<String>,
}

/// Telegram client for sending messages
pub struct TelegramClient {
    client: reqwest::Client,
    bot_token: String,
    chat_id: String,
}

impl TelegramClient {
    /// Create a new Telegram client from environment variables
    ///
    /// Returns None if TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID are not set
    pub fn from_env() -> Option<Self> {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").ok()?;
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").ok()?;

        if bot_token.is_empty() || chat_id.is_empty() {
            return None;
        }

        Some(Self {
            client: reqwest::Client::new(),
            bot_token,
            chat_id,
        })
    }

    /// Create a new Telegram client with explicit credentials
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
            chat_id,
        }
    }

    /// Send a message via Telegram Bot API
    pub async fn send_message(&self, text: &str) -> Result<(), NotifyError> {
        let url = format!("{}/bot{}/sendMessage", TELEGRAM_API_BASE, self.bot_token);

        let request = SendMessageRequest {
            chat_id: &self.chat_id,
            text,
            parse_mode: "MarkdownV2",
        };

        let response = self.client.post(&url).json(&request).send().await?;

        let telegram_response: TelegramResponse = response.json().await?;

        if !telegram_response.ok {
            return Err(NotifyError::TelegramApi(
                telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(())
    }
}

/// Notifier for sending daily vocabulary notifications
pub struct Notifier {
    telegram: TelegramClient,
    word_count: usize,
}

impl Notifier {
    /// Create a new Notifier from environment variables
    ///
    /// Returns None if Telegram credentials are not configured
    pub fn from_env() -> Option<Self> {
        let telegram = TelegramClient::from_env()?;
        Some(Self {
            telegram,
            word_count: DEFAULT_WORD_COUNT,
        })
    }

    /// Create a new Notifier with explicit Telegram client
    pub fn new(telegram: TelegramClient) -> Self {
        Self {
            telegram,
            word_count: DEFAULT_WORD_COUNT,
        }
    }

    /// Set the number of words to include in notifications
    pub fn with_word_count(mut self, count: usize) -> Self {
        self.word_count = count;
        self
    }

    /// Select random words from a list
    ///
    /// Returns up to `word_count` words. If fewer words are available,
    /// returns all of them.
    pub fn select_words<'a>(&self, vocabularies: &'a [Vocabulary]) -> Vec<&'a Vocabulary> {
        if vocabularies.is_empty() {
            return Vec::new();
        }

        let count = self.word_count.min(vocabularies.len());
        let mut rng = rand::thread_rng();

        vocabularies.choose_multiple(&mut rng, count).collect()
    }

    /// Format vocabularies into a Telegram message
    ///
    /// Uses MarkdownV2 format for rich text display
    pub fn format_message(&self, vocabularies: &[&Vocabulary]) -> String {
        if vocabularies.is_empty() {
            return "📚 *Today's Vocabulary*\n\nNo words collected today\\!".to_string();
        }

        let mut message = String::from("📚 *Today's Vocabulary*\n\n");

        for (i, vocab) in vocabularies.iter().enumerate() {
            // Escape special characters for MarkdownV2
            let word = escape_markdown(&vocab.word);
            let definition = escape_markdown(&vocab.definition);
            let sentence = escape_markdown(&vocab.context_sentence);

            message.push_str(&format!(
                "{}\\. *{}*\n   📖 _{}_\n   > \"{}\"\n\n",
                i + 1,
                word,
                definition,
                sentence
            ));
        }

        message
    }

    /// Send vocabulary notification
    ///
    /// Fetches today's vocabulary, selects random words, and sends via Telegram
    pub async fn notify(&self, vocabularies: &[Vocabulary]) -> Result<NotifyResult, NotifyError> {
        if vocabularies.is_empty() {
            warn!("No vocabulary available for notification");
            return Ok(NotifyResult {
                words_sent: 0,
                skipped: true,
            });
        }

        let selected = self.select_words(vocabularies);
        let message = self.format_message(&selected);

        info!(
            word_count = selected.len(),
            "Sending vocabulary notification"
        );

        self.telegram.send_message(&message).await?;

        Ok(NotifyResult {
            words_sent: selected.len(),
            skipped: false,
        })
    }
}

/// Result of a notification attempt
#[derive(Debug)]
pub struct NotifyResult {
    /// Number of words included in the notification
    pub words_sent: usize,
    /// Whether the notification was skipped (e.g., no words available)
    pub skipped: bool,
}

/// Escape special characters for Telegram MarkdownV2 format
fn escape_markdown(text: &str) -> String {
    text.chars()
        .map(|c| {
            if matches!(
                c,
                '_' | '*'
                    | '['
                    | ']'
                    | '('
                    | ')'
                    | '~'
                    | '`'
                    | '>'
                    | '#'
                    | '+'
                    | '-'
                    | '='
                    | '|'
                    | '{'
                    | '}'
                    | '.'
                    | '!'
            ) {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_vocabulary() -> Vocabulary {
        Vocabulary {
            word: "ephemeral".to_string(),
            definition: "Lasting for a very short time.".to_string(),
            context_sentence: "Fashions are ephemeral, changing with every season.".to_string(),
            source_url: "https://example.com/article".to_string(),
        }
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("hello"), "hello");
        assert_eq!(escape_markdown("hello_world"), "hello\\_world");
        assert_eq!(escape_markdown("test*bold*"), "test\\*bold\\*");
        assert_eq!(escape_markdown("a.b.c"), "a\\.b\\.c");
        assert_eq!(escape_markdown("(test)"), "\\(test\\)");
    }

    #[test]
    fn test_select_words_empty() {
        let telegram = TelegramClient::new("token".to_string(), "chat".to_string());
        let notifier = Notifier::new(telegram);

        let words: Vec<Vocabulary> = vec![];
        let selected = notifier.select_words(&words);
        assert!(selected.is_empty());
    }

    #[test]
    fn test_select_words_fewer_than_count() {
        let telegram = TelegramClient::new("token".to_string(), "chat".to_string());
        let notifier = Notifier::new(telegram).with_word_count(5);

        let words = vec![sample_vocabulary(), sample_vocabulary()];
        let selected = notifier.select_words(&words);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_select_words_exact_count() {
        let telegram = TelegramClient::new("token".to_string(), "chat".to_string());
        let notifier = Notifier::new(telegram).with_word_count(3);

        let words = vec![
            sample_vocabulary(),
            sample_vocabulary(),
            sample_vocabulary(),
            sample_vocabulary(),
            sample_vocabulary(),
        ];
        let selected = notifier.select_words(&words);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn test_format_message_empty() {
        let telegram = TelegramClient::new("token".to_string(), "chat".to_string());
        let notifier = Notifier::new(telegram);

        let message = notifier.format_message(&[]);
        assert!(message.contains("No words collected today"));
    }

    #[test]
    fn test_format_message_with_words() {
        let telegram = TelegramClient::new("token".to_string(), "chat".to_string());
        let notifier = Notifier::new(telegram);

        let vocab = sample_vocabulary();
        let message = notifier.format_message(&[&vocab]);

        assert!(message.contains("Today's Vocabulary"));
        assert!(message.contains("ephemeral"));
        assert!(message.contains("Lasting for a very short time"));
    }

    #[test]
    fn test_telegram_client_from_env_missing() {
        // Clear env vars to ensure they're not set
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        std::env::remove_var("TELEGRAM_CHAT_ID");

        let client = TelegramClient::from_env();
        assert!(client.is_none());
    }

    #[test]
    fn test_notifier_from_env_missing() {
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        std::env::remove_var("TELEGRAM_CHAT_ID");

        let notifier = Notifier::from_env();
        assert!(notifier.is_none());
    }
}
