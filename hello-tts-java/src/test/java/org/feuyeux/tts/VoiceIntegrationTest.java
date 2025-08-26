package org.feuyeux.tts;

import org.feuyeux.tts.tts.Voice;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Integration tests for the Voice class using the actual shared voice configuration file.
 */
public class VoiceIntegrationTest {

    @Test
    void testParseVoicesFromSharedConfigFile() throws Exception {
        // Test parsing the actual shared voice configuration file
        String configPath = "../shared/tts_config.json";
        
        List<Voice> voices = Voice.parseVoicesFromJsonFile(configPath);
        
        // Verify we got some voices
        assertFalse(voices.isEmpty(), "Should parse at least some voices from the config file");
        
        // Verify we have expected voices
        boolean hasAria = voices.stream().anyMatch(v -> v.name().equals("en-US-AriaNeural"));
        assertTrue(hasAria, "Should contain Aria voice");
        
        boolean hasXiaoxiao = voices.stream().anyMatch(v -> v.name().equals("zh-CN-XiaoxiaoNeural"));
        assertTrue(hasXiaoxiao, "Should contain Xiaoxiao voice");
        
        // Test filtering by language
        List<Voice> englishVoices = Voice.getVoicesByLanguage(voices, "en")
                .collect(Collectors.toList());
        assertFalse(englishVoices.isEmpty(), "Should have English voices");
        
        List<Voice> spanishVoices = Voice.getVoicesByLanguage(voices, "es")
                .collect(Collectors.toList());
        assertFalse(spanishVoices.isEmpty(), "Should have Spanish voices");
        
        // Test filtering by locale
        List<Voice> usVoices = Voice.getVoicesByLocale(voices, "en-us")
                .collect(Collectors.toList());
        assertFalse(usVoices.isEmpty(), "Should have US English voices");
        
        // Test filtering by gender (all voices have "Unknown" gender in config)
        List<Voice> unknownGenderVoices = Voice.getVoicesByGender(voices, "Unknown")
                .collect(Collectors.toList());
        assertFalse(unknownGenderVoices.isEmpty(), "Should have unknown gender voices");
        
        // Print some information for verification
        System.out.println("Total voices parsed: " + voices.size());
        System.out.println("English voices: " + englishVoices.size());
        System.out.println("Spanish voices: " + spanishVoices.size());
        System.out.println("Unknown gender voices: " + unknownGenderVoices.size());
        
        // Print first few voices for verification
        System.out.println("\nFirst few voices:");
        voices.stream().limit(5).forEach(voice -> 
            System.out.println("  " + voice.toDisplayString()));
    }
    
    @Test
    void testVoiceDisplayStrings() throws Exception {
        String configPath = "../shared/tts_config.json";
        List<Voice> voices = Voice.parseVoicesFromJsonFile(configPath);
        
        // Test that all voices have proper display strings
        for (Voice voice : voices) {
            String displayString = voice.toDisplayString();
            assertNotNull(displayString);
            assertFalse(displayString.trim().isEmpty());
            
            // Should contain display name, locale, and gender
            assertTrue(displayString.contains(voice.displayName()));
            assertTrue(displayString.contains(voice.locale()));
            assertTrue(displayString.contains(voice.gender()));
        }
    }
}