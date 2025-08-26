use crate::models::{TTSError, Voice};
use crate::backends::TTSBackend;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tokio::fs;

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

pub struct EdgeTTS {
    client: Client,
}

impl EdgeTTS {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl TTSBackend for EdgeTTS {
    async fn synthesize_text(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError> {
        use std::process::Stdio;
        use tokio::process::Command;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "tts_output_{}.mp3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

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
        let url = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TTSError::Network)?;
        
        let voices: Vec<EdgeVoiceData> = response.json().await.map_err(TTSError::Network)?;
        
        Ok(voices
            .into_iter()
            .map(|v| Voice {
                name: v.short_name,
                display_name: v.friendly_name,
                locale: v.locale,
                gender: v.gender,
                description: None,
            })
            .collect())
    }

    async fn save_audio(&self, audio_data: &[u8], filename: &str) -> Result<(), TTSError> {
        fs::write(filename, audio_data).await.map_err(TTSError::Io)
    }
}
