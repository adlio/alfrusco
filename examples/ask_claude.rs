use alfrusco::{config, AsyncRunnable, Item, Workflow, WorkflowError};
use clap::Parser;
use serde::{Deserialize, Serialize};

/// An Alfred workflow that sends your query to Claude and returns the
/// response as Alfred items. Each paragraph in Claude's response becomes
/// a separate item that can be actioned (copied to clipboard via arg).
///
/// Requires the ANTHROPIC_API_KEY environment variable to be set (configure
/// this in Alfred's workflow environment variables).
///
#[derive(Parser, Debug)]
struct AskClaudeWorkflow {
    /// The question or prompt to send to Claude
    query: Vec<String>,

    /// Anthropic API key (reads from ANTHROPIC_API_KEY env var)
    #[arg(long, env = "ANTHROPIC_API_KEY", hide = true)]
    api_key: String,

    /// Model to use
    #[arg(long, default_value = "claude-sonnet-4-20250514")]
    model: String,

    /// System prompt to guide Claude's responses
    #[arg(long, default_value = "You are a helpful assistant inside an Alfred workflow. Keep responses concise and actionable. Respond with short paragraphs — each paragraph will become a selectable item for the user.")]
    system: String,

    /// Override API base URL (for testing)
    #[arg(long, default_value = "https://api.anthropic.com")]
    api_url: String,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let command = AskClaudeWorkflow::parse();
    alfrusco::execute_async(&config::AlfredEnvProvider, command, &mut std::io::stdout()).await;
}

#[async_trait::async_trait]
impl AsyncRunnable for AskClaudeWorkflow {
    type Error = AskClaudeError;

    async fn run_async(self, wf: &mut Workflow) -> Result<(), AskClaudeError> {
        let query = self.query.join(" ");
        if query.is_empty() {
            wf.append_item(
                Item::new("Ask Claude...")
                    .subtitle("Type a question or prompt")
                    .valid(false),
            );
            return Ok(());
        }

        let request = MessagesRequest {
            model: self.model,
            max_tokens: 1024,
            system: Some(self.system),
            messages: vec![Message {
                role: "user".to_string(),
                content: query,
            }],
        };

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/messages", self.api_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AskClaudeError::Api(format!("{status}: {body}")));
        }

        let messages_response: MessagesResponse = response.json().await?;

        // Extract text from the response content blocks
        let full_text: String = messages_response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Split into paragraphs and create an item for each
        let paragraphs: Vec<&str> = full_text
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();

        if paragraphs.is_empty() {
            wf.append_item(Item::new("No response from Claude").valid(false));
        } else if paragraphs.len() == 1 {
            // Single paragraph — show it directly
            wf.append_item(
                Item::new(paragraphs[0])
                    .subtitle("Press Enter to copy")
                    .arg(paragraphs[0]),
            );
        } else {
            // Multiple paragraphs — one item each
            for (i, paragraph) in paragraphs.iter().enumerate() {
                // Use the first line as the title, rest as subtitle if long
                let lines: Vec<&str> = paragraph.lines().collect();
                let title = lines[0];
                let subtitle = if lines.len() > 1 {
                    lines[1..].join(" ")
                } else {
                    format!("({}/{}). Press Enter to copy", i + 1, paragraphs.len())
                };
                wf.append_item(Item::new(title).subtitle(subtitle).arg(*paragraph));
            }
        }

        Ok(())
    }
}

// --- Anthropic API Types ---

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

// --- Error Type ---

#[derive(Debug)]
pub enum AskClaudeError {
    Reqwest(reqwest::Error),
    Api(String),
}

impl From<reqwest::Error> for AskClaudeError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl WorkflowError for AskClaudeError {}

impl std::fmt::Display for AskClaudeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AskClaudeError::Reqwest(e) => write!(f, "HTTP error: {e}"),
            AskClaudeError::Api(msg) => write!(f, "Anthropic API error: {msg}"),
        }
    }
}

impl std::error::Error for AskClaudeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AskClaudeError::Reqwest(e) => Some(e),
            AskClaudeError::Api(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const MOCK_RESPONSE: &str = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": "Here is the answer.\n\nIt has multiple paragraphs."}
        ],
        "model": "claude-sonnet-4-20250514",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 20}
    }"#;

    #[tokio::test]
    async fn test_ask_claude_with_mock() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "test-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(MOCK_RESPONSE, "application/json"),
            )
            .mount(&server)
            .await;

        let command = AskClaudeWorkflow {
            query: vec!["What is Rust?".to_string()],
            api_key: "test-key".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            system: "Be concise.".to_string(),
            api_url: server.uri(),
        };

        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Here is the answer."));
        assert!(output.contains("It has multiple paragraphs."));
    }

    #[tokio::test]
    async fn test_ask_claude_empty_query() {
        let command = AskClaudeWorkflow {
            query: vec![],
            api_key: "test-key".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            system: "Be concise.".to_string(),
            api_url: "http://unused".to_string(),
        };

        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Ask Claude..."));
    }

    #[tokio::test]
    async fn test_ask_claude_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let command = AskClaudeWorkflow {
            query: vec!["test".to_string()],
            api_key: "bad-key".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            system: "Be concise.".to_string(),
            api_url: server.uri(),
        };

        let mut buffer = Vec::new();
        let dir = tempfile::tempdir().unwrap().keep();
        alfrusco::execute_async(&config::TestingProvider(dir), command, &mut buffer).await;
        let output = String::from_utf8(buffer).unwrap();
        // Should show an error item
        assert!(output.contains("Anthropic API error"));
    }
}
