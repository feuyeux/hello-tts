# Hello TTS

A multi-language TTS demonstration using **Microsoft Edge TTS** and **Google TTS** services, implemented in **Python**, **Dart**, **Rust**, and **Java**.

## 🌐 Supported Languages

1. 🇨🇳 Chinese: 你好，我可以为你朗读。
2. 🇺🇸 English: Hello, I can read for you.
3. 🇩🇪 German: Hallo, ich kann es für dich vorlesen.
4. 🇫🇷 French: Bonjour, je peux vous lire ce texte.
5. 🇪🇸 Spanish: Hola, puedo leer esto para ti.
6. 🇮🇹 Italian: Ciao, posso leggerlo per te.
7. 🇷🇺 Russian: Здравствуйте, я могу прочитать это для вас.
8. 🇬🇷 Greek: Γεια σας, μπορώ να το διαβάσω για εσάς.
9. 🇸🇦 Arabic: مرحبًا، أستطيع قراءة هذا لك.
10. 🇮🇳 Hindi: नमस्ते, मैं आपके लिए इसे पढ़ कर सुना सकता हूँ.
11. 🇯🇵 Japanese: こんにちは、これを読み上げることができます。
12. 🇰🇷 Korean: 안녕하세요, 제가 읽어 드릴 수 있습니다。

## ✨ Features

- 🌍 **Multilingual Support**: 400+ voices across 140+ languages
- 🔄 **Multiple Backends**: Microsoft Edge TTS and Google TTS
- 🚀 **Multiple Implementations**: Python, Dart, Java, and Rust
- 📁 **Audio Formats**: MP3, WAV, OGG support
- ⚡ **Performance**: Concurrent processing and optimizations

## 🚀 Quick Start

### Requirements

- Python 3.7+
- Dart SDK 2.17+
- Java 21 LTS
- Rust 1.60+
- Internet connection

### Run Examples

```bash
# Using Edge TTS (default)
./run.sh --language english --backend edge

# Using Google TTS
./run.sh --language chinese --backend google
```

### Language-specific Setup

#### Python

```bash
cd hello-tts-python
python3 -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python hello_tts.py --text "Hello" --voice "en-US-JennyNeural"
```

#### Dart

```bash
cd hello-tts-dart
dart pub get
dart run bin/hello_tts.dart --text "Hello World" --voice "en-US-JennyNeural"
```

#### Java

```bash
cd hello-tts-java
mvn compile
mvn exec:java -Dexec.mainClass="org.feuyeux.tts.HelloTTS"
```

#### Rust

```bash
cd hello-tts-rust
cargo run --example hello_tts -- --backend edge --text "Hello World"
```

## 📖 Documentation

For detailed setup and usage, see:

- [Python README](./hello-tts-python/README.md)
- [Dart README](./hello-tts-dart/README.md)
- [Java README](./hello-tts-java/README.md)
- [Rust README](./hello-tts-rust/README.md)
