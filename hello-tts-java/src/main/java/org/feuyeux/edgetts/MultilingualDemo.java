package org.feuyeux.edgetts;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import org.feuyeux.edgetts.audio.AudioPlayer;
import org.feuyeux.edgetts.tts.EdgeTtsProcessor;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Demonstrates multi-language text-to-speech capabilities using Edge TTS
 */
public class MultilingualDemo {

    // Active backend chosen via system property '-Dtts.backend'
    private static String ACTIVE_BACKEND = "edge";


    private static final String CONFIG_PATH = "../shared/edge_tts_voices.json";
    private static final Logger log = LoggerFactory.getLogger(MultilingualDemo.class);

    private static final String OUTPUT_DIR = "./";
    private static final boolean PLAY_AUDIO = false; // Set to true if you want to play each audio

    /**
     * Language configuration data structure
     */
    public static class LanguageConfig {
        public String code;
        public String name;
        public String flag;
        public String text;
        public String voice;
        public String altVoice;

        public LanguageConfig() {}

        public LanguageConfig(String code, String name, String flag, String text, String voice, String altVoice) {
            this.code = code;
            this.name = name;
            this.flag = flag;
            this.text = text;
            this.voice = voice;
            this.altVoice = altVoice;
        }

        @Override
        public String toString() {
            return String.format("%s %s (%s)", flag, name, code.toUpperCase());
        }
    }

    /**
     * Map an Edge-style voice name (e.g. "zh-CN-XiaoxiaoNeural") to a Google TTS language code
     */
    private String mapEdgeVoiceToGoogleLang(String edgeVoice) {
        if (edgeVoice == null || edgeVoice.isEmpty()) return "en";
        try {
            String[] parts = edgeVoice.split("-");
            if (parts.length >= 2) {
                String lang = parts[0].toLowerCase();
                String region = parts[1].toUpperCase();
                if ("zh".equals(lang)) {
                    if ("CN".equals(region)) return "zh-CN";
                    if ("TW".equals(region) || "HK".equals(region)) return "zh-TW";
                    return "zh";
                }
                // common case: return two-letter language code
                return lang;
            } else if (parts.length == 1) {
                return parts[0].toLowerCase();
            }
        } catch (Exception e) {
            // fallback
        }
        return "en";
    }

    public static void main(String[] args) {
        log.info("🌍 Multilingual Edge TTS Demo - Java Implementation");
        log.info("=".repeat(60));
        log.info("Generating audio for 12 languages with custom sentences...");

        try {
            // store active backend for use in generateAudioForLanguage
            ACTIVE_BACKEND = System.getProperty("tts.backend", "edge");
            MultilingualDemo demo = new MultilingualDemo();
            demo.runDemo();
        } catch (Exception e) {
            log.error("❌ Demo failed: {}", e.getMessage(), e);
            System.exit(1);
        }
    }

    /**
     * Runs the complete multilingual demo
     */
    public void runDemo() throws Exception {
        // Load language configuration
        List<LanguageConfig> languages = loadLanguageConfig();
        if (languages.isEmpty()) {
            log.error("❌ Failed to load language configuration or no languages found");
            System.exit(1);
        }

        log.info("\uD83D\uDCCB Found {} languages to process", languages.size());

        // Create output directory (using current directory for Java implementation)
        File outputDir = new File(OUTPUT_DIR);
        String outputPath = outputDir.getAbsolutePath();
        log.info("\uD83D\uDCC1 Output directory: {}", outputPath);

        // Initialize TTS client. Allow selecting backend via system property '-Dtts.backend'
        String backend = System.getProperty("tts.backend", "edge");
        EdgeTtsProcessor client;
        try {
            client = new EdgeTtsProcessor(backend);
            log.info("✅ TTS client initialized (backend={})", backend);
        } catch (Exception e) {
            log.error("❌ Failed to initialize TTS client: {}", e.getMessage(), e);
            throw e;
        }

        // Initialize audio player if needed
        AudioPlayer player = null;
        if (PLAY_AUDIO) {
            player = new AudioPlayer();
        }

        // Process each language
        int successfulCount = 0;
        int failedCount = 0;
        long startTime = System.currentTimeMillis();

        for (int i = 0; i < languages.size(); i++) {
            LanguageConfig languageConfig = languages.get(i);
            log.info("\n\uD83D\uDCCD Processing language {}/{}", i + 1, languages.size());

            boolean success = generateAudioForLanguage(client, languageConfig, outputPath, player);

            if (success) {
                successfulCount++;
            } else {
                failedCount++;
            }

            // Small delay between languages to be polite to the service
            if (i < languages.size() - 1) {
                log.info("⏳ Waiting before next language...");
                Thread.sleep(2000);
            }
        }

        // Summary
        long endTime = System.currentTimeMillis();
        double duration = (endTime - startTime) / 1000.0;

        log.info("\n🏁 Processing Complete!");
        log.info("=".repeat(40));
        log.info("✅ Successful: {}", successfulCount);
        log.info("❌ Failed: {}", failedCount);
        log.info( "⏱️  Total time: {} seconds", duration);
        log.info("\uD83D\uDCC1 Output files saved in: {}", outputPath);

        if (successfulCount > 0) {
            log.info("\n\uD83C\uDF89 Successfully generated audio files for {} languages!", successfulCount);
            log.info("You can find all generated MP3 files in the output directory.");
        }

        if (failedCount > 0) {
            System.exit(1);
        }
    }

    /**
     * Load language configuration from JSON file
     */
    private List<LanguageConfig> loadLanguageConfig() {
        List<LanguageConfig> languages = new ArrayList<>();

        try {
            File configFile = new File(CONFIG_PATH);
            if (!configFile.exists()) {
                log.error("❌ Configuration file not found: " + CONFIG_PATH);
                return languages;
            }

            ObjectMapper mapper = new ObjectMapper();
            JsonNode root = mapper.readTree(configFile);
            JsonNode languagesNode = root.get("languages");

            if (languagesNode != null && languagesNode.isArray()) {
                for (JsonNode languageNode : languagesNode) {
                    LanguageConfig config = new LanguageConfig();
                    config.code = languageNode.get("code").asText();
                    config.name = languageNode.get("name").asText();
                    config.flag = languageNode.get("flag").asText();
                    config.text = languageNode.get("text").asText();
                    config.voice = languageNode.get("voice").asText();
                    JsonNode altVoiceNode = languageNode.get("alt_voice");
                    config.altVoice = altVoiceNode != null ? altVoiceNode.asText() : null;

                    languages.add(config);
                }
            }

        } catch (IOException e) {
            log.error("❌ Error loading configuration: {}", e.getMessage(), e);
        }

        return languages;
    }

    /**
     * Generate audio for a single language
     */
    private boolean generateAudioForLanguage(EdgeTtsProcessor client, LanguageConfig languageConfig,
                                             String outputDir, AudioPlayer player) {
        String langCode = languageConfig.code;
        String langName = languageConfig.name;
        String flag = languageConfig.flag;
        String text = languageConfig.text;
        String voice = languageConfig.voice;
        String altVoice = languageConfig.altVoice;

        log.info("\n{} {} ({})", flag, langName, langCode.toUpperCase());
        log.info("Text: {}", text);
        log.info("Voice: {}", voice);

        try {
            // Try primary voice first
            byte[] audioData = null;
            String usedVoice = voice; 
            String synthVoice = voice;
            if ("google".equalsIgnoreCase(ACTIVE_BACKEND)) {
                // map edge voice name to google language code
                synthVoice = mapEdgeVoiceToGoogleLang(voice);
                log.info("Mapped Edge voice '{}' -> Google lang '{}'", voice, synthVoice);
            }

            try {
                CompletableFuture<byte[]> audioFuture = client.synthesizeText(text, synthVoice);
                audioData = audioFuture.get();
            } catch (ExecutionException | InterruptedException e) {
                log.warn("Primary voice failed: {}", e.getMessage());
                if (altVoice != null && !altVoice.isEmpty()) {
                    log.info("Trying alternative voice: {}", altVoice);
                    try {
                        String altSynthVoice = altVoice;
                        if ("google".equalsIgnoreCase(ACTIVE_BACKEND)) {
                            altSynthVoice = mapEdgeVoiceToGoogleLang(altVoice);
                        }
                        CompletableFuture<byte[]> audioFuture = client.synthesizeText(text, altSynthVoice);
                        audioData = audioFuture.get();
                        usedVoice = altVoice; 
                    } catch (ExecutionException | InterruptedException e2) {
                        log.warn("Alternative voice also failed: {}", e2.getMessage());
                        throw e2;
                    }
                } else {
                    throw e;
                }
            }

            // Generate filename
            long timestamp = System.currentTimeMillis() / 1000;
            String langPrefix = langCode.split("-")[0]; // e.g., 'zh' from 'zh-cn'
            String backend = ACTIVE_BACKEND;
            String filename = langPrefix + "_java_" + backend + "_" + timestamp + ".mp3";
            String outputPath = new File(outputDir, filename).getPath();

            // Save audio
            CompletableFuture<Void> saveFuture = client.saveAudio(audioData, outputPath);
            saveFuture.get();

            log.info("✅ Generated: {}", filename);
            log.info("\uD83D\uDCC1 Saved to: {}", outputPath);
            // If google backend, display mapped google language instead of edge voice name
            if ("google".equalsIgnoreCase(ACTIVE_BACKEND)) {
                String googleUsed = mapEdgeVoiceToGoogleLang(usedVoice);
                log.info("🎤 Used voice (Google lang): {}", googleUsed);
            } else {
                log.info("🎤 Used voice: {}", usedVoice);
            }

            // Play audio if requested (uncomment the lines below if needed)
            /*
            if (PLAY_AUDIO && player != null) {
                try {
                    log.info("🔊 Playing audio...");
                    player.playFile(outputPath);
                    log.info("✅ Playback completed");
                } catch (Exception e) {
                    log.warn("⚠️  Could not play audio: " + e.getMessage());
                }
            }
            */

            return true;

        } catch (Exception e) {
            log.error("❌ Failed to generate audio for {}: {}", langName, e.getMessage(), e);
            return false;
        }
    }
}