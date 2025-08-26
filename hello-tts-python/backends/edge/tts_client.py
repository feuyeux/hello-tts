"""Edge TTS client implementation."""
from config_manager import TTSConfig
from voice import Voice
import logging
import edge_tts
import aiofiles
import os
from typing import List, Optional
import sys
from pathlib import Path

# Add parent directories to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))


logger = logging.getLogger(__name__)


class TTSError(Exception):
    pass


class TTSClient:
    def __init__(self, config: Optional[TTSConfig] = None):
        self.config = config or TTSConfig()
        self._voices_cache: Optional[List[Voice]] = None

    async def synthesize_text(self, text: str, voice: str) -> bytes:
        """Synthesize text to audio using Edge TTS."""
        communicate = edge_tts.Communicate(text, voice)
        audio_data = b""
        async for chunk in communicate.stream():
            if chunk["type"] == "audio":
                audio_data += chunk["data"]

        if not audio_data:
            raise TTSError("No audio data generated")

        return audio_data

    async def save_audio(self, audio_data: bytes, filename: str) -> None:
        async with aiofiles.open(filename, 'wb') as f:
            await f.write(audio_data)

    async def list_voices(self) -> List[Voice]:
        if self._voices_cache is None:
            # Try to load from config file first
            config_path = "shared/tts_config.json"
            if os.path.exists(config_path):
                try:
                    self._voices_cache = Voice.parse_voices_from_json_file(
                        config_path)
                    return self._voices_cache
                except Exception as e:
                    logger.warning(
                        f"Failed to load voices from config file: {e}")

            # Fallback to API
            voices_data = await edge_tts.list_voices()
            self._voices_cache = [
                Voice(
                    name=v["ShortName"],
                    display_name=v.get("FriendlyName", v.get("ShortName", "")),
                    locale=v.get("Locale", ""),
                    gender=v.get("Gender", "unknown")
                )
                for v in voices_data
            ]
        return self._voices_cache

    async def get_voices_by_language(self, language: str) -> List[Voice]:
        all_voices = await self.list_voices()
        return [v for v in all_voices if v.locale.startswith(language) or v.locale.split('-')[0] == language]

    def clear_voice_cache(self) -> None:
        self._voices_cache = None
