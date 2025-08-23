//! Hello TTS - Rust implementation
//!
//! This crate provides a Rust client for both Microsoft Edge TTS and Google TTS services,
//! demonstrating text-to-speech functionality with audio playback capabilities.

pub mod audio_player;
pub mod config_manager;
pub mod tts_client;

pub use audio_player::{AudioError, AudioPlayer};
pub use config_manager::{
    create_default_config, get_preset, list_presets, load_config, ConfigManager,
};
pub use tts_client::{TTSProcessor, TTSConfig, TTSError, Voice};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
    create_default_config, get_preset, list_presets, load_config, AudioError, AudioPlayer,
    ConfigManager, TTSProcessor, TTSConfig, TTSError,
        Voice,
    };
}
