use crate::audio_player::AudioPlayer;
use crate::backends::edge::EdgeTTS;
use crate::backends::google::GoogleTTS;
use crate::backends::TTSBackend;
use crate::config::TTSConfig;
use crate::models::{TTSError, Voice};
use log::{info};
use std::path::Path;
use tokio::fs;

/// TTS Processor that delegates to a configured backend
pub struct TTSProcessor {
    config: TTSConfig,
    voices_cache: Option<Vec<Voice>>,
    backend: Box<dyn TTSBackend + Send + Sync>,
}

impl TTSProcessor {
    /// Create a new TTSProcessor with optional configuration
    pub fn new(config: Option<TTSConfig>) -> Self {
        let config = config.unwrap_or_default();
        let backend: Box<dyn TTSBackend + Send + Sync> = match config.backend.as_str() {
            "google" => Box::new(GoogleTTS::new()),
            _ => Box::new(EdgeTTS::new()),
        };

        Self {
            config,
            voices_cache: None,
            backend,
        }
    }

    /// Convert text to audio data using the configured backend
    pub async fn synthesize_text(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError> {
        self.backend.synthesize_text(text, voice).await
    }

    /// Get all available voices from the configured backend
    pub async fn list_voices(&mut self) -> Result<Vec<Voice>, TTSError> {
        if self.config.cache_voices {
            if let Some(ref voices) = self.voices_cache {
                info!("Using cached voices");
                return Ok(voices.clone());
            }
        }

        info!("Fetching voices from backend");
        let voices = self.backend.list_voices().await?;

        if self.config.cache_voices {
            self.voices_cache = Some(voices.clone());
        }

        Ok(voices)
    }

    /// Synthesize text and play it, optionally saving to a file
    pub async fn synthesize_and_play(
        &self,
        text: &str,
        voice: &str,
        output_path: Option<&Path>,
        play: bool,
    ) -> Result<(), TTSError> {
        let audio_data = self.synthesize_text(text, voice).await?;

        if let Some(path) = output_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(path, &audio_data).await?;
        }

        if play {
            let player = AudioPlayer::new().map_err(|e| TTSError::Synthesis(e.to_string()))?;
            player
                .play_audio_data(audio_data, Some("mp3"))
                .map_err(|e| TTSError::Synthesis(e.to_string()))?;
        }

        Ok(())
    }

    /// Clear the cached voice list
    pub fn clear_voice_cache(&mut self) {
        self.voices_cache = None;
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

    #[test]
    fn test_tts_config_default() {
        let config = TTSConfig::default();
        assert_eq!(config.default_voice, "en-US-AriaNeural");
        assert_eq!(config.output_format, "mp3");
        assert_eq!(config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_tts_client_creation() {
        let client = TTSProcessor::new(None);
        assert_eq!(client.config.default_voice, "en-US-AriaNeural");
    }
}

impl TTSProcessor {
    pub async fn save_audio(&self, audio_data: &[u8], filename: &str) -> Result<(), TTSError> {
        self.backend.save_audio(audio_data, filename).await
    }
}
