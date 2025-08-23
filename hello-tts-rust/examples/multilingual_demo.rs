use hello_tts_rust::prelude::*;
use env_logger;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use log::{info, error};

#[derive(Debug, Deserialize, Serialize)]
struct LanguageConfig {
    code: String,
    name: String,
    flag: String,
    text: String,
    voice: String,
    alt_voice: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
    languages: Vec<LanguageConfig>,
}

/// Load language configuration from JSON file
fn load_language_config() -> Result<Vec<LanguageConfig>, Box<dyn std::error::Error>> {
    // Determine backend from env var TTS_BACKEND (or default to edge)
    let backend = std::env::var("TTS_BACKEND").unwrap_or_else(|_| "edge".to_string());
    // Prefer unified tts_config.json, fallback to per-backend files
    let unified = "../shared/tts_config.json";
    let per_backend = if backend.to_lowercase() == "google" {
        "../shared/google_tts_voices.json"
    } else {
        "../shared/edge_tts_voices.json"
    };

    // Try unified config first
    if let Ok(content) = fs::read_to_string(unified) {
        let raw: serde_json::Value = serde_json::from_str(&content)?;
        let mut languages: Vec<LanguageConfig> = Vec::new();
        if let Some(arr) = raw.get("languages").and_then(|v| v.as_array()) {
            for item in arr {
                let code = item.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if code.is_empty() { continue; }
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let flag = item.get("flag").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                // pick appropriate voice for backend
                let voice = if backend.to_lowercase() == "google" {
                    item.get("google_voice").and_then(|v| v.as_str()).or_else(|| item.get("voice").and_then(|v| v.as_str())).unwrap_or("")
                } else {
                    item.get("edge_voice").and_then(|v| v.as_str()).or_else(|| item.get("voice").and_then(|v| v.as_str())).unwrap_or("")
                }.to_string();
                let alt = item.get("alt_voice").and_then(|v| v.as_str()).map(|s| s.to_string());
                if voice.is_empty() { continue; }
                languages.push(LanguageConfig{ code, name, flag, text, voice, alt_voice: alt });
            }
        }
        return Ok(languages);
    }

    // Fallback to per-backend legacy file
    match fs::read_to_string(per_backend) {
        Ok(content) => {
            let config: Config = serde_json::from_str(&content)?;
            Ok(config.languages)
        }
        Err(e) => {
            error!("❌ Configuration file not found: {} or {}", unified, per_backend);
            Err(Box::new(e))
        }
    }
}

/// Generate audio for a single language
async fn generate_audio_for_language(
    client: &mut TTSProcessor,
    language_config: &LanguageConfig,
    output_dir: &str,
    play_audio: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let lang_code = &language_config.code;
    let lang_name = &language_config.name;
    let flag = &language_config.flag;
    let text = &language_config.text;
    let voice = &language_config.voice;
    let alt_voice = &language_config.alt_voice;

    info!("\n{} {} ({})", flag, lang_name, lang_code.to_uppercase());
    info!("Text: {}", text);
    info!("Voice: {}", voice);

    // Try primary voice first
    let mut used_voice = voice.clone();
    let audio_data = match client.synthesize_text_with_options(text, voice, None).await {
        Ok(data) => data,
        Err(e) => {
            error!("Primary voice failed: {}", e);
            if let Some(alt_voice_name) = alt_voice {
                info!("Trying alternative voice: {}", alt_voice_name);
                        match client.synthesize_text_with_options(text, alt_voice_name, None).await {
                    Ok(data) => {
                        used_voice = alt_voice_name.clone();
                        data
                    }
                    Err(e2) => {
                        error!("Alternative voice also failed: {}", e2);
                        return Ok(false);
                    }
                }
            } else {
                error!("❌ Failed to generate audio for {}: {}", lang_name, e);
                return Ok(false);
            }
        }
    };

    // Generate filename
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let lang_prefix = lang_code.split('-').next().unwrap_or("unknown");
    let backend = std::env::var("TTS_BACKEND").unwrap_or_else(|_| "edge".to_string());
    let filename = format!("{}_rust_{}_{}.mp3", lang_prefix, backend, timestamp);
    let output_path = PathBuf::from(output_dir).join(&filename);

    // Save audio
    match client.save_audio(&audio_data, output_path.to_str().unwrap()).await {
            Ok(_) => {
                info!("✅ Generated: {}", filename);
                info!("📁 Saved to: {}", output_path.display());
                info!("🎤 Used voice: {}", used_voice);

            // Play audio if requested
            if play_audio {
                match AudioPlayer::new() {
                    Ok(player) => {
                        info!("🔊 Playing audio...");
                        match player.play_file(output_path.to_str().unwrap()) {
                            Ok(_) => info!("✅ Playback completed"),
                            Err(e) => error!("⚠️  Could not play audio: {}", e),
                        }
                    }
                    Err(e) => println!("⚠️  Could not create audio player: {}", e),
                }
            }

            Ok(true)
        }
        Err(e) => {
            error!("❌ Failed to save audio for {}: {}", lang_name, e);
            Ok(false)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("🌍 Multilingual Edge TTS Demo - Rust Implementation");
    info!("{}", "=".repeat(60));
    info!("Generating audio for 12 languages with custom sentences...");

    // Load language configuration
    let languages = match load_language_config() {
        Ok(langs) => langs,
        Err(e) => {
            error!("❌ Failed to load language configuration: {}", e);
            std::process::exit(1);
        }
    };

    if languages.is_empty() {
        error!("❌ No languages found in configuration");
        std::process::exit(1);
    }

    info!("📋 Found {} languages to process", languages.len());

    // Create output directory
    let output_dir = "./";  // Using current directory for Rust implementation
    let output_path = std::fs::canonicalize(output_dir)?;
    info!("📁 Output directory: {}", output_path.display());

    // Initialize TTS client
    let mut client = TTSProcessor::new(None);
    info!("✅ TTS client initialized");

    // Process each language
    let mut successful_count = 0;
    let mut failed_count = 0;
    let start_time = std::time::Instant::now();

    for (i, language_config) in languages.iter().enumerate() {
    info!("\n📍 Processing language {}/{}", i + 1, languages.len());

        match generate_audio_for_language(&mut client, language_config, output_dir, false).await {
            Ok(success) => {
                if success {
                    successful_count += 1;
                } else {
                    failed_count += 1;
                }
            }
                Err(e) => {
                error!("❌ Error processing {}: {}", language_config.name, e);
                failed_count += 1;
            }
        }

        // Small delay between languages to be polite to the service
        if i < languages.len() - 1 {
            info!("⏳ Waiting before next language...");
            sleep(Duration::from_secs(2)).await;
        }
    }

    // Summary
    let duration = start_time.elapsed();

    info!("\n🏁 Processing Complete!");
    info!("{}", "=".repeat(40));
    info!("✅ Successful: {}", successful_count);
    info!("❌ Failed: {}", failed_count);
    info!("⏱️  Total time: {:.2} seconds", duration.as_secs_f64());
    info!("📁 Output files saved in: {}", output_path.display());

    if successful_count > 0 {
    info!("\n🎉 Successfully generated audio files for {} languages!", successful_count);
    info!("You can find all generated MP3 files in the output directory.");
    }

    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}