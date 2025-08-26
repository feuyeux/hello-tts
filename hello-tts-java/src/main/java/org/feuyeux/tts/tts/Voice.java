package org.feuyeux.tts.tts;

import lombok.extern.slf4j.Slf4j;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.stream.Stream;

/**
 * Immutable Voice class representing a text-to-speech voice with its properties.
 * This class provides voice information including name, display name, locale, and gender.
 */
@Slf4j
public record Voice(String name, String displayName, String locale, String gender, String description) {
    /**
     * Creates a new Voice instance.
     *
     * @param name        The technical name of the voice (e.g., "en-US-AriaNeural")
     * @param displayName The human-readable display name (e.g., "Aria")
     * @param locale      The locale/language code (e.g., "en-US")
     * @param gender      The voice gender ("Male" or "Female")
     * @param description Optional description of the voice characteristics
     */
    @JsonCreator
    public Voice(
            @JsonProperty("name") String name,
            @JsonProperty("displayName") String displayName,
            @JsonProperty("locale") String locale,
            @JsonProperty("gender") String gender,
            @JsonProperty("description") String description) {
        this.name = Objects.requireNonNull(name, "Voice name cannot be null");
        this.displayName = Objects.requireNonNull(displayName, "Display name cannot be null");
        this.locale = Objects.requireNonNull(locale, "Locale cannot be null");
        this.gender = Objects.requireNonNull(gender, "Gender cannot be null");
        this.description = description; // Can be null
    }

    /**
     * Creates a new Voice instance without description.
     *
     * @param name        The technical name of the voice
     * @param displayName The human-readable display name
     * @param locale      The locale/language code
     * @param gender      The voice gender
     */
    public Voice(String name, String displayName, String locale, String gender) {
        this(name, displayName, locale, gender, null);
    }

    /**
     * Gets the technical name of the voice.
     *
     * @return The voice name (e.g., "en-US-AriaNeural")
     */
    @Override
    public String name() {
        return name;
    }

    /**
     * Gets the human-readable display name of the voice.
     *
     * @return The display name (e.g., "Aria")
     */
    @Override
    public String displayName() {
        return displayName;
    }

    /**
     * Gets the locale/language code of the voice.
     *
     * @return The locale (e.g., "en-US")
     */
    @Override
    public String locale() {
        return locale;
    }

    /**
     * Gets the gender of the voice.
     *
     * @return The gender ("Male" or "Female")
     */
    @Override
    public String gender() {
        return gender;
    }

    /**
     * Gets the description of the voice characteristics.
     *
     * @return The description, or null if not available
     */
    @Override
    public String description() {
        return description;
    }

    /**
     * Gets the language code from the locale (e.g., "en" from "en-US").
     *
     * @return The language code
     */
    public String getLanguageCode() {
        if (locale.contains("-")) {
            return locale.substring(0, locale.indexOf("-"));
        }
        return locale;
    }

    /**
     * Gets the language code from the locale (e.g., "en" from "en-US").
     *
     * @return The language code
     * @deprecated Use getLanguageCode() instead for consistency with other implementations
     */
    @Deprecated
    public String getLanguage() {
        return getLanguageCode();
    }

    /**
     * Gets the country code from the locale (e.g., "US" from "en-US").
     *
     * @return The country code, or empty string if not available
     */
    public String getCountryCode() {
        if (locale.contains("-")) {
            return locale.substring(locale.indexOf("-") + 1);
        }
        return "";
    }

    /**
     * Gets the country code from the locale (e.g., "US" from "en-US").
     *
     * @return The country code, or empty string if not available
     * @deprecated Use getCountryCode() instead for consistency with other implementations
     */
    @Deprecated
    public String getCountry() {
        return getCountryCode();
    }

    /**
     * Checks if this voice matches the given language code.
     *
     * @param language The language code to match (e.g., "en", "es", "fr")
     * @return true if the voice matches the language
     */
    public boolean matchesLanguage(String language) {
        if (language == null) return false;
        String normalizedLanguage = language.toLowerCase().trim();
        return getLanguageCode().toLowerCase().equals(normalizedLanguage) ||
                locale().toLowerCase().equals(normalizedLanguage);
    }

    /**
     * Filters a list of voices by language code.
     *
     * @param voices   The list of voices to filter
     * @param language The language code to filter by (e.g., "en", "es", "fr")
     * @return A stream of voices matching the specified language
     */
    public static Stream<Voice> getVoicesByLanguage(List<Voice> voices, String language) {
        Objects.requireNonNull(voices, "Voices list cannot be null");
        Objects.requireNonNull(language, "Language cannot be null");

        String normalizedLanguage = language.toLowerCase().trim();

        return voices.stream()
                .filter(voice -> voice.getLanguageCode().toLowerCase().equals(normalizedLanguage));
    }

    /**
     * Filters a list of voices by full locale (e.g., "en-US").
     *
     * @param voices The list of voices to filter
     * @param locale The locale to filter by
     * @return A stream of voices matching the specified locale
     */
    public static Stream<Voice> getVoicesByLocale(List<Voice> voices, String locale) {
        Objects.requireNonNull(voices, "Voices list cannot be null");
        Objects.requireNonNull(locale, "Locale cannot be null");

        String normalizedLocale = locale.toLowerCase().trim();

        return voices.stream()
                .filter(voice -> voice.locale().toLowerCase().equals(normalizedLocale));
    }

    /**
     * Filters a list of voices by gender.
     *
     * @param voices The list of voices to filter
     * @param gender The gender to filter by ("Male" or "Female")
     * @return A stream of voices matching the specified gender
     */
    public static Stream<Voice> getVoicesByGender(List<Voice> voices, String gender) {
        Objects.requireNonNull(voices, "Voices list cannot be null");
        Objects.requireNonNull(gender, "Gender cannot be null");

        String normalizedGender = gender.toLowerCase().trim();

        return voices.stream()
                .filter(voice -> voice.gender().toLowerCase().equals(normalizedGender));
    }

    /**
     * Parses voices from a JSON file containing voice configuration data.
     *
     * @param jsonFilePath The path to the JSON file
     * @return A list of Voice objects parsed from the file
     * @throws Exception If the file cannot be read or parsed
     */
    public static List<Voice> parseVoicesFromJsonFile(String jsonFilePath) throws Exception {
        Objects.requireNonNull(jsonFilePath, "JSON file path cannot be null");

        ObjectMapper mapper = new ObjectMapper();
        JsonNode rootNode = mapper.readTree(new File(jsonFilePath));
        List<Voice> voices = new ArrayList<>();

        // Parse voices from the languages section
        JsonNode languagesNode = rootNode.get("languages");
        if (languagesNode != null && languagesNode.isArray()) {
            for (JsonNode langNode : languagesNode) {
                try {
                    String code = langNode.get("code").asText();
                    String name = langNode.get("name").asText();
                    JsonNode edgeVoiceNode = langNode.get("edge_voice");

                    if (edgeVoiceNode != null && !edgeVoiceNode.isNull()) {
                        String voiceName = edgeVoiceNode.asText();
                        Voice voice = new Voice(voiceName, name, code, "Unknown");
                        voices.add(voice);
                    }
                } catch (Exception e) {
                    // Log error but continue processing other voices
                    log.warn("Warning: Failed to parse language entry: {}", e.getMessage());
                }
            }
        }

        return voices;
    }

    @Override
    public String toString() {
        return String.format("Voice{name='%s', displayName='%s', locale='%s', gender='%s', description='%s'}",
                name, displayName, locale, gender, description);
    }

    /**
     * Returns a formatted string representation suitable for display to users.
     *
     * @return A user-friendly string representation
     */
    public String toDisplayString() {
        StringBuilder sb = new StringBuilder();
        sb.append(displayName).append(" (").append(locale).append(", ").append(gender).append(")");
        if (description != null && !description.trim().isEmpty()) {
            sb.append(" - ").append(description);
        }
        return sb.toString();
    }
}