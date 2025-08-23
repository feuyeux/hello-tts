#!/usr/bin/env python3
"""
语速比较测试脚本
Compare different speech speeds for Chinese TTS
"""
from utils import create_output_directory
from config_manager import ConfigManager
from tts import HelloTTS
import sys
import time
from pathlib import Path

# Add current directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))


def test_speech_speeds():
    """Test different speech speeds for Chinese text."""
    print("🎤 语速比较测试 - Chinese Speech Speed Comparison")
    print("=" * 60)

    text = "这是中文语音合成的语速测试，我们来比较不同后端和设置的语速效果。"

    config = ConfigManager.load_config()
    create_output_directory(config.output_directory)

    tests = [
        {
            'name': 'Google TTS - 正常速度',
            'backend': 'google',
            'voice': 'zh',
            'config_updates': {'google_slow_speech': False},
            'ssml': False,
            'text': text
        },
        {
            'name': 'Google TTS - 慢速',
            'backend': 'google',
            'voice': 'zh',
            'config_updates': {'google_slow_speech': True},
            'ssml': False,
            'text': text
        },
        {
            'name': 'Edge TTS - 正常速度',
            'backend': 'edge',
            'voice': 'zh-CN-XiaoxiaoNeural',
            'config_updates': {},
            'ssml': False,
            'text': text
        },
        {
            'name': 'Edge TTS - 快速 (SSML)',
            'backend': 'edge',
            'voice': 'zh-CN-XiaoxiaoNeural',
            'config_updates': {},
            'ssml': True,
            'text': f'<speak><prosody rate="fast">{text}</prosody></speak>'
        },
        {
            'name': 'Edge TTS - 超快速 (SSML)',
            'backend': 'edge',
            'voice': 'zh-CN-XiaoxiaoNeural',
            'config_updates': {},
            'ssml': True,
            'text': f'<speak><prosody rate="x-fast">{text}</prosody></speak>'
        }
    ]

    for i, test in enumerate(tests, 1):
        print(f"\n{i}. {test['name']}")
        print("-" * 40)

        try:
            # Update config
            test_config = ConfigManager.load_config()
            for key, value in test['config_updates'].items():
                setattr(test_config, key, value)

            # Initialize TTS
            tts = HelloTTS(backend=test['backend'], config=test_config)

            # Generate filename
            timestamp = int(time.time())
            safe_name = test['name'].replace(
                ' ', '_').replace('(', '').replace(')', '')
            filename = f"./output/speed_test_{i}_{safe_name}_{timestamp}.mp3"

            print(f"生成音频: {filename}")

            # Synthesize
            start_time = time.time()
            audio = tts.synthesize_text(
                test['text'], voice=test['voice'], use_ssml=test['ssml'])
            synthesis_time = time.time() - start_time

            # Save
            tts.save_audio(audio, filename)

            print(f"合成时间: {synthesis_time:.2f}秒")
            print(f"文件大小: {len(audio)} bytes")
            print("✅ 完成")

        except Exception as e:
            print(f"❌ 错误: {e}")

    print(f"\n🎯 测试完成！请播放 output/ 目录中的音频文件来比较语速效果。")
    print("推荐使用 Edge TTS 的 SSML 功能来获得最快的中文语速。")


if __name__ == '__main__':
    test_speech_speeds()
