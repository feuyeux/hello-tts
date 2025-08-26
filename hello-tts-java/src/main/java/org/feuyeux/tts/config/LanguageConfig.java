package org.feuyeux.tts.config;

import lombok.Data;
import lombok.extern.slf4j.Slf4j;

@Data
@Slf4j
public class  LanguageConfig {
    private String code;
    private String name;
    private String flag;
    private String text;
    private String edgeVoice;
    private String googleVoice;
    @Override
    public String toString() {
        return String.format("%s %s (%s)", flag, name, code.toUpperCase());
    }
}