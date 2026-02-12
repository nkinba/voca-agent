use async_trait::async_trait;
use chrono::{DateTime, Utc};
use feed_rs::parser;
use scraper::{Html, Selector};

use spread_core::{Article, CoreError, FetcherPort, SourceType};

/// Feed item metadata (URL and title)
#[derive(Debug, Clone)]
pub struct FeedItem {
    pub url: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
}

pub struct RssFetcher {
    client: reqwest::Client,
}

impl RssFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch all items from a feed (RSS or Atom)
    pub async fn fetch_feed(&self, feed_url: &str) -> Result<Vec<FeedItem>, CoreError> {
        let response = self
            .client
            .get(feed_url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let feed = parser::parse(bytes.as_ref()).map_err(|e| CoreError::Parse(e.to_string()))?;

        let items: Vec<FeedItem> = feed
            .entries
            .iter()
            .filter_map(|entry| {
                let url = entry.links.first()?.href.clone();
                let title = entry
                    .title
                    .as_ref()
                    .map(|t| t.content.clone())
                    .unwrap_or_else(|| "Untitled".to_string());
                let published_at = entry.published.or(entry.updated).unwrap_or_else(Utc::now);

                Some(FeedItem {
                    url,
                    title,
                    published_at,
                })
            })
            .collect();

        Ok(items)
    }

    /// Fetch the body content of a URL and convert to plain text
    pub async fn fetch_body(&self, url: &str) -> Result<String, CoreError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // Parse HTML and extract text content
        let document = Html::parse_document(&html);

        // Try to extract main content (article, main, or body)
        let content = extract_main_content(&document);

        Ok(content)
    }
}

/// Extract main text content from HTML document
fn extract_main_content(document: &Html) -> String {
    // Try to find article or main content first
    let selectors = [
        "article",
        "main",
        "[role=\"main\"]",
        ".content",
        ".post-content",
        "body",
    ];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text: String = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                if !text.is_empty() {
                    return text;
                }
            }
        }
    }

    String::new()
}

impl Default for RssFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetcherPort for RssFetcher {
    async fn fetch(&self, url: &str) -> Result<Article, CoreError> {
        // 1. reqwest로 GET 요청
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        // 2. feed-rs로 파싱 (RSS, Atom, JSON Feed 자동 감지)
        let feed = parser::parse(bytes.as_ref()).map_err(|e| CoreError::Parse(e.to_string()))?;

        // 3. 가장 최신 Item 하나 추출
        let entry = feed
            .entries
            .first()
            .ok_or_else(|| CoreError::Parse("No items found in feed".to_string()))?;

        // 4. Entry -> Article 변환
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_else(|| "Untitled".to_string());

        let content = entry
            .summary
            .as_ref()
            .map(|s| s.content.clone())
            .or_else(|| entry.content.as_ref().and_then(|c| c.body.clone()))
            .unwrap_or_default();

        let link = entry
            .links
            .first()
            .map(|l| l.href.clone())
            .unwrap_or_else(|| url.to_string());

        let published_at = entry.published.or(entry.updated).unwrap_or_else(Utc::now);

        let collected_at = Utc::now();

        Ok(Article {
            url: link,
            title,
            content,
            source: SourceType::RSS,
            published_at,
            collected_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <description>A test RSS feed</description>
    <item>
      <title>Test Article</title>
      <link>https://example.com/article1</link>
      <description>This is a test article content.</description>
      <pubDate>Mon, 01 Jan 2024 12:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Older Article</title>
      <link>https://example.com/article2</link>
      <description>This is an older article.</description>
      <pubDate>Sun, 31 Dec 2023 12:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    const SAMPLE_ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Rust Blog</title>
  <link href="https://blog.rust-lang.org/" rel="alternate"/>
  <id>https://blog.rust-lang.org/</id>
  <updated>2024-01-15T00:00:00+00:00</updated>
  <entry>
    <title>Rust 1.75 Released</title>
    <link href="https://blog.rust-lang.org/2024/01/15/rust-1.75.html" rel="alternate"/>
    <id>https://blog.rust-lang.org/2024/01/15/rust-1.75.html</id>
    <published>2024-01-15T00:00:00+00:00</published>
    <updated>2024-01-15T00:00:00+00:00</updated>
    <summary>The Rust team has published a new version of Rust, 1.75.</summary>
  </entry>
  <entry>
    <title>Older Rust Post</title>
    <link href="https://blog.rust-lang.org/2023/12/01/old-post.html" rel="alternate"/>
    <id>https://blog.rust-lang.org/2023/12/01/old-post.html</id>
    <published>2023-12-01T00:00:00+00:00</published>
    <updated>2023-12-01T00:00:00+00:00</updated>
    <summary>An older blog post.</summary>
  </entry>
</feed>"#;

    #[test]
    fn test_parse_rss_feed() {
        let feed = parser::parse(SAMPLE_RSS.as_bytes()).unwrap();
        assert_eq!(feed.title.unwrap().content, "Test Feed");
        assert_eq!(feed.entries.len(), 2);

        let first_entry = feed.entries.first().unwrap();
        assert_eq!(first_entry.title.as_ref().unwrap().content, "Test Article");
        assert_eq!(
            first_entry.links.first().unwrap().href,
            "https://example.com/article1"
        );
    }

    #[test]
    fn test_parse_atom_feed() {
        let feed = parser::parse(SAMPLE_ATOM.as_bytes()).unwrap();
        assert_eq!(feed.title.unwrap().content, "Rust Blog");
        assert_eq!(feed.entries.len(), 2);

        let first_entry = feed.entries.first().unwrap();
        assert_eq!(
            first_entry.title.as_ref().unwrap().content,
            "Rust 1.75 Released"
        );
        assert_eq!(
            first_entry.links.first().unwrap().href,
            "https://blog.rust-lang.org/2024/01/15/rust-1.75.html"
        );
        assert!(first_entry.published.is_some());
    }

    #[test]
    fn test_parse_empty_rss() {
        let empty_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
    <link>https://example.com</link>
    <description>No items here</description>
  </channel>
</rss>"#;

        let feed = parser::parse(empty_rss.as_bytes()).unwrap();
        assert!(feed.entries.is_empty());
    }

    #[test]
    fn test_parse_invalid_xml() {
        let invalid_xml = "this is not valid xml";
        let result = parser::parse(invalid_xml.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_rfc2822_date_parsing() {
        use chrono::Datelike;

        let date_str = "Mon, 01 Jan 2024 12:00:00 +0000";
        let parsed = DateTime::parse_from_rfc2822(date_str);
        assert!(parsed.is_ok());
        let dt = parsed.unwrap().with_timezone(&Utc);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }
}
