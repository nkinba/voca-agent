use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rss::Channel;
use scraper::{Html, Selector};
use std::io::BufReader;

use voca_core::{Article, CoreError, FetcherPort, SourceType};

/// RSS feed item metadata (URL and title)
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

    /// Fetch all items from an RSS feed
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

        let channel = Channel::read_from(BufReader::new(bytes.as_ref()))
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let items: Vec<FeedItem> = channel
            .items()
            .iter()
            .filter_map(|item| {
                let url = item.link()?.to_string();
                let title = item.title().unwrap_or("Untitled").to_string();
                let published_at = item
                    .pub_date()
                    .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

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
    let selectors = ["article", "main", "[role=\"main\"]", ".content", ".post-content", "body"];

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

        // 2. 응답 본문을 rss::Channel::read_from으로 파싱
        let channel = Channel::read_from(BufReader::new(bytes.as_ref()))
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        // 3. 가장 최신 Item 하나 추출
        let item = channel
            .items()
            .first()
            .ok_or_else(|| CoreError::Parse("No items found in RSS feed".to_string()))?;

        // 4. Item -> Article 변환
        let title = item
            .title()
            .unwrap_or("Untitled")
            .to_string();

        let content = item
            .description()
            .or_else(|| item.content())
            .unwrap_or("")
            .to_string();

        let link = item
            .link()
            .unwrap_or(url)
            .to_string();

        // pub_date를 RFC2822로 파싱
        let published_at = item
            .pub_date()
            .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

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

    #[test]
    fn test_parse_rss_channel() {
        let channel = Channel::read_from(BufReader::new(SAMPLE_RSS.as_bytes())).unwrap();
        assert_eq!(channel.title(), "Test Feed");
        assert_eq!(channel.items().len(), 2);

        let first_item = channel.items().first().unwrap();
        assert_eq!(first_item.title(), Some("Test Article"));
        assert_eq!(first_item.link(), Some("https://example.com/article1"));
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

        let channel = Channel::read_from(BufReader::new(empty_rss.as_bytes())).unwrap();
        assert!(channel.items().is_empty());
    }

    #[test]
    fn test_parse_invalid_xml() {
        let invalid_xml = "this is not valid xml";
        let result = Channel::read_from(BufReader::new(invalid_xml.as_bytes()));
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
