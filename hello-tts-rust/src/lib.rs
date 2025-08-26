//! Hello TTS - Rust implementation
//!
//! This crate provides a Rust client for both Microsoft Edge TTS and Google TTS services,
//! demonstrating text-to-speech functionality with audio playback capabilities.

pub mod audio_player;
pub mod backends;
pub mod config;
pub mod models;
pub mod tts_client;

pub use audio_player::{AudioError, AudioPlayer};
pub use config::{TTSConfig, TTSConfigFile};
pub use models::{TTSError, Voice};
pub use tts_client::TTSProcessor;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        AudioError, AudioPlayer,
        TTSProcessor, TTSConfig, TTSError, Voice, TTSConfigFile,
    };
    pub use crate::backends::TTSBackend;
}
