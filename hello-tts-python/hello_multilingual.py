#!/usr/bin/env python3
"""
Multilingual batch demo for TTS Python implementation.
Generates audio files for multiple languages with specified sentences.
"""

from audio_player import AudioPlayer, AudioError
from utils import create_output_directory
import asyncio
import json
import os
import time
import logging
import sys
from dataclasses import dataclass
from typing import Optional, Dict, List, Any

# Add the parent directory to Python path to import local modules
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

# Configure logging
logger = logging.getLogger('multilingual_demo')
handler = logging.StreamHandler()
handler.setFormatter(logging.Formatter('%(levelname)s: %(message)s'))
logger.addHandler(handler)
logger.setLevel(logging.INFO)

# Dynamically import TTSClient and TTSError based on backend


def get_tts_client_and_error(backend):
    if backend == 'google':
        from backends.google.tts_client import TTSClient, TTSError
    else:
        from backends.edge.tts_client import TTSClient, TTSError
    return TTSClient, TTSError


@dataclass
class LanguageConfig:
    """Language configuration structure"""
    code: str
    name: str
    flag: str
    text: str
    voice: str
    alt_voice: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any], backend: str) -> 'LanguageConfig':
        voice = data[f'{backend}_voice'] if f'{backend}_voice' in data else data.get(
            'voice', '')
        return cls(
            code=data.get('code', ''),
            name=data.get('name', ''),
            flag=data.get('flag', ''),
            text=data.get('text', ''),
            voice=voice,
            alt_voice=data.get('alt_voice')
        )


async def load_language_config(backend: str) -> List[LanguageConfig]:
    """Load language configuration from shared config file."""
    config_path = "../shared/tts_config.json"
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config_json = json.load(f)
            languages_data = config_json.get('languages', [])

            return [
                LanguageConfig.from_dict(lang_data, backend)
                for lang_data in languages_data
                if lang_data.get('code')  # Filter out entries without a code
            ]

    except FileNotFoundError:
        logger.error(f"❌ Configuration file not found: {config_path}")
        return []
    except json.JSONDecodeError as e:
        logger.error(f"❌ Error parsing configuration file: {e}")
        return []
    except Exception as e:
        logger.error(f"❌ Error loading configuration: {e}")
        return []


async def generate_audio_for_language(client, language_config: LanguageConfig, output_dir: str, backend: str, play_audio: bool = False) -> bool:
    """Generate audio for a single language configuration."""
    lang_code = language_config.code
    lang_name = language_config.name
    flag = language_config.flag
    text = language_config.text
    voice = language_config.voice
    alt_voice = language_config.alt_voice

    logger.info(f"\n{flag} {lang_name} ({lang_code.upper()})")
    logger.info(f"Text: {text}")
    logger.info(f"Voice: {voice}")

    try:
        # Try primary voice first
        audio_data = None
        used_voice = voice

        import inspect
        try:
            synthesize = client.synthesize_text
            result = synthesize(text, voice)

            # Handle both async and sync results
            if inspect.isawaitable(result):
                audio_data = await result
            else:
                audio_data = result

            if audio_data is None:
                logger.error(
                    f"❌ Synthesis failed for {lang_name}: No audio returned")
                return False

        except Exception as e:
            logger.warning(f"Primary voice failed: {e}")
            if alt_voice:
                logger.info(f"Trying alternative voice: {alt_voice}")
                try:
                    result = synthesize(text, alt_voice)

                    # Handle both async and sync results for alt voice
                    if inspect.isawaitable(result):
                        audio_data = await result
                    else:
                        audio_data = result

                    if audio_data is None:
                        logger.error(
                            f"❌ Synthesis failed for {lang_name} (alt voice): No audio returned")
                        return False

                    used_voice = alt_voice
                except Exception as e2:
                    logger.warning(f"Alternative voice also failed: {e2}")
                    raise e2
            else:
                raise e

        # Generate filename
        timestamp = int(time.time())
        lang_prefix = lang_code.split('-')[0]  # e.g., 'zh' from 'zh-cn'
        backend_name = 'gtts' if backend == 'google' else backend
        filename = f"{lang_prefix}_python_{backend_name}_{timestamp}.mp3"
        output_path = os.path.join(output_dir, filename)

        # Save audio (handle both async and sync save_audio methods)
        save_audio = client.save_audio
        if inspect.iscoroutinefunction(save_audio):
            await save_audio(audio_data, output_path)
        else:
            save_audio(audio_data, output_path)

        logger.info(f"✅ Generated: {filename}")
        logger.info(f"📁 Saved to: {os.path.abspath(output_path)}")
        logger.info(f"🎤 Used voice: {used_voice}")

        # Play audio if requested
        if play_audio:
            try:
                logger.info("🔊 Playing audio...")
                player = AudioPlayer()
                player.play_file(output_path)
                logger.info("✅ Playback completed")
            except AudioError as e:
                logger.warning(f"⚠️  Could not play audio: {e}")

        return True

    except Exception as e:
        logger.error(f"❌ Failed to generate audio for {lang_name}: {e}")
        return False


async def main() -> int:
    """Main function for multilingual demo."""
    logger.info("🌍 Multilingual TTS Demo - Python Implementation")
    logger.info("=" * 60)

    # Determine backend from environment or arguments
    backend = os.environ.get('TTS_BACKEND')
    if not backend:
        for i, arg in enumerate(sys.argv):
            if arg == '--backend' and i + 1 < len(sys.argv):
                backend = sys.argv[i + 1]
                break
    backend = backend or 'edge'

    # Load language configuration
    languages = await load_language_config(backend)
    if not languages:
        logger.error(
            "❌ Failed to load language configuration or no languages found")
        return 1

    logger.info(f"📋 Found {len(languages)} languages to process")

    # Create output directory
    output_dir = "output"
    create_output_directory(output_dir)
    logger.info(f"📁 Output directory: {os.path.abspath(output_dir)}")

    # Initialize TTS client
    try:
        TTSClient, TTSError = get_tts_client_and_error(backend)
        client = TTSClient()
        logger.info(f"✅ TTS client initialized with {backend} backend")
    except Exception as e:
        logger.error(f"❌ Failed to initialize TTS client: {e}")
        return 1

    # Process each language
    successful_count = 0
    failed_count = 0
    start_time = time.time()

    for i, language_config in enumerate(languages):
        logger.info(f"\n📍 Processing language {i + 1}/{len(languages)}")

        success = await generate_audio_for_language(
            client,
            language_config,
            output_dir,
            backend,
            play_audio=False  # Set to True if you want to play each audio
        )

        if success:
            successful_count += 1
        else:
            failed_count += 1

        # Small delay between languages to be polite to the service
        if i < len(languages) - 1:
            logger.info("⏳ Waiting before next language...")
            await asyncio.sleep(2)

    # Summary
    end_time = time.time()
    duration = end_time - start_time

    logger.info("\n🏁 Processing Complete!")
    logger.info("=" * 40)
    logger.info(f"✅ Successful: {successful_count}")
    logger.info(f"❌ Failed: {failed_count}")
    logger.info(f"⏱️  Total time: {duration:.2f} seconds")
    logger.info(f"📁 Output files saved in: {os.path.abspath(output_dir)}")

    if successful_count > 0:
        logger.info(
            f"\n🎉 Successfully generated audio files for {successful_count} languages!")
        logger.info(
            "You can find all generated MP3 files in the output directory.")

    return 0 if failed_count == 0 else 1


if __name__ == "__main__":
    try:
        exit_code = asyncio.run(main())
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n⏹️  Operation cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Unexpected error: {e}")
        sys.exit(1)
