use hello_tts_rust::prelude::*;
use clap::Parser;
use env_logger;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use log::{info, error, LevelFilter};

#[derive(Parser)]
#[command(name = "hello-tts-multilingual")]
#[command(about = "Multilingual TTS demo supporting both Edge TTS and Google TTS")]
struct Cli {
    /// TTS backend to use (edge or google)
    #[arg(short, long, default_value = "edge")]
    backend: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LanguageConfig {
    code: String,
    name: String,
    flag: String,
    text: String,
    edge_voice: String,
    google_voice: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    languages: Vec<LanguageConfig>,
}

/// Load language configuration from JSON file
fn load_language_config() -> Result<Vec<LanguageConfig>, TTSError> {
    let config_path = "../shared/tts_config.json";
    
    let config_str = fs::read_to_string(config_path)
        .map_err(|e| TTSError::Config(format!(
            "Failed to read config file {}: {}", config_path, e
        )))?;

    let config: Config = serde_json::from_str(&config_str)
        .map_err(|e| TTSError::Config(format!(
            "Invalid JSON in config file: {}", e
        )))?;

    Ok(config.languages)
}

/// Generate audio for a single language
async fn generate_audio_for_language(
    client: &mut TTSProcessor,
    language_config: &LanguageConfig,
    output_dir: &str,
    play_audio: bool,
    backend: &str,
) -> Result<bool, TTSError> {
    let voice = if backend == "google" {
        &language_config.google_voice
    } else {
        &language_config.edge_voice
    };

    info!("🗣️  {} ({})", language_config.name, language_config.flag);
    info!("📝 Text: {}", language_config.text);
    info!("🎙️  Voice: {}", voice);

    // Generate timestamp for unique filenames
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let lang_code = language_config.code.split('-').next().unwrap_or("unknown");

    let output_path = PathBuf::from(output_dir)
        .join(format!("{}_rust_{}_{}.mp3", lang_code, backend, timestamp));

    match client.synthesize_and_play(&language_config.text, voice, Some(&output_path), play_audio).await {
        Ok(_) => {
            info!("✅ Generated audio saved to {:?}", output_path);
            Ok(true)
        }
        Err(e) => {
            error!("❌ Failed to generate audio: {}", e);
            Ok(false)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with debug level
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    // Parse command line arguments
    let cli = Cli::parse();
    let backend = cli.backend;

    info!("🌍 Multilingual TTS Demo - Rust Implementation");
    info!("Backend: {}", backend);
    info!("{}", "=".repeat(60));

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
    let output_dir = "output";
    fs::create_dir_all(output_dir)?;
    let output_path = std::fs::canonicalize(output_dir)?;
    info!("📁 Output directory: {}", output_path.display());

    // Initialize TTS client with backend configuration
    let mut config = TTSConfig::default();
    config.backend = backend.clone();
    let mut client = TTSProcessor::new(Some(config));
    info!("✅ TTS client initialized with {} backend", backend);

    // Process each language
    let mut successful_count = 0;
    let mut failed_count = 0;
    let start_time = std::time::Instant::now();

    for (i, language_config) in languages.iter().enumerate() {
        info!("📍 Processing language {}/{}", i + 1, languages.len());

        match generate_audio_for_language(&mut client, language_config, output_dir, false, &backend).await {
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

    info!("🏁 Processing Complete!");
    info!("{}", "=".repeat(40));
    info!("✅ Successful: {}", successful_count);
    info!("❌ Failed: {}", failed_count);
    info!("⏱️ Total time: {:.2} seconds", duration.as_secs_f64());
    info!("📁 Output files saved in: {}", output_path.display());

    if successful_count > 0 {
        info!("🎉 Successfully generated audio files for {} languages!", successful_count);
        info!("Generated MP3 files are in the output directory.");
    }

    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
