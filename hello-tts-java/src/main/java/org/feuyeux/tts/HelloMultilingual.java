package org.feuyeux.tts;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

import org.feuyeux.tts.audio.AudioPlayer;
import org.feuyeux.tts.config.LanguageConfig;
import org.feuyeux.tts.tts.TtsProcessor;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Demonstrates multi-language text-to-speech capabilities using Edge TTS
 */
public class HelloMultilingual {

    // Active backend chosen via system property '-Dtts.backend'
    private static String ACTIVE_BACKEND = "edge";
    
    private static final String CONFIG_PATH = "../shared/tts_config.json";
    private static final Logger log = LoggerFactory.getLogger(HelloMultilingual.class);

    private static final boolean PLAY_AUDIO = false; // Set to true if you want to play each audio

    public static void main(String[] args) {
        log.info("🌍 Multilingual Edge TTS Demo - Java Implementation");
        log.info("=".repeat(60));
        log.info("Generating audio for 12 languages with custom sentences...");

        try {
            ACTIVE_BACKEND = System.getProperty("tts.backend", "edge");
            HelloMultilingual demo = new HelloMultilingual();
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

        // Initialize TTS client. Allow selecting backend via system property '-Dtts.backend'
        String backend = System.getProperty("tts.backend", "edge");
        TtsProcessor client;
        try {
            client = new TtsProcessor(backend);
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

            boolean success = generateAudioForLanguage(client, languageConfig, player);

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
                    config.setCode(languageNode.get("code").asText());
                    config.setName(languageNode.get("name").asText());
                    config.setFlag(languageNode.get("flag").asText());
                    config.setText(languageNode.get("text").asText());
                    config.setEdgeVoice(languageNode.get("edge_voice").asText());
                    config.setGoogleVoice(languageNode.get("google_voice").asText());
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
    private boolean generateAudioForLanguage(TtsProcessor client, LanguageConfig languageConfig,
                                             AudioPlayer player) {
        try {
 
       
                String synthVoice = ACTIVE_BACKEND.equals("edge")? languageConfig.getEdgeVoice() : languageConfig.getGoogleVoice();
                CompletableFuture<byte[]> audioFuture = client.synthesizeText(languageConfig.getText(),synthVoice );
 
     

            // Generate filename
            long timestamp = System.currentTimeMillis() / 1000;
            String langPrefix = languageConfig.getCode().split("-")[0]; // e.g., 'zh' from 'zh-cn'
            String backend = ACTIVE_BACKEND;
            String filename = langPrefix + "_java_" + backend + "_" + timestamp + ".mp3";
            String outputPath = new File(".", filename).getPath();

            // Save audio
            CompletableFuture<Void> saveFuture = client.saveAudio(audioFuture , outputPath);
            saveFuture.get();

            if (PLAY_AUDIO && player != null) {
                try {
                    log.info("🔊 Playing audio...");
                    player.playFile(outputPath);
                    log.info("✅ Playback completed");
                } catch (Exception e) {
                    log.warn("Could not play audio: {}", e.getMessage());
                }
            }

            return true;

        } catch (Exception e) {
            log.error("❌ Failed to generate audio for {}: {}", languageConfig.getName(), e.getMessage(), e);
            return false;
        }
    }
}