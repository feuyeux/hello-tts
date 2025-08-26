use crate::models::TTSError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for TTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSConfig {
    pub default_voice: String,
    pub backend: String,
    pub output_format: String,
    pub output_directory: String,
    pub auto_play: bool,
    pub cache_voices: bool,
    pub max_retries: u32,
    pub timeout: Duration,
    pub rate: String,
    pub pitch: String,
    pub volume: String,
    pub batch_size: usize,
    pub max_concurrent: usize,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            default_voice: "en-US-AriaNeural".to_string(),
            backend: "edge".to_string(),
            output_format: "mp3".to_string(),
            output_directory: "./output".to_string(),
            auto_play: true,
            cache_voices: true,
            max_retries: 3,
            timeout: Duration::from_secs(30),
            rate: "0%".to_string(),
            pitch: "0%".to_string(),
            volume: "100%".to_string(),
            batch_size: 5,
            max_concurrent: 3,
        }
    }
}

impl TTSConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), TTSError> {
        if self.default_voice.is_empty() {
            return Err(TTSError::Config(
                "default_voice cannot be empty".to_string(),
            ));
        }
        if self.batch_size == 0 {
            return Err(TTSError::Config("batch_size must be positive".to_string()));
        }
        if self.max_concurrent == 0 {
            return Err(TTSError::Config(
                "max_concurrent must be positive".to_string(),
            ));
        }
        Ok(())
    }

    /// Load configuration from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, TTSError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TTSError::Config(format!("Failed to read config file {}: {}", path, e)))?;

        let config: TTSConfig = serde_json::from_str(&content)
            .map_err(|e| TTSError::Config(format!("Invalid JSON in config file: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn to_json_file(&self, path: &str) -> Result<(), TTSError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| TTSError::Config(format!("Failed to serialize config: {}", e)))?;

        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TTSError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        std::fs::write(path, content).map_err(|e| {
            TTSError::Config(format!("Failed to write config file {}: {}", path, e))
        })?;

        Ok(())
    }
}

// Used to parse shared/tts_config.json
#[derive(Debug, Deserialize)]
pub struct LanguageConfig {
    pub code: String,
    pub name: String,
    pub flag: Option<String>,
    pub text: Option<String>,
    pub edge_voice: Option<String>,
    pub google_voice: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TTSConfigFile {
    pub languages: Vec<LanguageConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_config_default() {
        let config = TTSConfig::default();
        assert_eq!(config.default_voice, "en-US-AriaNeural");
        assert_eq!(config.output_format, "mp3");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.backend, "edge");
    }
}
