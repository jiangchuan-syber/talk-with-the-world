use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const DEEPSEEK_CHAT_COMPLETIONS_URL: &str =
    "https://api.deepseek.com/v1/chat/completions";

#[derive(Clone)]
pub struct TranslateService {
    client: Client,
    api_key: Arc<Mutex<String>>,
    model: Arc<Mutex<String>>,
    cancel_token: Arc<Mutex<Option<CancellationToken>>>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    temperature: f64,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

impl TranslateService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: Arc::new(Mutex::new(String::new())),
            model: Arc::new(Mutex::new("deepseek-v4-flash".to_string())),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_api_key(&self, key: String) {
        *self.api_key.lock().await = key;
    }

    pub async fn set_model(&self, model: String) {
        *self.model.lock().await = model;
    }

    pub async fn translate(&self, chinese_text: &str) -> Result<String, String> {
        let api_key = self.api_key.lock().await.clone();
        if api_key.is_empty() {
            return Err("API key not configured".to_string());
        }

        let model = self.model.lock().await.clone();
        let url = DEEPSEEK_CHAT_COMPLETIONS_URL;
        let request_start = Instant::now();
        let token = CancellationToken::new();
        {
            *self.cancel_token.lock().await = Some(token.clone());
        }

        let request = ChatRequest {
            model: model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a translator. Translate the following Chinese text to natural English. Output ONLY the translation, nothing else. Do not add quotes or explanations.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: chinese_text.to_string(),
                },
            ],
            stream: false,
            temperature: 0.3,
        };

        let fut = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send();

        tokio::select! {
            _ = token.cancelled() => {
                Err("Translation cancelled".to_string())
            }
            result = fut => {
                match result {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            let status = resp.status();
                            let body = resp.text().await.unwrap_or_default();
                            return Err(format!("API error {}: {}", status, body));
                        }
                        let chat_resp: ChatResponse = resp
                            .json()
                            .await
                            .map_err(|e| format!("Parse error: {}", e))?;
                        chat_resp
                            .choices
                            .first()
                            .map(|c| {
                                let output = c.message.content.trim().to_string();
                                log::info!(
                                    "Chat API request finished in {}ms (model={}, input_chars={}, output_chars={})",
                                    request_start.elapsed().as_millis(),
                                    model,
                                    chinese_text.chars().count(),
                                    output.chars().count()
                                );
                                output
                            })
                            .ok_or_else(|| "Empty response".to_string())
                    }
                    Err(e) => Err(format!("Request error: {}", e)),
                }
            }
        }
    }
}
