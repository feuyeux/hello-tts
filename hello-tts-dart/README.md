# hello_tts_dart

This package consolidates the Microsoft Edge TTS and Google gTTS backends into a
single, easy-to-use Dart implementation. It provides a CLI and an importable API.

## 🚀 Quick Start

```bash
cd hello-tts-dart
dart pub get
```

## Usage

### CLI Examples

```bash
# Edge TTS examples
dart run bin/hello_tts.dart --backend edge --text "Hello World" --voice "en-US-JennyNeural"
dart run bin/hello_tts.dart --backend edge --text "你好世界" --voice "zh-CN-XiaoxiaoNeural"

# Google TTS examples
dart run bin/hello_tts.dart --backend google --text "Hello World" --voice "en"
dart run bin/hello_tts.dart --backend google --text "你好世界" --voice "zh"

# List available voices
dart run bin/hello_tts.dart --list-voices
dart run bin/hello_tts.dart --list-voices --language en

# Save to specific file
dart run bin/hello_tts.dart --backend edge --text "Hello" --voice "en-US-AriaNeural" --output hello.mp3

# Help
dart run bin/hello_tts.dart --help
```

```bash
dart run bin/hello_multilingual.dart --backend google
```

## References

- [Microsoft Edge TTS](https://github.com/rany2/edge-tts)
- [Google Text-to-Speech](https://github.com/pndurette/gTTS)
