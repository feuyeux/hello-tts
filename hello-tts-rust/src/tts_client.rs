use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::fs;
use log::{info, error};

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
}

impl Voice {
    pub fn new(name: String, display_name: String, locale: String, gender: String) -> Self {
        Self {
            name,
            display_name,
            locale,
            gender,
        }
    }

    /// Get language code from locale (e.g., 'en' from 'en-US')
    pub fn language_code(&self) -> &str {
        self.locale.split('-').next().unwrap_or(&self.locale)
    }

    /// Check if this voice matches the given language code
    pub fn matches_language(&self, language: &str) -> bool {
        self.locale == language || self.language_code() == language
    }
}

/// Configuration for TTS client
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

        // Create directory if it doesn't exist
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

/// Edge TTS voice data structure from API
#[derive(Debug, Deserialize)]
struct EdgeVoiceData {
    #[serde(rename = "ShortName")]
    short_name: String,
    #[serde(rename = "FriendlyName")]
    friendly_name: String,
    #[serde(rename = "Locale")]
    locale: String,
    #[serde(rename = "Gender")]
    gender: String,
}

/// TTS Processor for Microsoft Edge TTS service
pub struct TTSProcessor {
    client: Client,
    config: TTSConfig,
    voices_cache: Option<Vec<Voice>>,
}

impl TTSProcessor {
    /// Create a new TTSProcessor with optional configuration
    pub fn new(config: Option<TTSConfig>) -> Self {
        let config = config.unwrap_or_default();
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            voices_cache: None,
        }
    }

    /// Convert text to audio data using specified voice
    pub async fn synthesize_text(
        &self,
        text: &str,
        voice: &str,
    ) -> Result<Vec<u8>, TTSError> {
        match self.config.backend.as_str() {
            "google" => self.synthesize_via_google_tts(text, voice).await,
            _ => self.synthesize_via_edge_tts(text, voice).await,
        }
    }

    /// Compatibility shim: synthesize with additional options (e.g., SSML)
    /// Examples/old code may call this signature. For now we forward to synthesize_text.
    pub async fn synthesize_text_with_options<T: Into<Option<bool>>>(
        &self,
        text: &str,
        voice: &str,
        _use_ssml: T,
    ) -> Result<Vec<u8>, TTSError> {
        // Accept either bool or Option<bool> for compatibility. Currently SSML option is ignored.
        let _flag: bool = _use_ssml.into().unwrap_or(false);
        self.synthesize_text(text, voice).await
    }

    /// Use Python edge-tts library via process execution
    async fn synthesize_via_edge_tts(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError> {
        use std::process::Stdio;
        use tokio::process::Command;

        // Create temporary file for output (use MP3 format)
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "tts_output_{}.mp3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        // Try edge-tts command
        let mut cmd = Command::new("edge-tts");
        cmd.args([
            "--voice",
            voice,
            "--text",
            text,
            "--write-media",
            temp_file.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let output = cmd.output().await;

        let success = match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        // If direct edge-tts command fails, try python -m edge_tts
        if !success {
            let mut python_cmd = Command::new("python");
            python_cmd
                .args([
                    "-m",
                    "edge_tts",
                    "--voice",
                    voice,
                    "--text",
                    text,
                    "--write-media",
                    temp_file.to_str().unwrap(),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let python_output = python_cmd
                .output()
                .await
                .map_err(|e| TTSError::Synthesis(format!("Failed to execute edge-tts: {}", e)))?;

            if !python_output.status.success() {
                let stderr = String::from_utf8_lossy(&python_output.stderr);
                return Err(TTSError::Synthesis(format!("Edge TTS failed: {}", stderr)));
            }
        }

        // Read the generated audio file
        if temp_file.exists() {
            let audio_data = fs::read(&temp_file)
                .await
                .map_err(|e| TTSError::Synthesis(format!("Failed to read audio file: {}", e)))?;

            // Clean up temporary file
            let _ = fs::remove_file(&temp_file).await;

            Ok(audio_data)
        } else {
            Err(TTSError::Synthesis(
                "Audio file was not generated".to_string(),
            ))
        }
    }

    /// Use Python gTTS library via process execution
    async fn synthesize_via_google_tts(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError> {
        use std::process::Stdio;
        use tokio::process::Command;

        // Create temporary file for output
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "gtts_output_{}.mp3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        // Normalize voice -> language code for gTTS
        // e.g. "zh-CN-XiaoxiaoNeural" -> "zh-cn", "en-US-AriaNeural" -> "en-us", "en" -> "en"
        let vn = voice.replace('_', "-");
        let mut lang_code = "en".to_string();
        if vn.contains('-') {
            let parts: Vec<&str> = vn.split('-').collect();
            if parts.len() >= 2 && parts[1].len() == 2 {
                lang_code = format!("{}-{}", parts[0].to_lowercase(), parts[1].to_lowercase());
            } else {
                lang_code = parts[0].to_lowercase();
            }
        } else if !vn.is_empty() {
            lang_code = vn.to_lowercase();
        }

        // Execute gtts-cli command
        let mut cmd = Command::new("gtts-cli");
        cmd.args([
            "--text", text,
            "--lang", &lang_code,
            "--output", temp_file.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| TTSError::Synthesis(format!("Failed to execute gtts-cli: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TTSError::Synthesis(format!("Google TTS failed: {}", stderr)));
        }

        // Read the generated audio file
        if temp_file.exists() {
            let audio_data = fs::read(&temp_file)
                .await
                .map_err(|e| TTSError::Synthesis(format!("Failed to read audio file: {}", e)))?;

            // Clean up temporary file
            let _ = fs::remove_file(&temp_file).await;

            Ok(audio_data)
        } else {
            Err(TTSError::Synthesis(
                "Audio file was not generated".to_string(),
            ))
        }
    }

    /// Save audio data to file
    pub async fn save_audio(&self, audio_data: &[u8], filename: &str) -> Result<(), TTSError> {
        // Ensure output directory exists
        if let Some(parent) = std::path::Path::new(filename).parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(filename, audio_data).await?;
        Ok(())
    }

    /// Get all available voices from Edge TTS service
    pub async fn list_voices(&mut self) -> Result<Vec<Voice>, TTSError> {
        if self.config.cache_voices && self.voices_cache.is_some() {
            return Ok(self.voices_cache.as_ref().unwrap().clone());
        }

        let voices_url = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

        let response = self
            .client
            .get(voices_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TTSError::Synthesis(format!(
                "Failed to fetch voices: HTTP {}",
                response.status()
            )));
        }

        let voices_data: Vec<EdgeVoiceData> = response.json().await?;

        let voices: Vec<Voice> = voices_data
            .into_iter()
            .map(|v| Voice::new(v.short_name, v.friendly_name, v.locale, v.gender))
            .collect();

        if self.config.cache_voices {
            self.voices_cache = Some(voices.clone());
        }

        Ok(voices)
    }

    /// Get voices filtered by language code
    pub async fn get_voices_by_language(&mut self, language: &str) -> Result<Vec<Voice>, TTSError> {
        let all_voices = self.list_voices().await?;

        let filtered_voices: Vec<Voice> = all_voices
            .into_iter()
            .filter(|voice| voice.matches_language(language))
            .collect();

        Ok(filtered_voices)
    }

    /// Clear the cached voice list to force refresh on next request
    pub fn clear_voice_cache(&mut self) {
        self.voices_cache = None;
    }

    /// Convert multiple texts to audio data using specified voice
    pub async fn batch_synthesize_text(
        &self,
        texts: &[&str],
        voice: &str,
        use_ssml: bool,
    ) -> Result<Vec<Vec<u8>>, TTSError> {
        let mut results = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            info!(
                "Processing batch item {}/{}: {}...",
                i + 1,
                texts.len(),
                &text[..text.len().min(50)]
            );
            match self
                .synthesize_text_with_options(text, voice, use_ssml)
                .await
            {
                Ok(audio_data) => results.push(audio_data),
                Err(e) => {
                    return Err(TTSError::Synthesis(format!(
                        "Failed to synthesize batch item {}: {}",
                        i + 1,
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Convert multiple texts to audio data concurrently using specified voice
    /// Note: This is a simplified implementation for demonstration
    pub async fn batch_synthesize_concurrent(
        &self,
        texts: &[&str],
        voice: &str,
        use_ssml: bool,
        _max_concurrent: usize,
    ) -> Result<Vec<Vec<u8>>, TTSError> {
        // For simplicity, we'll process sequentially but with async/await
        // In a real implementation, you would use proper concurrent processing with Arc<Self>
        let mut results = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            info!(
                "Processing concurrent item {}/{}: {}...",
                i + 1,
                texts.len(),
                &text[..text.len().min(50)]
            );
            match self
                .synthesize_text_with_options(text, voice, use_ssml)
                .await
            {
                Ok(audio_data) => results.push(audio_data),
                Err(e) => {
                    return Err(TTSError::Synthesis(format!(
                        "Failed to synthesize concurrent item {}: {}",
                        i + 1,
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Save multiple audio data to files
    pub async fn batch_save_audio(
        &self,
        audio_data_list: &[Vec<u8>],
        filename_template: &str,
    ) -> Result<Vec<String>, TTSError> {
        let mut saved_files = Vec::new();

        for (i, audio_data) in audio_data_list.iter().enumerate() {
            let filename = filename_template.replace("{}", &(i + 1).to_string());

            match self.save_audio(audio_data, &filename).await {
                Ok(_) => {
                    saved_files.push(filename.clone());
                    info!("Saved batch item {}: {}", i + 1, filename);
                }
                Err(e) => {
                    return Err(TTSError::Synthesis(format!(
                        "Failed to save batch item {}: {}",
                        i + 1,
                        e
                    )))
                }
            }
        }

        Ok(saved_files)
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
