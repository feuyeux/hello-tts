package org.feuyeux.tts;

import lombok.extern.slf4j.Slf4j;
import org.apache.commons.cli.*;
import org.feuyeux.tts.audio.AudioPlayer;
import org.feuyeux.tts.tts.TtsProcessor;
import org.feuyeux.tts.tts.Voice;

import java.io.File;
import java.io.IOException;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;

/**
 * Main class demonstrating Edge TTS functionality in Java
 * Provides command-line interface for text-to-speech conversion with voice selection and multi-language support
 */
@Slf4j
public class HelloTTS {

    private static final String DEFAULT_VOICE = "en-US-AriaNeural";
    private static final String VOICE_CONFIG_PATH = "../shared/tts_config.json";

    public static void main(String[] args) {
        log.info("=== Hello Edge TTS - Java实现启动 ===");
        final Options options = createCommandLineOptions();
        CommandLineParser parser = new DefaultParser();
        try {
            CommandLine cmd = parser.parse(options, args);

            if (cmd.hasOption("help")) {
                printHelp(options);
                return;
            }
            if (cmd.hasOption("list-voices")) {
                listAvailableVoices();
                return;
            }
            runTTSConversion(cmd);
        } catch (ParseException e) {
            log.error("命令行参数解析错误: {}", e.getMessage());
            printHelp(options);
            System.exit(1);
        } catch (Exception e) {
            log.error("应用程序执行错误: {}", e.getMessage(), e);
            System.exit(1);
        }
    }

    /**
     * Creates command line options for the application
     */
    private static Options createCommandLineOptions() {
        Options options = new Options();

        options.addOption(Option.builder("t")
                .longOpt("text")
                .hasArg()
                .desc("Text to convert to speech (default: 'Hello, World!')")
                .build());

        options.addOption(Option.builder("v")
                .longOpt("voice")
                .hasArg()
                .desc("Voice to use (default: " + DEFAULT_VOICE + ")")
                .build());

        options.addOption(Option.builder("o")
                .longOpt("output")
                .hasArg()
                .desc("Output filename (default: auto-generated)")
                .build());

        options.addOption(Option.builder("l")
                .longOpt("language")
                .hasArg()
                .desc("Filter voices by language code (e.g., 'en', 'es', 'fr')")
                .build());
        options.addOption(Option.builder("b")
                .longOpt("backend")
                .hasArg()
                .desc("Select TTS backend (default: edge)")
                .build());
        options.addOption(Option.builder()
                .longOpt("list-voices")
                .desc("List all available voices")
                .build());

        options.addOption(Option.builder()
                .longOpt("no-play")
                .desc("Don't play audio after generation")
                .build());

        options.addOption(Option.builder("h")
                .longOpt("help")
                .desc("Show this help message")
                .build());

        return options;
    }

    /**
     * Prints help information for command line usage
     */
    private static void printHelp(Options options) {
        HelpFormatter formatter = new HelpFormatter();
        formatter.printHelp("java -jar hello-edge-tts.jar [OPTIONS]",
                "\nJava implementation of Edge TTS demonstration\n\n",
                options,
                """                        
                        Examples:
                          java -jar hello-edge-tts.jar
                          java -jar hello-edge-tts.jar --text "Hello from Java!" --voice en-US-DavisNeural
                          java -jar hello-edge-tts.jar --list-voices
                          java -jar hello-edge-tts.jar --demo
                          java -jar hello-edge-tts.jar --language es --text "Hola mundo"
                        """);
    }

    /**
     * Lists all available voices from the configuration file
     */
    private static void listAvailableVoices() {
        log.info("=== Available Voices ===\n");
        try {
            List<Voice> voices = loadVoicesFromConfig();

            if (voices.isEmpty()) {
                log.info("No voices found in configuration file.");
                return;
            }
            // Group voices by language
            voices.stream()
                    .collect(Collectors.groupingBy(Voice::getLanguageCode))
                    .forEach((language, voiceList) -> {
                        log.info("Language: {}", language.toUpperCase());
                        log.info("─".repeat(40));
                        voiceList.forEach(voice -> log.info("  {} {} {} {}",
                                voice.name(),
                                voice.displayName(),
                                voice.gender(),
                                voice.description() != null ? voice.description() : ""));
                        log.info("");
                    });

        } catch (Exception e) {
            log.error("Error loading voices: {}", e.getMessage(), e);
        }
    }

    /**
     * Runs the main TTS conversion based on command line arguments
     */
    private static void runTTSConversion(CommandLine cmd) throws Exception {
        // Get parameters from command line or use defaults
        String text = cmd.getOptionValue("text", "Hello, World! This is a demonstration of TTS in Java.");
        String voice = cmd.getOptionValue("voice", DEFAULT_VOICE);
        String outputFile = cmd.getOptionValue("output");
        String languageFilter = cmd.getOptionValue("language");
        boolean shouldPlay = !cmd.hasOption("no-play");
        String backend = cmd.getOptionValue("backend", "edge").toLowerCase();
        if (languageFilter != null) {
            voice = findVoiceByLanguage(languageFilter, voice);
        }
        // Generate output filename if not specified
        if (outputFile == null) {
            outputFile = generateOutputFilename(backend,voice);
        }
        log.info("Text: {} | Voice: {} | Output: {}", text, voice, outputFile);
        TtsProcessor client = new TtsProcessor(backend);
        try {
            log.info("Converting text to speech...");
            CompletableFuture<byte[]> audioFuture = client.synthesizeText(text, voice);
            CompletableFuture<Void> saveFuture = client.saveAudio(audioFuture, outputFile);
            saveFuture.get();
            log.info("✓ Audio saved to: {}", outputFile);
            // Play the audio if requested
            if (shouldPlay) {
                log.info("Playing audio...");
                AudioPlayer player = new AudioPlayer();
                player.playFile(outputFile);
                log.info("✓ Playback completed!");
            }

        } catch (ExecutionException | InterruptedException e) {
            log.error("✗ TTS Error: {}", e.getMessage(), e);
            throw e;
        }
    }

    /**
     * Finds a voice by language code, falling back to default if not found
     */
    private static String findVoiceByLanguage(String languageCode, String fallbackVoice) {
        try {
            List<Voice> voices = loadVoicesFromConfig();
            List<Voice> matchingVoices = Voice.getVoicesByLanguage(voices, languageCode).toList();
            if (!matchingVoices.isEmpty()) {
                Voice selectedVoice = matchingVoices.getFirst();
                log.info("Found voice for language '{}': {}", languageCode, selectedVoice.name());
                return selectedVoice.name();
            } else {
                log.info("No voices found for language '{}', using: {}", languageCode, fallbackVoice);
                return fallbackVoice;
            }
        } catch (Exception e) {
            log.error("Error finding voice by language: {}", e.getMessage());
            return fallbackVoice;
        }
    }

    /**
     * Generates an output filename based on the voice name
     */
    private static String generateOutputFilename(String backend, String voice) {
        String lang = voice.split("-")[0];
        long timestamp = System.currentTimeMillis() / 1000;
        return "edge".equals(backend)?"edgetts_":"gtts_" + lang + "_java_" + timestamp + ".mp3";
    }

    /**
     * Loads voices from the shared configuration file
     */
    private static List<Voice> loadVoicesFromConfig() throws Exception {
        File configFile = new File(VOICE_CONFIG_PATH);
        if (!configFile.exists()) {
            throw new IOException("Voice configuration file not found: " + VOICE_CONFIG_PATH);
        }

        return Voice.parseVoicesFromJsonFile(VOICE_CONFIG_PATH);
    }

}
