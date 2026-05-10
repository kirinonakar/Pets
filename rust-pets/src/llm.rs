use keyring_core::Entry;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    LmStudio,
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub lm_studio_endpoint: String,
    pub lm_studio_model: String,
    pub google_model: String,
    // Google API key is NOT stored in the config file, but in Credential Manager
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::LmStudio,
            lm_studio_endpoint: "http://localhost:1234/v1".to_string(),
            lm_studio_model: String::new(),
            google_model: "gemini-3.1-flash-lite-preview".to_string(),
        }
    }
}

pub const GOOGLE_MODELS: &[&str] = &[
    "gemini-3.1-flash-lite-preview",
    "gemini-3-flash-preview",
    "gemini-3.1-pro-preview",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemma-4-26b-a4b-it",
    "gemma-4-31b-it",
];

const SERVICE_NAME: &str = "rust-pets";
const KEY_NAME: &str = "google-api-key";

static KEYRING_INIT: OnceLock<Result<(), String>> = OnceLock::new();

fn ensure_keyring_store() -> Result<(), Box<dyn Error>> {
    let result = KEYRING_INIT.get_or_init(|| {
        keyring::use_native_store(false).map_err(|err| err.to_string())
    });

    match result {
        Ok(()) => Ok(()),
        Err(message) => Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            message.clone(),
        ))),
    }
}

fn google_api_key_entry() -> Result<Entry, Box<dyn Error>> {
    ensure_keyring_store()?;
    Ok(Entry::new(SERVICE_NAME, KEY_NAME)?)
}

pub fn get_google_api_key() -> Option<String> {
    let entry = google_api_key_entry().ok()?;
    entry.get_password().ok()
}

pub fn set_google_api_key(key: &str) -> Result<(), Box<dyn Error>> {
    let entry = google_api_key_entry()?;

    if key.is_empty() {
        let _ = entry.delete_credential();
    } else {
        entry.set_password(key)?;
    }

    Ok(())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct LmStudioModelsResponse {
    data: Vec<LmStudioModel>,
}

#[derive(Deserialize)]
struct LmStudioModel {
    id: String,
}

#[derive(Serialize)]
struct LmStudioChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct LmStudioChatResponse {
    choices: Vec<LmStudioChoice>,
}

#[derive(Deserialize)]
struct LmStudioChoice {
    message: ChatMessage,
}

#[derive(Serialize)]
struct GoogleGeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GoogleContent>,
    pub contents: Vec<GoogleContent>,
}

#[derive(Serialize)]
struct GoogleContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    pub text: String,
}

#[derive(Deserialize)]
struct GoogleGeminiResponse {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleContentResponse,
}

#[derive(Deserialize)]
struct GoogleContentResponse {
    parts: Vec<GooglePartResponse>,
}

#[derive(Deserialize)]
struct GooglePartResponse {
    text: String,
}

pub async fn fetch_lm_studio_models(endpoint: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let data: LmStudioModelsResponse = resp.json().await?;
    Ok(data.data.into_iter().map(|m| m.id).collect())
}

pub async fn chat_completion(
    config: &LlmConfig,
    api_key: Option<&str>,
    messages: Vec<ChatMessage>,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();

    match config.provider {
        LlmProvider::LmStudio => {
            let url = format!("{}/chat/completions", config.lm_studio_endpoint.trim_end_matches('/'));
            let req = LmStudioChatRequest {
                model: config.lm_studio_model.clone(),
                messages,
                temperature: 0.7,
            };
            let resp = client.post(url).json(&req).send().await?;
            let data: LmStudioChatResponse = resp.json().await?;
            Ok(data.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
        }
        LlmProvider::Google => {
            let key = api_key.ok_or("Google API Key is missing")?;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                config.google_model
            );
            
            let mut system_instruction = None;
            let mut contents = Vec::new();

            for m in messages {
                if m.role == "system" {
                    system_instruction = Some(GoogleContent {
                        role: None,
                        parts: vec![GooglePart { text: m.content }],
                    });
                } else {
                    let role = if m.role == "assistant" || m.role == "model" {
                        "model".to_string()
                    } else {
                        "user".to_string()
                    };
                    contents.push(GoogleContent {
                        role: Some(role),
                        parts: vec![GooglePart { text: m.content }],
                    });
                }
            }

            let req = GoogleGeminiRequest { system_instruction, contents };
            let resp = client.post(url)
                .header("x-goog-api-key", key)
                .json(&req)
                .send()
                .await?;
            
            if !resp.status().is_success() {
                let err_text = resp.text().await?;
                let sanitized_err = err_text.replace(key, "***");
                return Err(format!("Google API Error: {}", sanitized_err).into());
            }

            let data: GoogleGeminiResponse = resp.json().await?;
            Ok(data.candidates.first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.clone())
                .unwrap_or_default())
        }
    }
}