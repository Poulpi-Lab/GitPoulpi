//! The two HTTP dialects: OpenAI chat-completions and Anthropic Messages.

use std::sync::Arc;

use gitpoulpi_hosting::{HttpRequest, HttpResponse, HttpTransport};
use serde_json::{json, Value};

use crate::{AiClient, Completion, Error, Request, Result, Usage};

/// OpenAI's chat-completions dialect. Also serves OpenRouter and any
/// OpenAI-compatible server, which differ only by base URL and key.
pub struct OpenAiClient {
    transport: Arc<dyn HttpTransport>,
    base_url: String,
    model: String,
    key: Option<String>,
}

impl OpenAiClient {
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        base_url: &str,
        model: &str,
        key: Option<String>,
    ) -> Self {
        Self {
            transport,
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            key,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for OpenAiClient {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        self.chat(json!([
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ]))
        .await
    }

    /// The earlier turns of an exchange as real chat messages, after the
    /// instructions and the repository context (one system message: the
    /// order OpenAI-style servers cache a prefix by), the conversation first
    /// and what changes last.
    async fn complete_request(&self, request: &Request<'_>) -> Result<Completion> {
        let mut messages = vec![json!({"role": "system", "content": request.system_text()})];
        for turn in request.history {
            messages.push(json!({"role": "user", "content": turn.user}));
            messages.push(json!({"role": "assistant", "content": turn.assistant}));
        }
        messages.push(json!({"role": "user", "content": request.user}));
        Ok(Completion {
            text: self.chat(Value::Array(messages)).await?,
            usage: Usage::default(),
        })
    }
}

impl OpenAiClient {
    /// One chat-completions call with `messages`; the answer's text.
    async fn chat(&self, messages: Value) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": messages,
        });
        let mut request =
            HttpRequest::post_json(format!("{}/chat/completions", self.base_url), &body);
        if let Some(key) = &self.key {
            request = request.header("Authorization", format!("Bearer {key}"));
        }
        let response = send(&*self.transport, request).await?;
        let value: Value =
            serde_json::from_str(&response.body).map_err(|e| Error::Decode(e.to_string()))?;
        let text = value["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Decode("no choices[0].message.content in the answer".into()))?;
        non_empty(text)
    }
}

/// The Anthropic Messages API.
pub struct AnthropicClient {
    transport: Arc<dyn HttpTransport>,
    base_url: String,
    model: String,
    key: String,
}

impl AnthropicClient {
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        base_url: &str,
        model: &str,
        key: String,
    ) -> Self {
        Self {
            transport,
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            key,
        }
    }
}

impl AnthropicClient {
    /// One Messages call with `system` as the API takes it (a string, or
    /// blocks with `cache_control`); the answer's text and cache counters.
    async fn messages(&self, system: Value, messages: Value) -> Result<Completion> {
        let body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system,
            "messages": messages,
        });
        let request = HttpRequest::post_json(format!("{}/messages", self.base_url), &body)
            .header("x-api-key", self.key.clone())
            .header("anthropic-version", "2023-06-01");
        let response = send(&*self.transport, request).await?;
        let value: Value =
            serde_json::from_str(&response.body).map_err(|e| Error::Decode(e.to_string()))?;
        let blocks = value["content"]
            .as_array()
            .ok_or_else(|| Error::Decode("no content array in the answer".into()))?;
        let text: String = blocks
            .iter()
            .filter(|b| b["type"] == "text")
            .filter_map(|b| b["text"].as_str())
            .collect();
        let usage = Usage {
            cache_read_tokens: value["usage"]["cache_read_input_tokens"]
                .as_u64()
                .unwrap_or(0),
            cache_creation_tokens: value["usage"]["cache_creation_input_tokens"]
                .as_u64()
                .unwrap_or(0),
        };
        non_empty(&text)?;
        Ok(Completion { text, usage })
    }
}

#[async_trait::async_trait]
impl AiClient for AnthropicClient {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let messages = json!([{"role": "user", "content": user}]);
        Ok(self.messages(json!(system), messages).await?.text)
    }

    /// The instructions and the repository context are two system blocks,
    /// each marked `cache_control: ephemeral`, so the provider reuses them for
    /// the next reply (the conversation, hint and draft are in the user
    /// message and change every time).
    ///
    /// Earlier turns of the exchange go as real messages, and the stable
    /// prefix stays under `cache_control`: the instructions, the context, the
    /// first user message (it holds the conversation, the same in every
    /// request of the exchange) and the newest answer. That is four markers
    /// at most, the API's limit; only the final instruction is new.
    async fn complete_request(&self, request: &Request<'_>) -> Result<Completion> {
        let context = request.context.filter(|c| !c.text.is_empty());
        if context.is_none() && request.history.is_empty() {
            return Ok(Completion {
                text: self.complete(request.system, request.user).await?,
                usage: Usage::default(),
            });
        }
        let mark = json!({"type": "ephemeral"});
        let mut system =
            vec![json!({"type": "text", "text": request.system, "cache_control": mark})];
        if let Some(context) = context {
            system.push(json!({"type": "text", "text": context.text, "cache_control": mark}));
        }
        let cached = |role: &str, text: &str| {
            json!({
                "role": role,
                "content": [{"type": "text", "text": text, "cache_control": mark}],
            })
        };
        let mut messages = Vec::new();
        let last = request.history.len().saturating_sub(1);
        for (i, turn) in request.history.iter().enumerate() {
            messages.push(if i == 0 {
                cached("user", &turn.user)
            } else {
                json!({"role": "user", "content": turn.user})
            });
            messages.push(if i == last {
                cached("assistant", &turn.assistant)
            } else {
                json!({"role": "assistant", "content": turn.assistant})
            });
        }
        messages.push(json!({"role": "user", "content": request.user}));
        self.messages(Value::Array(system), Value::Array(messages))
            .await
    }
}

async fn send(transport: &dyn HttpTransport, request: HttpRequest) -> Result<HttpResponse> {
    let response = transport.send(request).await.map_err(|e| match e {
        // The security check refused: keep it typed for the dialog.
        gitpoulpi_hosting::Error::Trust(problem) => Error::Trust(problem),
        other => Error::Http(other.to_string()),
    })?;
    if response.is_success() {
        return Ok(response);
    }
    let detail = error_detail(&response.body);
    Err(match response.status {
        401 | 403 => Error::Unauthorized(detail),
        status => Error::Api { status, detail },
    })
}

/// Both dialects put the human message at `error.message` (or `error` as
/// a string); anything else is the truncated raw body.
fn error_detail(body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        if let Some(text) = value["error"]["message"]
            .as_str()
            .or_else(|| value["error"].as_str())
            .or_else(|| value["message"].as_str())
        {
            return text.to_string();
        }
    }
    let body = body.trim();
    if body.chars().count() > 300 {
        format!("{}...", body.chars().take(300).collect::<String>())
    } else {
        body.to_string()
    }
}

fn non_empty(text: &str) -> Result<String> {
    if text.trim().is_empty() {
        Err(Error::Empty)
    } else {
        Ok(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_hosting::RecordedTransport;

    fn run<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn openai_posts_chat_completions_with_a_bearer_key_and_reads_the_answer() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/chat/completions",
            200,
            r#"{"choices":[{"message":{"content":"feat: add x"}}]}"#,
        ));
        let client = OpenAiClient::new(
            transport.clone(),
            "https://api.example/v1/",
            "gpt-test",
            Some("sk-secret".into()),
        );
        let answer = run(client.complete("sys", "usr")).unwrap();
        assert_eq!(answer, "feat: add x");
        let seen = transport.requests();
        assert_eq!(seen[0].url, "https://api.example/v1/chat/completions");
        assert!(seen[0]
            .headers
            .contains(&("Authorization".into(), "Bearer sk-secret".into())));
        let body: Value = serde_json::from_str(seen[0].body.as_ref().unwrap()).unwrap();
        assert_eq!(body["model"], "gpt-test");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "usr");
    }

    #[test]
    fn a_keyless_openai_compatible_server_gets_no_authorization_header() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/chat/completions",
            200,
            r#"{"choices":[{"message":{"content":"ok"}}]}"#,
        ));
        let client = OpenAiClient::new(transport.clone(), "http://localhost:11434/v1", "m", None);
        run(client.complete("s", "u")).unwrap();
        assert!(transport.requests()[0]
            .headers
            .iter()
            .all(|(k, _)| k != "Authorization"));
    }

    #[test]
    fn anthropic_posts_messages_with_its_own_headers() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/messages",
            200,
            r#"{"content":[{"type":"text","text":"fix: "},{"type":"text","text":"y"}]}"#,
        ));
        let client = AnthropicClient::new(
            transport.clone(),
            "https://api.anthropic.com/v1",
            "claude-x",
            "ak".into(),
        );
        assert_eq!(run(client.complete("sys", "usr")).unwrap(), "fix: y");
        let seen = transport.requests();
        assert!(seen[0].headers.contains(&("x-api-key".into(), "ak".into())));
        assert!(seen[0]
            .headers
            .contains(&("anthropic-version".into(), "2023-06-01".into())));
        let body: Value = serde_json::from_str(seen[0].body.as_ref().unwrap()).unwrap();
        assert_eq!(body["system"], "sys");
        assert_eq!(body["messages"][0]["content"], "usr");
    }

    #[test]
    fn a_rejected_key_and_an_api_error_are_told_apart() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/chat/completions",
            401,
            r#"{"error":{"message":"Bad key"}}"#,
        ));
        let client = OpenAiClient::new(transport, "http://x/v1", "m", Some("k".into()));
        assert_eq!(
            run(client.complete("s", "u")),
            Err(Error::Unauthorized("Bad key".into()))
        );
        let transport =
            Arc::new(RecordedTransport::new().on("/messages", 429, r#"{"error":"slow down"}"#));
        let client = AnthropicClient::new(transport, "http://x/v1", "m", "k".into());
        assert_eq!(
            run(client.complete("s", "u")),
            Err(Error::Api {
                status: 429,
                detail: "slow down".into()
            })
        );
    }

    #[test]
    fn an_empty_or_malformed_answer_is_an_error() {
        let transport = Arc::new(RecordedTransport::new().on(
            "/chat/completions",
            200,
            r#"{"choices":[{"message":{"content":"  "}}]}"#,
        ));
        let client = OpenAiClient::new(transport, "http://x/v1", "m", None);
        assert_eq!(run(client.complete("s", "u")), Err(Error::Empty));
        let transport = Arc::new(RecordedTransport::new().on("/chat/completions", 200, "nope"));
        let client = OpenAiClient::new(transport, "http://x/v1", "m", None);
        assert!(matches!(
            run(client.complete("s", "u")),
            Err(Error::Decode(_))
        ));
    }
}
