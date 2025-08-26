package org.feuyeux.tts.config;

import java.io.IOException;

// Convenience functions
class ConfigUtils {
    public static TTSConfig loadConfig(String configPath) throws IOException {
        return ConfigManager.loadConfig(configPath);
    }

    public static TTSConfig getPreset(String presetName) {
        return ConfigManager.getPreset(presetName);
    }

    public static void createDefaultConfig(String filePath, String preset) throws IOException {
        ConfigManager.createDefaultConfig(filePath, preset);
    }
}
