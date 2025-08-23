//! Basic usage example for Hello Edge TTS
//!
//! This example demonstrates the core functionality of the Edge TTS client:
//! - Creating a TTS client
//! - Listing available voices
//! - Filtering voices by language
//! - Synthesizing text to speech (demo mode)
//! - Playing audio files
//!
//! Run this example with: cargo run --example hello_tts

use hello_edge_tts::prelude::*;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    crate::setup_logger();
    info!("🚀 Hello Edge TTS - Basic Usage Example");
    info!("{}", "=".repeat(50));

    // Step 1: Create TTS client with default configuration
    info!("\n1️⃣ Creating TTS client...");
    let mut client = TTSProcessor::new(None);
    info!("✅ TTS client created successfully");

    // Step 2: List all available voices
    info!("\n2️⃣ Fetching available voices...");
    match client.list_voices().await {
        Ok(voices) => {
            info!("✅ Found {} voices total", voices.len());

            // Show first 5 voices as examples
            info!("\n📋 Sample voices:");
            for (i, voice) in voices.iter().take(5).enumerate() {
                info!(
                    "   {}. {} ({}) - {}",
                    i + 1,
                    voice.display_name,
                    voice.locale,
                    voice.gender
                );
            }

            if voices.len() > 5 {
                info!("   ... and {} more voices", voices.len() - 5);
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch voices: {}", e);
            return Ok(());
        }
    }

    // Step 3: Filter voices by language
    info!("\n3️⃣ Filtering voices by language...");
    let languages = vec!["en", "es", "fr", "de", "ja"];

    for lang in languages {
        match client.get_voices_by_language(lang).await {
            Ok(lang_voices) => {
                if !lang_voices.is_empty() {
                    info!(
                        "🌍 {} voices for '{}': {}",
                        lang_voices.len(),
                        lang.to_uppercase(),
                        lang_voices
                            .iter()
                            .take(3)
                            .map(|v| v.display_name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if lang_voices.len() > 3 {
                        println!("     ... and {} more", lang_voices.len() - 3);
                    }
                }
            }
            Err(e) => {
                    error!("❌ Failed to get voices for {}: {}", lang, e);
            }
        }
    }

    // Step 4: Demonstrate text synthesis (demo mode)
    info!("\n4️⃣ Demonstrating text synthesis...");

    // Get English voices for demo
    match client.get_voices_by_language("en").await {
        Ok(en_voices) => {
            if let Some(voice) = en_voices.first() {
                info!("🎤 Using voice: {} ({})", voice.display_name, voice.name);

                let demo_texts = vec![
                    "Hello, World!",
                    "This is a demonstration of Edge TTS with Rust.",
                    "The quick brown fox jumps over the lazy dog.",
                ];

                for (i, text) in demo_texts.iter().enumerate() {
                    info!("\n   📝 Synthesizing text {}: \"{}\"", i + 1, text);

                    match client.synthesize_text(text, &voice.name, None).await {
                        Ok(audio_data) => {
                            info!(
                                "   ✅ Synthesis successful! Generated {} bytes of audio data",
                                audio_data.len()
                            );

                            // Save to file
                            let filename = format!("edgetts_example_{}_rust.mp3", i + 1);
                            match client.save_audio(&audio_data, &filename).await {
                                Ok(()) => {
                                    info!("   💾 Audio saved to: {}", filename);
                                }
                                Err(e) => {
                                    error!("   ❌ Failed to save audio: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("   ❌ Synthesis failed: {}", e);
                            info!("   💡 This is expected in demo mode - full WebSocket implementation needed");
                        }
                    }
                }
            } else {
                println!("❌ No English voices available for demo");
            }
        }
        Err(e) => {
                error!("❌ Failed to get English voices: {}", e);
        }
    }

    // Step 5: Demonstrate audio player functionality
    info!("\n5️⃣ Demonstrating audio player...");
    match AudioPlayer::new() {
        Ok(player) => {
            info!("✅ Audio player created successfully");
            info!("🔊 Current volume: {:.1}%", player.volume() * 100.0);

            // Demonstrate volume control
            player.set_volume(0.8);
            info!("🔧 Volume set to: {:.1}%", player.volume() * 100.0);

            // Note: We can't actually play audio in this demo since we don't have real audio files
            println!("💡 Audio player is ready to play files with player.play_file(filename)");
            println!("💡 Use player.play_audio_data(audio_bytes) to play raw audio data");
        }
        Err(e) => {
            error!("❌ Failed to create audio player: {}", e);
            info!("💡 This might happen if no audio devices are available");
        }
    }

    // Step 6: Demonstrate configuration
    info!("\n6️⃣ Demonstrating custom configuration...");
    let custom_config = TTSConfig {
        default_voice: "en-US-JennyNeural".to_string(),
        output_format: "wav".to_string(),
        output_directory: "./custom_output".to_string(),
        auto_play: false,
        cache_voices: true,
        max_retries: 5,
        timeout: std::time::Duration::from_secs(45),
        rate: "0%".to_string(),
        pitch: "0%".to_string(),
        volume: "100%".to_string(),
        ssml: false,
        batch_size: 5,
        max_concurrent: 3,
    };

    let _custom_client = TTSProcessor::new(Some(custom_config));
    info!("✅ Custom TTS client created with:");
    info!("   • Default voice: en-US-JennyNeural");
    info!("   • Output format: WAV");
    info!("   • Output directory: ./custom_output");
    info!("   • Auto-play: disabled");
    info!("   • Voice caching: enabled");
    info!("   • Max retries: 5");
    info!("   • Timeout: 45 seconds");

    // Step 7: Summary and next steps
    info!("\n🎉 Basic usage example completed!");
    info!("{}", "=".repeat(50));
    info!("📚 What you learned:");
    info!("   • How to create and configure a TTS client");
    info!("   • How to list and filter available voices");
    info!("   • How to synthesize text to speech (API structure)");
    info!("   • How to use the audio player for playback");
    info!("   • How to customize client configuration");

    info!("\n🚀 Next steps:");
    info!("   • Try the CLI: cargo run -- speak --text 'Hello' --voice en-US-AriaNeural");
    info!("   • List voices: cargo run -- voices --language en");
    info!("   • Run demo: cargo run -- demo --language en");
    info!("   • Implement WebSocket communication for actual TTS synthesis");

    info!("\n💡 Note: This example shows the API structure. Full Edge TTS synthesis");
    info!("   requires WebSocket implementation which is beyond this demo scope.");

    Ok(())
}
