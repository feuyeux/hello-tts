#!/usr/bin/env python3
"""
Hello TTS Command Line Interface.

A unified CLI for text-to-speech synthesis using different backends.
"""
from audio_player import AudioPlayer, AudioError
from utils import create_output_directory
from config_manager import ConfigManager
from tts import HelloTTS
import argparse
import asyncio
import logging
import sys
import time
from pathlib import Path

# Add current directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))


logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO,
                    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s')


def main():
    """Main entry point for the CLI."""
    parser = argparse.ArgumentParser(description='Hello TTS unified CLI')
    parser.add_argument(
        '--text', '-t', default='Hello, World!', help='Text to synthesize')
    parser.add_argument('--voice', '-v', default=None, help='Voice to use')
    parser.add_argument('--backend', '-b',
                        choices=['edge', 'google'], help='Backend to use')
    parser.add_argument('--output', '-o', help='Output filename')
    parser.add_argument('--no-play', action='store_true',
                        help="Don't play audio")
    parser.add_argument('--list-voices', action='store_true',
                        help='List voices and exit')
    parser.add_argument('--slow', action='store_true',
                        help='Use slow speech (Google TTS only)')

    args = parser.parse_args()

    try:
        cfg = ConfigManager.load_config()
        create_output_directory(cfg.output_directory)
        tts = HelloTTS(backend=args.backend, config=cfg)

        # List voices if requested
        if args.list_voices:
            voices = tts.list_voices()
            print(f'Found {len(voices)} voices')
            for v in voices[:50]:
                print(f"{v.name} - {v.locale} - {v.gender}")
            return

        output_file = args.output if args.output else f"{cfg.output_directory}/hello_tts_{args.voice.split('-')[0] if args.voice else 'en'}_{int(time.time())}.{cfg.output_format}"

        logger.info(f"Synthesizing text: {args.text}")

        # Synthesize and save audio
        audio = tts.synthesize_text(args.text, voice=args.voice)
        tts.save_audio(audio, output_file)
        logger.info(f"Saved audio to {output_file}")

        # Play audio if requested
        if not args.no_play and cfg.auto_play:
            try:
                player = AudioPlayer()
                player.play_file(output_file)
                logger.info("Audio playback completed")
            except AudioError as e:
                logger.warning(f"Playback failed: {e}")

    except KeyboardInterrupt:
        logger.info("Operation cancelled by user")
        sys.exit(1)
    except Exception as e:
        logger.error(f"An error occurred: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
