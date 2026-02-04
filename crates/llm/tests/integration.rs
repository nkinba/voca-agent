//! Integration tests for voca-llm
//!
//! These tests require a valid OPENAI_API_KEY environment variable.
//! Run with: cargo test --package voca-llm --test integration -- --ignored

use voca_core::LlmPort;
use voca_llm::RigLlmEngine;

const SAMPLE_TEXT: &str = r#"
The phenomenon of cognitive dissonance, first articulated by Leon Festinger in 1957,
remains a quintessential concept in social psychology. When individuals encounter
information that contradicts their preexisting beliefs, they experience psychological
discomfort that compels them to reconcile the discrepancy. This amelioration can occur
through various mechanisms, including rationalization, avoidance, or genuine attitude change.
The ubiquitous nature of this phenomenon in everyday decision-making underscores its
significance in understanding human behavior.
"#;

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_extract_vocabulary_from_text() {
    let engine = RigLlmEngine::new().expect("Failed to create RigLlmEngine");

    let result = engine.extract(SAMPLE_TEXT).await;

    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    let vocabularies = result.unwrap();
    assert!(!vocabularies.is_empty(), "No vocabularies extracted");
    assert!(
        vocabularies.len() <= 5,
        "Too many vocabularies extracted: {}",
        vocabularies.len()
    );

    for vocab in &vocabularies {
        assert!(!vocab.word.is_empty(), "Empty word found");
        assert!(!vocab.definition.is_empty(), "Empty definition found");
        assert!(
            !vocab.context_sentence.is_empty(),
            "Empty context sentence found"
        );
        println!(
            "Word: {}\nDefinition: {}\nContext: {}\n",
            vocab.word, vocab.definition, vocab.context_sentence
        );
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_extract_with_simple_text() {
    let engine = RigLlmEngine::new().expect("Failed to create RigLlmEngine");

    let simple_text = "The ephemeral nature of fame often leads to existential contemplation.";

    let result = engine.extract(simple_text).await;
    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    let vocabularies = result.unwrap();
    println!("Extracted {} words from simple text", vocabularies.len());

    for vocab in &vocabularies {
        println!("  - {}: {}", vocab.word, vocab.definition);
    }
}
