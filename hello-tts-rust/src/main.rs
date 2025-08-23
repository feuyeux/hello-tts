use clap::{Parser, Subcommand};
use hello_tts_rust::prelude::*;
use std::path::PathBuf;
use log::{info, error};
use env_logger;


#[derive(Parser)]
#[command(name = "hello-tts-rust")]
#[command(about = "A Rust implementation supporting both Edge TTS and Google TTS")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert text to speech
    Speak {
        /// Text to convert to speech
        #[arg(short, long)]
        text: String,

        /// Voice to use for synthesis
        #[arg(short, long, default_value = "en-US-AriaNeural")]
        voice: String,

        /// TTS backend to use (edge or google)
        #[arg(short, long, default_value = "edge")]
        backend: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Play audio after synthesis
        #[arg(short, long, default_value = "true")]
        play: bool,
    },
    /// List available voices
    Voices {
        /// Filter by language code (e.g., 'en', 'fr', 'es')
        #[arg(short, long)]
        language: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Run basic demo
    Demo {
        /// Language for demo
        #[arg(short, long, default_value = "en")]
        language: String,
    },
}

// The async runtime and main entrypoint are set up at the bottom of this file

async fn handle_speak(
    text: String,
    voice: String,
    backend: String,
    output: Option<PathBuf>,
    play: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎤 Converting text to speech...");
    info!("Backend: {}", backend);
    info!("Text: {}", text);
    info!("Voice: {}", voice);

    let mut config = TTSConfig::default();
    config.backend = backend;
    let mut client = TTSProcessor::new(Some(config));

    // Verify the voice exists
    match client.list_voices().await {
        Ok(voices) => {
            if !voices.iter().any(|v| v.name == voice) {
                error!("❌ Voice '{}' not found!", voice);
                error!("💡 Use 'hello-edge-tts voices' to see available voices");
                return Ok(());
            }
        }
        Err(e) => {
            error!("❌ Failed to list voices: {}", e);
            return Ok(());
        }
    }

    // Attempt synthesis (demo uses external edge-tts command)
    match client.synthesize_text(&text, &voice).await {
        Ok(audio_data) => {
            let output_path = output.unwrap_or_else(|| {
                // Extract language from voice (e.g., 'en' from 'en-US-AriaNeural')
                let lang = voice.split('-').next().unwrap_or("unknown");
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                PathBuf::from(format!("edgetts_{}_rust_{}.mp3", lang, timestamp))
            });

            match client
                .save_audio(&audio_data, output_path.to_str().unwrap())
                .await
            {
                Ok(()) => {
                    info!("✅ Audio saved to: {}", output_path.display());

                    if play {
                        info!("🔊 Playing audio...");
                        match AudioPlayer::new() {
                            Ok(player) => {
                                if let Err(e) = player.play_file(output_path.to_str().unwrap()) {
                                    error!("❌ Failed to play audio: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("❌ Failed to create audio player: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to save audio: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ TTS synthesis failed: {}", e);
            error!("💡 This is a demo implementation. Full WebSocket support needed for actual synthesis.");
        }
    }

    Ok(())
}

async fn handle_voices(
    language: Option<String>,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎵 Fetching available voices...");

    let mut client = TTSProcessor::new(None);

    let voices = match language {
        Some(lang) => {
            info!("Filtering by language: {}", lang);
            client.get_voices_by_language(&lang).await?
        }
        None => client.list_voices().await?,
    };

    if voices.is_empty() {
        info!("No voices found for the specified criteria.");
        return Ok(());
    }

    info!("📋 Available voices ({} total):", voices.len());

    // If detailed, show full entries; otherwise, list summaries
    if detailed {
        for voice in &voices {
            info!("🎤 {}", voice.display_name);
            info!("   Name: {}", voice.name);
            info!("   Locale: {}", voice.locale);
            info!("   Language: {}", voice.language_code());
            info!("");
        }
    } else {
        // Group by language for better organization
        let mut by_language: std::collections::HashMap<String, Vec<Voice>> =
            std::collections::HashMap::new();

        for voice in voices {
            by_language
                .entry(voice.language_code().to_string())
                .or_insert_with(Vec::new)
                .push(voice);
        }

        for (lang, mut voices) in by_language {
            voices.sort_by(|a, b| a.display_name.cmp(&b.display_name));
            info!("\n🌍 {} ({} voices):", lang.to_uppercase(), voices.len());
            for voice in voices {
                info!("  • {} ({}) - {}", voice.display_name, voice.locale, voice.gender);
            }
        }
    }

    Ok(())
}

async fn handle_demo(language: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Running Hello Edge TTS Demo");
    info!("Language: {}", language);
    info!("{}", "=".repeat(40));

    let mut client = TTSProcessor::new(None);

    // Get voices for the specified language
    info!("1️⃣ Fetching voices for language '{}'...", language);
    let voices = client.get_voices_by_language(&language).await?;

    if voices.is_empty() {
        error!("❌ No voices found for language '{}'", language);
        error!("💡 Try 'hello-edge-tts voices' to see all available languages");
        return Ok(());
    }

    info!("✅ Found {} voice(s)", voices.len());

    // Show first few voices
    let display_count = std::cmp::min(3, voices.len());
    info!("\n2️⃣ Sample voices:");
    for (i, voice) in voices.iter().take(display_count).enumerate() {
        info!(
            "   {}. {} ({}) - {}",
            i + 1,
            voice.display_name,
            voice.locale,
            voice.gender
        );
    }

    // Demonstrate synthesis with first voice
    if let Some(first_voice) = voices.first() {
        info!(
            "\n3️⃣ Demonstrating synthesis with '{}'...",
            first_voice.display_name
        );

        let demo_texts = match language.as_str() {
            "en" => vec!["Hello, World!", "Welcome to Edge TTS with Rust!"],
            "es" => vec!["¡Hola, Mundo!", "¡Bienvenido a Edge TTS con Rust!"],
            "fr" => vec!["Bonjour, le Monde!", "Bienvenue à Edge TTS avec Rust!"],
            "de" => vec!["Hallo, Welt!", "Willkommen bei Edge TTS mit Rust!"],
            "ja" => vec!["こんにちは、世界！", "RustでEdge TTSへようこそ！"],
            "zh" => vec!["你好，世界！", "欢迎使用Rust的Edge TTS！"],
            _ => vec!["Hello, World!", "Welcome to Edge TTS with Rust!"],
        };

        for (i, text) in demo_texts.iter().enumerate() {
            info!("   📝 Text {}: {}", i + 1, text);

            match client.synthesize_text(text, &first_voice.name, None).await {
                Ok(_audio_data) => {
                    info!("   ✅ Synthesis successful (demo mode)");
                }
                Err(e) => {
                    error!("   ❌ Synthesis failed: {}", e);
                    error!(
                        "   💡 This is expected in demo mode - WebSocket implementation needed"
                    );
                }
            }
        }
    }

    info!("\n🎉 Demo completed!");
    info!("💡 Use 'hello-edge-tts speak --help' for synthesis options");
    info!("💡 Use 'hello-edge-tts voices --help' for voice listing options");

    Ok(())
}

fn setup_logger() {
    // Initialize env_logger with a default filter if RUST_LOG is not set
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Use a custom formatter to produce consistent logs across languages:
    // ISO timestamp, level, target (module), and message
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            let ts = chrono::Utc::now().to_rfc3339();
            writeln!(buf, "{} [{}] {}: {}", ts, record.level(), record.target(), record.args())
        })
        .init();
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let cli = Cli::parse();

    match cli.command {
        Commands::Speak {
            text,
            voice,
            backend,
            output,
            play,
        } => {
            handle_speak(text, voice, backend, output, play).await?;
        }
        Commands::Voices { language, detailed } => {
            handle_voices(language, detailed).await?;
        }
        Commands::Demo { language } => {
            handle_demo(language).await?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run())
    {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
