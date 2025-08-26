# Rust TTS Implementation

This directory contains the Rust implementation of the hello-edge-tts project using `reqwest` and `tokio` for async HTTP communication. This implementation showcases Rust's memory safety, performance, and robust error handling with the Result type system.

## 🚀 Quick Start

```bash
# Navigate to Rust directory
cd hello-edge-tts-rust
cargo build
# Edge TTS examples
cargo run --example hello_tts -- --backend edge --text "Hello World" --voice "en-US-JennyNeural"
cargo run --example hello_tts -- --backend edge --text "你好世界" --voice "zh-CN-XiaoxiaoNeural"
# Google TTS examples
cargo run --example hello_tts -- --backend google --text "Hello World" --voice "en"
cargo run --example hello_tts -- --backend google --text "你好世界" --voice "zh"

cargo run --example hello_tts -- --list-voices

cargo run --example hello_multilingual -- --backend edge
cargo run --example hello_multilingual -- --backend google
```
