//! App configuration — settings persistence (JSON file)
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub transcription_provider: String,
    pub post_processing_enabled: bool,
    pub post_processing_model: String,
    pub api_base_url: String,
    pub minimum_recording_duration_ms: f64,
    pub custom_vocabulary: String,
    pub custom_system_prompt: String,
    pub custom_context_prompt: String,
    pub selected_hotkey: String,
    pub selected_microphone_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transcription_provider: "local".to_string(),
            post_processing_enabled: false,
            post_processing_model: "meta-llama/llama-4-scout-17b-16e-instruct".to_string(),
            api_base_url: "https://api.groq.com/openai/v1".to_string(),
            minimum_recording_duration_ms: 200.0,
            custom_vocabulary: String::new(),
            custom_system_prompt: String::new(),
            custom_context_prompt: String::new(),
            selected_hotkey: "fn".to_string(),
            selected_microphone_id: "default".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    /// Default config file path for the current platform
    pub fn default_path(app_name: &str) -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home)
                .join("Library/Application Support")
                .join(app_name)
                .join("config.json")
        }
        #[cfg(target_os = "android")]
        {
            PathBuf::from("/data/data/me.gulya.wrenflow/files/config.json")
        }
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(appdata).join(app_name).join("config.json")
        }
        #[cfg(not(any(target_os = "macos", target_os = "android", target_os = "windows")))]
        {
            PathBuf::from(".").join("config.json")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let config = AppConfig::default();
        config.save(&path).unwrap();
        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded.transcription_provider, "local");
        assert!(!loaded.post_processing_enabled);
    }

    #[test]
    fn load_or_default_missing_file() {
        let config = AppConfig::load_or_default(Path::new("/nonexistent/config.json"));
        assert_eq!(config.minimum_recording_duration_ms, 200.0);
    }
}
