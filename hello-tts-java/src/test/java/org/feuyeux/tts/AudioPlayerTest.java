package org.feuyeux.tts;

import org.feuyeux.tts.audio.AudioPlayer;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for AudioPlayer class.
 * Tests basic functionality and error handling.
 */
class AudioPlayerTest {

    private AudioPlayer audioPlayer;

    @TempDir
    Path tempDir;

    @BeforeEach
    void setUp() {
        audioPlayer = new AudioPlayer();
    }

    @AfterEach
    void tearDown() {
        // No cleanup needed
    }

    @Test
    void testPlayFileWithNullFilename() {
        assertThrows(NullPointerException.class, () -> {
            audioPlayer.playFile(null);
        });
    }

    @Test
    void testPlayFileWithEmptyFilename() {
        assertThrows(IllegalArgumentException.class, () -> {
            audioPlayer.playFile("");
        });

        assertThrows(IllegalArgumentException.class, () -> {
            audioPlayer.playFile("   ");
        });
    }

    @Test
    void testPlayFileWithNonExistentFile() {
        AudioPlayer.AudioPlayerException exception = assertThrows(
                AudioPlayer.AudioPlayerException.class,
                () -> audioPlayer.playFile("nonexistent_file.wav"));

        assertTrue(exception.getMessage().contains("not found"));
    }

    @Test
    void testCreateAndPlayTempFile() throws IOException {
        // Create a simple WAV file header (minimal valid WAV)
        byte[] wavHeader = createMinimalWavHeader();

        File tempFile = tempDir.resolve("test.wav").toFile();
        try (FileOutputStream fos = new FileOutputStream(tempFile)) {
            fos.write(wavHeader);
        }

        // Expect an exception because the WAV file is minimal and not playable
        assertThrows(
                AudioPlayer.AudioPlayerException.class,
                () -> audioPlayer.playFile(tempFile.getAbsolutePath()));
    }

    @Test
    void testAsyncPlayFile() {
        // Test that async methods return CompletableFuture
        assertDoesNotThrow(() -> {
            var future = audioPlayer.playFileAsync("nonexistent.wav");
            assertNotNull(future);

            // The future should complete exceptionally due to file not found
            assertThrows(Exception.class, () -> future.join());
        });
    }

    /**
     * Creates a minimal WAV file header for testing purposes.
     * This creates a valid but empty WAV file structure.
     */
    private byte[] createMinimalWavHeader() {
        // Minimal WAV header (44 bytes)
        byte[] header = new byte[44];

        // RIFF header
        header[0] = 0x52;
        header[1] = 0x49;
        header[2] = 0x46;
        header[3] = 0x46; // "RIFF"

        // File size (36 + data size, little endian) - minimal size
        header[4] = 36;
        header[5] = 0;
        header[6] = 0;
        header[7] = 0;

        // WAVE header
        header[8] = 0x57;
        header[9] = 0x41;
        header[10] = 0x56;
        header[11] = 0x45; // "WAVE"

        // fmt subchunk
        header[12] = 0x66;
        header[13] = 0x6D;
        header[14] = 0x74;
        header[15] = 0x20; // "fmt "

        // Subchunk1Size (16 for PCM)
        header[16] = 16;
        header[17] = 0;
        header[18] = 0;
        header[19] = 0;

        // AudioFormat (1 for PCM)
        header[20] = 1;
        header[21] = 0;

        // NumChannels (1 = mono)
        header[22] = 1;
        header[23] = 0;

        // SampleRate (44100 Hz)
        header[24] = 0x44;
        header[25] = (byte) 0xAC;
        header[26] = 0;
        header[27] = 0;

        // ByteRate (SampleRate * NumChannels * BitsPerSample/8)
        header[28] = (byte) 0x88;
        header[29] = 0x58;
        header[30] = 0x01;
        header[31] = 0;

        // BlockAlign (NumChannels * BitsPerSample/8)
        header[32] = 2;
        header[33] = 0;

        // BitsPerSample (16)
        header[34] = 16;
        header[35] = 0;

        // data subchunk
        header[36] = 0x64;
        header[37] = 0x61;
        header[38] = 0x74;
        header[39] = 0x61; // "data"

        // Subchunk2Size (0 for empty audio)
        header[40] = 0;
        header[41] = 0;
        header[42] = 0;
        header[43] = 0;

        return header;
    }
}
