package org.feuyeux.edgetts.exception;

/**
 * 基础异常类，所有 TTS 相关异常均继承自此类
 */
public class TTSException extends Exception {
    public enum ErrorType {
        NETWORK,
        SYNTHESIS,
        VALIDATION,
        IO,
        CONFIG,
        VOICE_NOT_FOUND,
        AUDIO
    }
    private final ErrorType errorType;

    public TTSException(String message) {
        super(message);
        this.errorType = ErrorType.SYNTHESIS;
    }
    public TTSException(String message, Throwable cause) {
        super(message, cause);
        this.errorType = ErrorType.SYNTHESIS;
    }
    public TTSException(String message, ErrorType errorType) {
        super(message);
        this.errorType = errorType;
    }
    public TTSException(String message, Throwable cause, ErrorType errorType) {
        super(message, cause);
        this.errorType = errorType;
    }
    public ErrorType getErrorType() {
        return errorType;
    }
}