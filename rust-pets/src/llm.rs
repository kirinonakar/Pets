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


#[derive(Deserialize)]
struct LmStudioModelsResponse {
    data: Vec<LmStudioModel>,
}

#[derive(Deserialize)]
struct LmStudioModel {
    id: String,
}

pub async fn fetch_lm_studio_models(endpoint: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let data: LmStudioModelsResponse = resp.json().await?;
    Ok(data.data.into_iter().map(|m| m.id).collect())
}

// Helper for checking loaded models in LM Studio (often shown in the model list if multiple are available)
// But LM Studio usually returns all available models.
// Some OpenAI compatible endpoints return 'owned_by' or similar but LM Studio's /v1/models is standard.