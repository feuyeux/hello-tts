package org.feuyeux.edgetts.exception;

/**
 * Exception for network-related TTS errors
 */
public class NetworkException extends TTSException {
    public NetworkException(String message) {
        super(message, ErrorType.NETWORK);
    }
    public NetworkException(String message, Throwable cause) {
        super(message, cause, ErrorType.NETWORK);
    }
}