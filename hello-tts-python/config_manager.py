"""Configuration manager for Hello TTS."""
from dataclasses import dataclass
from typing import Optional


@dataclass
class TTSConfig:
    """Configuration for TTS settings."""
    default_voice: str = 'en-US-JennyNeural'  # For Edge TTS
    google_default_voice: str = 'en'  # For Google TTS
    # For Google TTS speed control (True=slow, False=normal)
    google_slow_speech: bool = False
    output_format: str = 'mp3'
    output_directory: str = './output'
    auto_play: bool = True
    cache_voices: bool = True
    max_retries: int = 3
    timeout: int = 30000


class ConfigManager:
    """Manages TTS configuration."""

    @classmethod
    def load_config(cls, path: Optional[str] = None) -> TTSConfig:
        """Load configuration from file or return default config."""
        # For now, return default config
        # TODO: Add file-based config loading if needed
        return TTSConfig()
