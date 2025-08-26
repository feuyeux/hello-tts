use crate::models::{TTSError, Voice};
use async_trait::async_trait;

#[async_trait]
pub trait TTSBackend {
    async fn synthesize_text(&self, text: &str, voice: &str) -> Result<Vec<u8>, TTSError>;
    async fn list_voices(&self) -> Result<Vec<Voice>, TTSError>;
    async fn save_audio(&self, audio_data: &[u8], filename: &str) -> Result<(), TTSError>;
}

pub mod edge;
pub mod google;
