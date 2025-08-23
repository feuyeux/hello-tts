"""Main TTS interface for Hello TTS."""
import asyncio
import logging
from typing import Optional

from config_manager import ConfigManager, TTSConfig

logger = logging.getLogger(__name__)


class HelloTTS:
    """Main TTS interface that handles different backends."""

    def __init__(self, backend: Optional[str] = None, config: Optional[TTSConfig] = None):
        self.config = config or ConfigManager.load_config()

        # Try to import available backends
        EdgeClient = None
        GoogleClient = None

        try:
            from backends.edge.tts_client import TTSClient as EdgeClient
        except ImportError:
            logger.debug("Edge TTS backend not available")

        try:
            from backends.google.tts_client import TTSClient as GoogleClient
        except ImportError:
            logger.debug("Google TTS backend not available")

        # Determine backend to use
        if backend is None:
            if EdgeClient is not None:
                backend = 'edge'
            elif GoogleClient is not None:
                backend = 'google'
            else:
                raise RuntimeError('No TTS backends available')

        self.backend_name = backend

        # Initialize the client
        if self.backend_name == 'edge' and EdgeClient is not None:
            self._client = EdgeClient(self.config)
        elif self.backend_name == 'google' and GoogleClient is not None:
            self._client = GoogleClient(self.config)
        else:
            raise RuntimeError(
                f"Requested backend '{self.backend_name}' not available")

    def synthesize_text(self, text: str, voice: Optional[str] = None) -> bytes:
        """Synthesize text to audio."""
        if voice is None:
            # Select appropriate default voice based on backend
            if self.backend_name == 'edge':
                voice = self.config.default_voice
            elif self.backend_name == 'google':
                voice = getattr(self.config, 'google_default_voice', 'en')
            else:
                voice = self.config.default_voice

        if self.backend_name == 'edge':
            return asyncio.run(self._client.synthesize_text(text, voice))
        else:
            return self._client.synthesize_text(text, voice)

    def save_audio(self, audio_data: bytes, filename: str) -> None:
        """Save audio data to file."""
        if self.backend_name == 'edge':
            asyncio.run(self._client.save_audio(audio_data, filename))
        else:
            self._client.save_audio(audio_data, filename)

    def list_voices(self):
        """List available voices."""
        if self.backend_name == 'edge':
            return asyncio.run(self._client.list_voices())
        else:
            return self._client.list_voices()
