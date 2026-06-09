use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "deepseek-v4-flash".to_string(),
            enabled: true,
        }
    }
}

impl AppConfig {
    pub fn normalized(mut self) -> Self {
        if self.model != "deepseek-v4-flash" && self.model != "deepseek-v4-pro" {
            self.model = "deepseek-v4-flash".to_string();
        }

        self
    }
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("cn2en");
    fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let config = if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    config.normalized()
}

/// Create `config.json` with an **empty API key** only if missing. User keys only live in this file
/// after they click Save; installers do not bundle it.
pub fn init_config_file_if_missing() -> Result<(), String> {
    let path = config_path();
    if path.exists() {
        return Ok(());
    }
    save_config(&AppConfig::default().normalized())
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    let normalized = config.clone().normalized();
    let json = serde_json::to_string_pretty(&normalized).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
