use crate::models::{TTSError, Voice};
use crate::config::TTSConfigFile;
use crate::backends::TTSBackend;
use async_trait::async_trait;
use log::{debug, error, info};
use tokio::fs;

pub struct GoogleTTS;

impl GoogleTTS {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TTSBackend for GoogleTTS {
    async fn synthesize_text(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError> {
        use std::process::Stdio;
        use tokio::process::Command;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "gtts_output_{}.mp3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

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

        let mut cmd = Command::new("gtts-cli");
        cmd.args([
            text,
            "--lang",
            &lang_code,
            "--output",
            temp_file.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| TTSError::Synthesis(format!("Failed to execute gtts-cli: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TTSError::Synthesis(format!(
                "Google TTS failed: {}",
                stderr
            )));
        }

        if temp_file.exists() {
            let audio_data = fs::read(&temp_file)
                .await
                .map_err(|e| TTSError::Synthesis(format!("Failed to read audio file: {}", e)))?;
            let _ = fs::remove_file(&temp_file).await;
            Ok(audio_data)
        } else {
            Err(TTSError::Synthesis(
                "Audio file was not generated".to_string(),
            ))
        }
    }

    async fn list_voices(&self) -> Result<Vec<Voice>, TTSError> {
        use std::env;
        use std::path::Path;

        let config_path = "../../shared/tts_config.json";
        let cwd = env::current_dir().unwrap_or_else(|_| Path::new("").to_path_buf());
        info!("Loading voices from config file: {}", config_path);
        debug!("Current working directory: {}", cwd.display());
        let abs_path = if Path::new(config_path).is_absolute() {
            Path::new(config_path).to_path_buf()
        } else {
            cwd.join(config_path)
        };
        debug!("Resolved config file path: {}", abs_path.display());
        let content = tokio::fs::read_to_string(&abs_path).await.map_err(|e| {
            error!("Failed to read config file: {}", abs_path.display());
            TTSError::Config(format!("Failed to read config file {}: {}", abs_path.display(), e))
        })?;
        let config: TTSConfigFile = serde_json::from_str(&content)
            .map_err(|e| TTSError::Config(format!("Invalid JSON in config file: {}", e)))?;

        let voices = config
            .languages
            .iter()
            .filter_map(|lang| {
                lang.google_voice.as_ref().map(|voice_name| Voice {
                    name: voice_name.clone(),
                    display_name: lang.name.clone(),
                    locale: lang.code.clone(),
                    gender: "Unknown".to_string(),
                    description: None,
                })
            })
            .collect();
        Ok(voices)
    }

    async fn save_audio(&self, audio_data: &[u8], filename: &str) -> Result<(), TTSError> {
        fs::write(filename, audio_data).await.map_err(TTSError::Io)
    }
}
