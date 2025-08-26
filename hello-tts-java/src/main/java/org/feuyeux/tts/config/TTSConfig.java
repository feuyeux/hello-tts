package org.feuyeux.tts.config;

import lombok.Getter;
import lombok.Setter;

import java.util.HashMap;
import java.util.Map;

/**
 * Configuration class for TTS client settings
 */
@Setter
@Getter
public class TTSConfig {
    // Getters and setters
    private String defaultVoice;
    private String outputFormat;
    private String outputDirectory;
    private boolean autoPlay;
    private boolean cacheVoices;
    private int maxRetries;
    private int timeout;
    private String rate;
    private String pitch;
    private String volume;
    private int batchSize;
    private int maxConcurrent;

    // Default constructor
    public TTSConfig() {
        this.defaultVoice = "en-US-AriaNeural";
        this.outputFormat = "mp3";
        this.outputDirectory = "./output";
        this.autoPlay = true;
        this.cacheVoices = true;
        this.maxRetries = 3;
        this.timeout = 30000;
        this.rate = "0%";
        this.pitch = "0%";
        this.volume = "100%";
        this.batchSize = 5;
        this.maxConcurrent = 3;
    }

    // Constructor with all parameters
    public TTSConfig(String defaultVoice, String outputFormat, String outputDirectory,
                     boolean autoPlay, boolean cacheVoices, int maxRetries, int timeout,
                     String rate, String pitch, String volume,
                     int batchSize, int maxConcurrent) {
        this.defaultVoice = defaultVoice;
        this.outputFormat = outputFormat;
        this.outputDirectory = outputDirectory;
        this.autoPlay = autoPlay;
        this.cacheVoices = cacheVoices;
        this.maxRetries = maxRetries;
        this.timeout = timeout;
        this.rate = rate;
        this.pitch = pitch;
        this.volume = volume;
        this.batchSize = batchSize;
        this.maxConcurrent = maxConcurrent;
        validate();
    }

    // Create from Map (for JSON deserialization)
    public static TTSConfig fromMap(Map<String, Object> map) {
        TTSConfig config = new TTSConfig();
        
        if (map.containsKey("defaultVoice")) config.defaultVoice = (String) map.get("defaultVoice");
        if (map.containsKey("outputFormat")) config.outputFormat = (String) map.get("outputFormat");
        if (map.containsKey("outputDirectory")) config.outputDirectory = (String) map.get("outputDirectory");
        if (map.containsKey("autoPlay")) config.autoPlay = (Boolean) map.get("autoPlay");
        if (map.containsKey("cacheVoices")) config.cacheVoices = (Boolean) map.get("cacheVoices");
        if (map.containsKey("maxRetries")) config.maxRetries = ((Number) map.get("maxRetries")).intValue();
        if (map.containsKey("timeout")) config.timeout = ((Number) map.get("timeout")).intValue();
        if (map.containsKey("rate")) config.rate = (String) map.get("rate");
        if (map.containsKey("pitch")) config.pitch = (String) map.get("pitch");
        if (map.containsKey("volume")) config.volume = (String) map.get("volume");
        if (map.containsKey("batchSize")) config.batchSize = ((Number) map.get("batchSize")).intValue();
        if (map.containsKey("maxConcurrent")) config.maxConcurrent = ((Number) map.get("maxConcurrent")).intValue();
        
        config.validate();
        return config;
    }

    // Convert to Map (for JSON serialization)
    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("defaultVoice", defaultVoice);
        map.put("outputFormat", outputFormat);
        map.put("outputDirectory", outputDirectory);
        map.put("autoPlay", autoPlay);
        map.put("cacheVoices", cacheVoices);
        map.put("maxRetries", maxRetries);
        map.put("timeout", timeout);
        map.put("rate", rate);
        map.put("pitch", pitch);
        map.put("volume", volume);
        map.put("batchSize", batchSize);
        map.put("maxConcurrent", maxConcurrent);
        return map;
    }

    // Validation
    public void validate() {
        if (maxRetries < 0) {
            throw new IllegalArgumentException("maxRetries must be non-negative");
        }
        if (timeout <= 0) {
            throw new IllegalArgumentException("timeout must be positive");
        }
        if (defaultVoice == null || defaultVoice.isEmpty()) {
            throw new IllegalArgumentException("defaultVoice cannot be empty");
        }
        if (batchSize <= 0) {
            throw new IllegalArgumentException("batchSize must be positive");
        }
        if (maxConcurrent <= 0) {
            throw new IllegalArgumentException("maxConcurrent must be positive");
        }
    }

    // Builder pattern
    public static class Builder {
        private final TTSConfig config = new TTSConfig();

        public Builder defaultVoice(String defaultVoice) {
            config.defaultVoice = defaultVoice;
            return this;
        }

        public Builder outputFormat(String outputFormat) {
            config.outputFormat = outputFormat;
            return this;
        }

        public Builder outputDirectory(String outputDirectory) {
            config.outputDirectory = outputDirectory;
            return this;
        }

        public Builder autoPlay(boolean autoPlay) {
            config.autoPlay = autoPlay;
            return this;
        }

        public Builder cacheVoices(boolean cacheVoices) {
            config.cacheVoices = cacheVoices;
            return this;
        }

        public Builder maxRetries(int maxRetries) {
            config.maxRetries = maxRetries;
            return this;
        }

        public Builder timeout(int timeout) {
            config.timeout = timeout;
            return this;
        }

        public Builder rate(String rate) {
            config.rate = rate;
            return this;
        }

        public Builder pitch(String pitch) {
            config.pitch = pitch;
            return this;
        }

        public Builder volume(String volume) {
            config.volume = volume;
            return this;
        }

        public Builder batchSize(int batchSize) {
            config.batchSize = batchSize;
            return this;
        }

        public Builder maxConcurrent(int maxConcurrent) {
            config.maxConcurrent = maxConcurrent;
            return this;
        }

        public TTSConfig build() {
            config.validate();
            return config;
        }
    }

    @Override
    public String toString() {
        return "TTSConfig{" +
                "defaultVoice='" + defaultVoice + '\'' +
                ", outputFormat='" + outputFormat + '\'' +
                ", outputDirectory='" + outputDirectory + '\'' +
                ", autoPlay=" + autoPlay +
                ", cacheVoices=" + cacheVoices +
                ", maxRetries=" + maxRetries +
                ", timeout=" + timeout +
                ", rate='" + rate + '\'' +
                ", pitch='" + pitch + '\'' +
                ", volume='" + volume + '\'' +
                ", batchSize=" + batchSize +
                ", maxConcurrent=" + maxConcurrent +
                '}';
    }
}