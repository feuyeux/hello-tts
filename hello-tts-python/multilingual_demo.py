#!/usr/bin/env python3
"""
Multilingual batch demo for Edge TTS Python implementation.
Generates audio files for 12 languages with specified sentences.
"""

import asyncio
import json
import os
import time
from pathlib import Path

import sys
# Add the parent directory to Python path to import local modules
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

# Dynamically import TTSClient and TTSError based on backend
def get_tts_client_and_error(backend):
    if backend == 'google':
        from backends.google.tts_client import TTSClient, TTSError
    else:
        from backends.edge.tts_client import TTSClient, TTSError
    return TTSClient, TTSError

from audio_player import AudioPlayer, AudioError
from utils import create_output_directory


async def load_language_config():
    """Load language configuration from shared config file."""
    # Determine backend: prefer TTS_BACKEND env var, fallback to --backend arg if provided
    backend = os.environ.get('TTS_BACKEND')
    if not backend:
        # simple argv parsing
        for i, a in enumerate(sys.argv):
            if a == '--backend' and i + 1 < len(sys.argv):
                backend = sys.argv[i + 1]
                break
    if not backend:
        backend = 'edge'

    # Prefer unified tts_config.json
    config_path = "../shared/tts_config.json"
    if not os.path.exists(os.path.join(os.path.dirname(__file__), '..', 'shared', 'tts_config.json')):
        # fallback to previous per-backend files
        if backend.lower() == 'google':
            config_path = "../shared/google_tts_voices.json"
        else:
            config_path = "../shared/edge_tts_voices.json"

    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config_json = json.load(f)
            # Normalize into {'languages': [...]}
            langs = []
            for v in config_json.get('languages', []) or config_json.get('voices', []):
                code = v.get('code')
                name = v.get('name', '')
                flag = v.get('flag', '')
                text = v.get('text', '')
                # support unified keys edge_voice/google_voice, or legacy 'voice'
                edge_voice = v.get('edge_voice') or v.get('voice')
                google_voice = v.get('google_voice') or v.get('voice')
                langs.append({
                    'code': code,
                    'name': name,
                    'flag': flag,
                    'text': text or f"Hello, this is a {backend} TTS demo in {name}.",
                    'edge_voice': edge_voice,
                    'google_voice': google_voice,
                    'alt_voice': v.get('alt_voice')
                })

            return {'languages': langs}
    except FileNotFoundError:
        print(f"Configuration file not found: {config_path}")
        return None
    except json.JSONDecodeError as e:
        print(f"Error parsing configuration file: {e}")
        return None


async def generate_audio_for_language(client, language_config, output_dir, play_audio=False):
    """Generate audio for a single language configuration."""

    lang_code = language_config['code']
    lang_name = language_config['name']
    flag = language_config['flag']
    text = language_config['text']
    # Determine voice based on backend
    backend = os.environ.get('TTS_BACKEND', 'edge')
    if backend.lower() == 'google':
        voice = language_config.get('google_voice') or language_config.get('voice')
        alt_voice = language_config.get('alt_google') or language_config.get('alt_voice')
    else:
        voice = language_config.get('edge_voice') or language_config.get('voice')
        alt_voice = language_config.get('alt_voice')

    # 获取后端标识
    backend = os.environ.get('TTS_BACKEND', 'edge')
    print(f"\n{flag} {lang_name} ({lang_code.upper()})")
    print(f"Text: {text}")
    print(f"Voice: {voice}")


    # Dynamically import TTSError for the correct backend
    backend = os.environ.get('TTS_BACKEND', 'edge')
    TTSClient, TTSError = get_tts_client_and_error(backend)

    try:
        # Try primary voice first
        audio_data = None
        used_voice = voice

        import inspect
        try:
            synthesize = client.synthesize_text
            result = synthesize(text, voice)
            if inspect.isawaitable(result):
                if result is None:
                    print(f"❌ Synthesis failed for {lang_name}: No audio returned (unsupported language or error)")
                    return False
                audio_data = await result
            else:
                if result is None:
                    print(f"❌ Synthesis failed for {lang_name}: No audio returned (unsupported language or error)")
                    return False
                audio_data = result
        except TTSError as e:
            print(f"Primary voice failed: {e}")
            if alt_voice:
                print(f"Trying alternative voice: {alt_voice}")
                try:
                    result = synthesize(text, alt_voice)
                    if inspect.isawaitable(result):
                        if result is None:
                            print(f"❌ Synthesis failed for {lang_name} (alt voice): No audio returned (unsupported language or error)")
                            return False
                        audio_data = await result
                    else:
                        if result is None:
                            print(f"❌ Synthesis failed for {lang_name} (alt voice): No audio returned (unsupported language or error)")
                            return False
                        audio_data = result
                    used_voice = alt_voice
                except TTSError as e2:
                    print(f"Alternative voice also failed: {e2}")
                    raise e2
            else:
                raise e

        # Generate filename
        timestamp = int(time.time())
        lang_prefix = lang_code.split('-')[0]  # e.g., 'zh' from 'zh-cn'
        filename = f"{lang_prefix}_python_{backend}_{timestamp}.mp3"
        output_path = os.path.join(output_dir, filename)

        # Save audio (handle both async and sync save_audio methods)
        save_audio = client.save_audio
        if inspect.iscoroutinefunction(save_audio):
            await save_audio(audio_data, output_path)
        else:
            save_audio(audio_data, output_path)

        print(f"✅ Generated: {filename}")
        print(f"📁 Saved to: {output_path}")
        print(f"🎤 Used voice: {used_voice}")

        # Play audio if requested
        if play_audio:
            try:
                print("🔊 Playing audio...")
                player = AudioPlayer()
                player.play_file(output_path)
                print("✅ Playback completed")
            except AudioError as e:
                print(f"⚠️  Could not play audio: {e}")

        return True

    except Exception as e:
        print(f"❌ Failed to generate audio for {lang_name}: {e}")
        return False


async def main():
    """Main function for multilingual demo."""
    print("🌍 Multilingual Edge TTS Demo - Python Implementation")
    print("=" * 60)
    print("Generating audio for 12 languages with custom sentences...")
    
    # Load language configuration
    config = await load_language_config()
    if not config:
        print("❌ Failed to load language configuration")
        return 1
    
    languages = config.get('languages', [])
    if not languages:
        print("❌ No languages found in configuration")
        return 1
    
    print(f"📋 Found {len(languages)} languages to process")
    
    # Create output directory
    output_dir = "output"
    create_output_directory(output_dir)
    print(f"📁 Output directory: {os.path.abspath(output_dir)}")
    

    # Determine backend
    backend = os.environ.get('TTS_BACKEND')
    if not backend:
        # simple argv parsing
        for i, a in enumerate(sys.argv):
            if a == '--backend' and i + 1 < len(sys.argv):
                backend = sys.argv[i + 1]
                break
    if not backend:
        backend = 'edge'

    # Dynamically import TTSClient and TTSError
    TTSClient, TTSError = get_tts_client_and_error(backend)

    # Initialize TTS client
    try:
        client = TTSClient()
        print("✅ TTS client initialized")
    except Exception as e:
        print(f"❌ Failed to initialize TTS client: {e}")
        return 1
    
    # Process each language
    successful_count = 0
    failed_count = 0
    start_time = time.time()
    
    for i, language_config in enumerate(languages, 1):
        print(f"\n📍 Processing language {i}/{len(languages)}")
        
        success = await generate_audio_for_language(
            client, 
            language_config, 
            output_dir, 
            play_audio=False  # Set to True if you want to play each audio
        )
        
        if success:
            successful_count += 1
        else:
            failed_count += 1
        
        # Small delay between languages to be polite to the service
        if i < len(languages):
            print("⏳ Waiting before next language...")
            await asyncio.sleep(2)
    
    # Summary
    end_time = time.time()
    duration = end_time - start_time
    
    print(f"\n🏁 Processing Complete!")
    print("=" * 40)
    print(f"✅ Successful: {successful_count}")
    print(f"❌ Failed: {failed_count}")
    print(f"⏱️  Total time: {duration:.2f} seconds")
    print(f"📁 Output files saved in: {os.path.abspath(output_dir)}")
    
    if successful_count > 0:
        print(f"\n🎉 Successfully generated audio files for {successful_count} languages!")
        print("You can find all generated MP3 files in the output directory.")
    
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
