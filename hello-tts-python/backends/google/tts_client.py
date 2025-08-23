"""Google TTS client using gTTS (Google Text-to-Speech)."""
from config_manager import TTSConfig
from voice import Voice
import io
import sys
from pathlib import Path
from typing import List, Optional

# Add parent directories to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

try:
    from gtts import gTTS
    from gtts.lang import tts_langs
    GTTS_AVAILABLE = True
except ImportError:
    GTTS_AVAILABLE = False
    gTTS = None
    tts_langs = None


class TTSError(Exception):
    """TTS specific error."""
    pass


class TTSClient:
    """Google TTS client using gTTS."""

    def __init__(self, config: Optional[TTSConfig] = None):
        if not GTTS_AVAILABLE:
            raise TTSError(
                "gTTS is required for Google TTS backend. Install with: pip install gTTS")

        self.config = config or TTSConfig()
        self._voices_cache: Optional[List[Voice]] = None

    def synthesize_text(self, text: str, voice: Optional[str] = None) -> bytes:
        """Synthesize text to audio using gTTS."""
        lang = self._extract_lang(voice or 'en')

        # Get speed setting from config - gTTS only supports slow=True/False
        slow_speech = getattr(self.config, 'google_slow_speech', False)

        try:
            # Use gTTS with simple settings for best compatibility
            tts = gTTS(
                text=text,
                lang=lang,
                slow=slow_speech  # False=normal speed, True=slow speed
            )
            audio_io = io.BytesIO()
            tts.write_to_fp(audio_io)
            audio_io.seek(0)
            return audio_io.getvalue()
        except Exception as e:
            raise TTSError(f"Failed to synthesize text: {e}")

    def save_audio(self, audio_data: bytes, filename: str) -> None:
        """Save audio data to file."""
        try:
            with open(filename, 'wb') as f:
                f.write(audio_data)
        except Exception as e:
            raise TTSError(f"Failed to save audio: {e}")

    def list_voices(self) -> List[Voice]:
        """List available voices (gTTS languages)."""
        if self._voices_cache is None:
            try:
                langs = tts_langs()
                self._voices_cache = [
                    Voice(
                        name=f"{lang_code}-Standard",
                        display_name=f"{lang_name} (Standard)",
                        locale=lang_code,
                        gender="unknown"
                    )
                    for lang_code, lang_name in langs.items()
                ]
            except Exception as e:
                raise TTSError(f"Failed to list voices: {e}")
        return self._voices_cache

    def _extract_lang(self, voice: str) -> str:
        """Extract language code from voice name."""
        # For Google TTS, voice format might be 'en-US-Standard' or just 'en'
        # We need to extract the language part for gTTS
        parts = voice.split('-')
        if len(parts) >= 1:
            lang_code = parts[0]
            # gTTS uses specific language codes, validate it
            try:
                available_langs = tts_langs()
                if lang_code in available_langs:
                    return lang_code
                # Try with full locale if available
                if len(parts) >= 2:
                    full_locale = f"{parts[0]}-{parts[1]}".lower()
                    if full_locale in available_langs:
                        return full_locale
            except:
                pass

        # Default to English if we can't determine the language
        return 'en'
