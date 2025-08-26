use serde::{Deserialize, Serialize};

/// Custom error type for TTS operations
#[derive(Debug, thiserror::Error)]
pub enum TTSError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TTS synthesis failed: {0}")]
    Synthesis(String),
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Voice information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub name: String,
    pub display_name: String,
    pub locale: String,
    pub gender: String,
    pub description: Option<String>,
}

impl Voice {
    pub fn new(name: String, display_name: String, locale: String, gender: String) -> Self {
        Self {
            name,
            display_name,
            locale,
            gender,
            description: None,
        }
    }

    pub fn matches_language(&self, lang_code: &str) -> bool {
        self.locale.to_lowercase().starts_with(&lang_code.to_lowercase())
    }

    pub fn language_code(&self) -> &str {
        self.locale.split('-').next().unwrap_or("en")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_creation() {
        let voice = Voice::new(
            "en-US-AriaNeural".to_string(),
            "Aria".to_string(),
            "en-US".to_string(),
            "Female".to_string(),
        );

        assert_eq!(voice.name, "en-US-AriaNeural");
        assert_eq!(voice.display_name, "Aria");
        assert_eq!(voice.locale, "en-US");
        assert_eq!(voice.gender, "Female");
        assert_eq!(voice.language_code(), "en");
    }

    #[test]
    fn test_voice_matches_language() {
        let voice = Voice::new(
            "en-US-AriaNeural".to_string(),
            "Aria".to_string(),
            "en-US".to_string(),
            "Female".to_string(),
        );

        assert!(voice.matches_language("en"));
        assert!(voice.matches_language("en-US"));
        assert!(!voice.matches_language("fr"));
    }
}
