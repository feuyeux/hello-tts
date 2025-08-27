use clap::Parser;
use env_logger;
use hello_tts_rust::prelude::*;
use log::{error, info, warn, LevelFilter};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "hello-tts-rust")]
#[command(about = "A Rust implementation supporting both Edge TTS and Google TTS")]
#[command(version = "0.1.0")]
struct Cli {
    /// Text to convert to speech
    #[arg(short, long)]
    text: Option<String>,

    /// Voice to use for synthesis
    #[arg(short, long, default_value = "en-US-AriaNeural")]
    voice: String,

    /// TTS backend to use (edge or google)
    #[arg(short, long, default_value = "edge")]
    backend: String,

    /// Output directory
    #[arg(short, long, default_value = "output")]
    output_dir: String,

    /// Don't play audio after synthesis (default is to play)
    #[arg(long)]
    noplay: bool,

    /// List available voices
    #[arg(short = 'l', long)]
    list_voices: bool,

    /// Filter voices by language
    #[arg(short = 'L', long)]
    language: Option<String>,

    /// Run basic demo
    #[arg(long)]
    demo: bool,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

/// Create output directory if it doesn't exist
fn create_output_directory(directory: &str) -> std::io::Result<()> {
    fs::create_dir_all(directory)
}

async fn handle_speak(
    text: String,
    voice: String,
    backend: String,
    output_dir: String,
    play: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎤 Converting text to speech...");
    info!("Backend: {}", backend);
    info!("Text: {}", text);
    info!("Voice: {}", voice);

    let mut config = TTSConfig::default();
    config.backend = backend.clone();
    let client = TTSProcessor::new(Some(config));

    create_output_directory(&output_dir)?;

    let lang = voice.split('-').next().unwrap_or("unknown");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let output_path = Path::new(&output_dir).join(format!(
        "{}_rust_{}_{}.mp3",
        lang,
        backend,
        timestamp
    ));

    match client.synthesize_and_play(&text, &voice, Some(&output_path), play).await {
        Ok(_) => {
            info!("✅ Synthesis complete. Audio saved to {:?}", output_path);
        }
        Err(e) => {
            error!("❌ Synthesis failed: {}", e);
        }
    }

    Ok(())
}

async fn display_voices_by_language(
    client: &mut TTSProcessor,
    filter_language: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.list_voices().await {
        Ok(voices) => {
            let mut voices_by_language: HashMap<String, Vec<&Voice>> = HashMap::new();
            for voice in &voices {
                voices_by_language
                                        .entry(voice.locale.clone())
                    .or_insert_with(Vec::new)
                    .push(voice);
            }

            if let Some(lang) = filter_language {
                if let Some(lang_voices) = voices_by_language.get(&lang) {
                    info!(
                        "{} Voices ({} voices):",
                        lang.to_uppercase(),
                        lang_voices.len()
                    );
                    for voice in lang_voices.iter().take(5) {
                        info!("  {} - {} ({})", voice.name, voice.display_name, voice.gender);
                    }
                    if lang_voices.len() > 5 {
                        info!("  ... and {} more voices", lang_voices.len() - 5);
                    }
                } else {
                    warn!("No voices found for language: {}", lang);
                }
            } else {
                info!("Available voices by language:");
                for (lang, lang_voices) in &voices_by_language {
                    info!("{} ({} voices):", lang.to_uppercase(), lang_voices.len());
                    for voice in lang_voices.iter().take(5) {
                        info!("  {} - {} ({})", voice.name, voice.display_name, voice.gender);
                    }
                    if lang_voices.len() > 5 {
                        info!("  ... and {} more voices", lang_voices.len() - 5);
                    }
                }
            }
        }
        Err(e) => {
            error!("Error displaying voices: {}", e);
        }
    }
    Ok(())
}

async fn run_demo(language: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running demo for language: {}", language);
    let text = match language {
        "en" => "Hello, this is a demo of the text-to-speech system.",
        "zh" => "你好，这是一个文本转语音系统的演示。",
        "ja" => "こんにちは、これはテキスト読み上げシステムのデモです。",
        _ => {
            error!("Unsupported language for demo: {}", language);
            return Ok(());
        }
    };
    let voice = match language {
        "en" => "en-US-AriaNeural",
        "zh" => "zh-CN-XiaoxiaoNeural",
        "ja" => "ja-JP-NanamiNeural",
        _ => "en-US-AriaNeural",
    };

    handle_speak(text.to_string(), voice.to_string(), "edge".to_string(), "output".to_string(), true).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let log_level = match cli.log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new().filter_level(log_level).init();

    let mut config = TTSConfig::default();
    config.backend = cli.backend.clone();
    let mut client = TTSProcessor::new(Some(config));

    if cli.list_voices {
        display_voices_by_language(&mut client, cli.language).await?;
    } else if cli.demo {
        let lang = cli.language.unwrap_or_else(|| "en".to_string());
        run_demo(&lang).await?;
    } else if let Some(text) = cli.text {
        handle_speak(text, cli.voice, cli.backend, cli.output_dir, !cli.noplay).await?;
    } else {
        warn!("No text provided. Use -t or --text to specify text to synthesize.");
        warn!("Or use --list-voices to see available voices, or --demo to run a demo.");
    }

    Ok(())
}
