# Hello TTS

A multi-language TTS demonstration using **Microsoft Edge TTS** and **Google TTS** services, implemented in **Python**, **Dart**, **Rust**, and **Java**.

## 🌐 Supported Languages

1. 🇨🇳 Chinese: 这是中文语音合成技术的演示。
2. 🇺🇸 English: This is a demonstration of TTS in English.
3. 🇩🇪 German: Dies ist eine Demonstration der deutschen Sprachsynthese-Technologie.
4. 🇫🇷 French: Ceci est une démonstration de la technologie de synthèse vocale française.
5. 🇪🇸 Spanish: Esta es una demostración de la tecnología de síntesis de voz en español.
6. 🇮🇹 Italian: Questa è una dimostrazione della tecnologia di sintesi vocale italiana.
7. 🇷🇺 Russian: Это демонстрация технологии синтеза речи на русском языке.
8. 🇬🇷 Greek: Αυτή είναι μια επίδειξη της τεχνολογίας σύνθεσης ομιλίας στα ελληνικά.
9. 🇸🇦 Arabic: هذا عرض توضيحي لتقنية تحويل النص إلى كلام باللغة العربية.
10. 🇮🇳 Hindi: यह हिंदी भाषा में टेक्स्ट-टू-स्पीच तकनीक का प्रदर्शन है।
11. 🇯🇵 Japanese: これは日本語音声合成技術のデモンストレーションです。
12. 🇰🇷 Korean: 이것은 한국어 음성 합성 기술의 시연입니다。

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

#### Python

```bash
cd hello-tts-python
python3 -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python hello_tts.py --text
```

#### Dart

```bash
cd hello-tts-dart
dart pub get
dart run bin/hello_tts.dart
```

#### Java

```bash
cd hello-tts-java
mvn compile
chcp 65001; mvn exec:java "-Dexec.mainClass=org.feuyeux.tts.HelloTTS"
```

#### Rust

```bash
cd hello-tts-rust
cargo run --example hello_tts
```

## 📖 Documentation

For detailed setup and usage, see:

- [Python README](./hello-tts-python/README.md)
- [Dart README](./hello-tts-dart/README.md)
- [Java README](./hello-tts-java/README.md)
- [Rust README](./hello-tts-rust/README.md)
