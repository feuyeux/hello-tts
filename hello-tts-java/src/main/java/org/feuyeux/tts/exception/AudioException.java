package org.feuyeux.tts.exception;

/**
 * Exception for audio-related TTS errors
 */
public class AudioException extends TTSException {
    public AudioException(String message) {
        super(message, ErrorType.AUDIO);
    }
    public AudioException(String message, Throwable cause) {
        super(message, cause, ErrorType.AUDIO);
    }
}