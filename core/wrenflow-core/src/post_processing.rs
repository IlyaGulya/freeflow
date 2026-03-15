//! LLM post-processing — context-aware transcript cleanup

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::time::Duration;

use crate::http_client::GROQ_BASE_URL;

const POST_PROCESSING_TIMEOUT_SECS: u64 = 20;
pub const DEFAULT_MODEL: &str = "meta-llama/llama-4-scout-17b-16e-instruct";

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a dictation post-processor. You clean up raw speech-to-text output for typing.

CRITICAL: Output MUST be in the SAME language as RAW_TRANSCRIPTION. If input is Russian, output Russian. If input is English, output English. NEVER translate to another language.

Rules:
- Add punctuation, capitalization, and formatting.
- Remove filler words (um, uh, like, you know) unless they carry meaning.
- Fix misspellings using context and custom vocabulary — only correct words already spoken, never insert new ones.
- Preserve tone, intent, and word choice exactly. Never censor, rephrase, or omit anything including profanity and slang.

Respond with JSON: {"text": "cleaned text", "reasoning": "brief explanation of changes made"}
If the input is empty or only noise, respond: {"text": "", "reasoning": "explanation"}"#;

pub const DEFAULT_SYSTEM_PROMPT_DATE: &str = "2026-02-24";

#[derive(Debug, Error)]
pub enum PostProcessingError {
    #[error("Post-processing failed with status {0}: {1}")]
    RequestFailed(u16, String),
    #[error("Invalid post-processing response: {0}")]
    InvalidResponse(String),
    #[error("Post-processing timed out after {0}s")]
    TimedOut(u64),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result of a post-processing pass.
#[derive(Debug, Clone)]
pub struct PostProcessingResult {
    /// The cleaned-up transcript text.
    pub transcript: String,
    /// The full prompt that was sent (for display/debugging).
    pub prompt: String,
    /// Brief reasoning from the LLM about what was changed.
    pub reasoning: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    temperature: f64,
    response_format: ResponseFormat,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct PostProcessingJson {
    text: Option<String>,
    reasoning: Option<String>,
}

/// Parse the LLM's response content as `{"text": "...", "reasoning": "..."}`.
/// Falls back to treating the entire content as plain text if JSON parsing fails.
fn parse_response(content: &str) -> (String, String) {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return (String::new(), "Empty response from LLM".to_string());
    }

    // Attempt JSON parse
    if let Ok(parsed) = serde_json::from_str::<PostProcessingJson>(trimmed) {
        let text = parsed
            .text
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let reasoning = parsed
            .reasoning
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        return (text, reasoning);
    }

    // Fallback: plain text
    let mut result = trimmed.to_string();

    // Strip outer quotes if the LLM wrapped the response
    if result.starts_with('"') && result.ends_with('"') && result.len() > 1 {
        result = result[1..result.len() - 1].trim().to_string();
    }

    if result == "EMPTY" {
        return (String::new(), "LLM returned EMPTY sentinel".to_string());
    }

    (result, "LLM returned plain text (no JSON)".to_string())
}

/// Merge and deduplicate vocabulary terms from a raw comma/semicolon/newline-separated string.
fn merged_vocabulary_terms(raw: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    raw.split(|c| c == '\n' || c == ',' || c == ';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.to_lowercase()))
        .collect()
}

/// Format vocabulary terms as a comma-separated string.
fn normalized_vocabulary_text(terms: &[String]) -> String {
    let cleaned: Vec<&str> = terms
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    cleaned.join(", ")
}

/// Run LLM post-processing on a transcript.
///
/// # Parameters
/// - `client` — shared reqwest Client
/// - `api_key` — Groq API key
/// - `transcript` — raw speech-to-text output
/// - `context_summary` — brief description of the current screen/context
/// - `model` — LLM model identifier (defaults to `DEFAULT_MODEL`)
/// - `custom_vocabulary` — raw vocabulary hint string (newline/comma/semicolon separated)
/// - `custom_system_prompt` — override the default system prompt (empty = use default)
/// - `base_url` — API base URL
pub async fn post_process(
    client: &Client,
    api_key: &str,
    transcript: &str,
    context_summary: &str,
    model: &str,
    custom_vocabulary: &str,
    custom_system_prompt: &str,
    base_url: &str,
) -> Result<PostProcessingResult, PostProcessingError> {
    let url = format!("{}/chat/completions", base_url);

    let vocabulary_terms = merged_vocabulary_terms(custom_vocabulary);
    let normalized_vocab = normalized_vocabulary_text(&vocabulary_terms);

    let vocabulary_prompt = if !normalized_vocab.is_empty() {
        format!(
            "The following vocabulary must be treated as high-priority terms while rewriting.\n\
             Use these spellings exactly in the output when relevant:\n{}",
            normalized_vocab
        )
    } else {
        String::new()
    };

    let mut system_prompt = if custom_system_prompt.trim().is_empty() {
        DEFAULT_SYSTEM_PROMPT.to_string()
    } else {
        custom_system_prompt.trim().to_string()
    };

    if !vocabulary_prompt.is_empty() {
        system_prompt.push_str("\n\n");
        system_prompt.push_str(&vocabulary_prompt);
    }

    let user_message = format!(
        "CONTEXT: {}\n\nRAW_TRANSCRIPTION: {}",
        context_summary, transcript
    );

    let prompt_for_display = format!(
        "Model: {}\n\n[System]\n{}\n\n[User]\n{}",
        model, system_prompt, user_message
    );

    let payload = ChatCompletionRequest {
        model: model.to_string(),
        temperature: 0.0,
        response_format: ResponseFormat {
            r#type: "json_object".to_string(),
        },
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_message,
            },
        ],
    };

    let response = tokio::time::timeout(
        Duration::from_secs(POST_PROCESSING_TIMEOUT_SECS),
        client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send(),
    )
    .await
    .map_err(|_| PostProcessingError::TimedOut(POST_PROCESSING_TIMEOUT_SECS))?
    .map_err(PostProcessingError::Http)?;

    let status = response.status().as_u16();
    if status != 200 {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        return Err(PostProcessingError::RequestFailed(status, body));
    }

    let completion: ChatCompletionResponse = response.json().await?;

    let content = completion
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| {
            PostProcessingError::InvalidResponse(
                "Missing choices[0].message.content".to_string(),
            )
        })?;

    let (cleaned_text, reasoning) = parse_response(&content);

    Ok(PostProcessingResult {
        transcript: cleaned_text,
        prompt: prompt_for_display,
        reasoning,
    })
}

/// Convenience wrapper using the default Groq base URL and default model.
pub async fn post_process_groq(
    client: &Client,
    api_key: &str,
    transcript: &str,
    context_summary: &str,
    custom_vocabulary: &str,
    custom_system_prompt: &str,
) -> Result<PostProcessingResult, PostProcessingError> {
    post_process(
        client,
        api_key,
        transcript,
        context_summary,
        DEFAULT_MODEL,
        custom_vocabulary,
        custom_system_prompt,
        GROQ_BASE_URL,
    )
    .await
}
