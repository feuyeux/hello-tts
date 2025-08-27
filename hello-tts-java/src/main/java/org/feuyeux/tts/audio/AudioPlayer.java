package org.feuyeux.tts.audio;

import lombok.extern.slf4j.Slf4j;
import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class AudioPlayer {
    public void playFile(String filename) throws AudioPlayerException {
        Objects.requireNonNull(filename, "Filename cannot be null");
        if (filename.trim().isEmpty()) {
            throw new IllegalArgumentException("Filename cannot be empty");
        }

        Path filePath = Paths.get(filename);
        if (!Files.exists(filePath)) {
            throw new AudioPlayerException("Audio file not found: " + filename);
        }

        if (!Files.isReadable(filePath)) {
            throw new AudioPlayerException("Audio file is not readable: " + filename);
        }

        try {
            playWithSystemCommand(filename);
        } catch (Exception e) {
            throw new AudioPlayerException("Failed to play audio file: " + filename, e);
        }
    }

    private void playWithSystemCommand(String filename) throws Exception {
        String os = System.getProperty("os.name").toLowerCase();
        File file = new File(filename);
        String absolutePath = file.getAbsolutePath();

        if (os.contains("mac")) {
            // macOS
            ProcessBuilder pb = new ProcessBuilder("afplay", absolutePath);
            Process process = pb.start();
            int exitCode = process.waitFor();
            if (exitCode != 0) {
                throw new Exception("Audio playback failed with exit code: " + exitCode);
            }
        } else if (os.contains("win")) {
            // Check file size first
            long fileSize = Files.size(file.toPath());
            if (fileSize < 100) { // Minimum size for a valid audio file
                throw new Exception("Audio file is too small to be valid: " + fileSize + " bytes");
            }

            // Windows - Use PowerShell to play MP3
            String psCommand = String.format(
                    "Add-Type -AssemblyName PresentationCore; " +
                            "$mediaPlayer = New-Object System.Windows.Media.MediaPlayer; " +
                            "$mediaPlayer.Open('file:///' + '%s'.Replace('\\', '/')); " +
                            "$mediaPlayer.Play(); " +
                            "Start-Sleep -Seconds 2; " + // Wait for the audio to start
                            "while ($mediaPlayer.Position -lt $mediaPlayer.NaturalDuration.TimeSpan) { Start-Sleep -Milliseconds 100 }; "
                            +
                            "$mediaPlayer.Stop(); " +
                            "$mediaPlayer.Close();",
                    absolutePath.replace("'", "''"));

            ProcessBuilder processBuilder = new ProcessBuilder(
                    "powershell",
                    "-NoProfile",
                    "-ExecutionPolicy", "Bypass",
                    "-Command", psCommand);
            processBuilder.redirectErrorStream(true);

            Process proc = processBuilder.start();
            try (BufferedReader reader = new BufferedReader(new InputStreamReader(proc.getInputStream()))) {
                String line;
                while ((line = reader.readLine()) != null) {
                    log.debug("PowerShell output: " + line);
                }
            }

            int exitCode = proc.waitFor();
            if (exitCode != 0) {
                throw new Exception("PowerShell audio playback failed with exit code: " + exitCode);
            }

        } else {
            // Linux - try multiple players
            String[] players = { "paplay", "aplay", "mpg123", "mplayer" };
            boolean played = false;
            for (String player : players) {
                try {
                    ProcessBuilder pb = new ProcessBuilder(player, absolutePath);
                    Process process = pb.start();
                    int exitCode = process.waitFor();
                    if (exitCode == 0) {
                        played = true;
                        break;
                    }
                } catch (Exception e) {
                    // Try next player
                }
            }

            if (!played) {
                throw new Exception("No suitable audio player found on Linux. Please install mpg123.");
            }
        }
    }

    public CompletableFuture<Void> playFileAsync(String filename) {
        return CompletableFuture.runAsync(() -> {
            try {
                playFile(filename);
            } catch (AudioPlayerException e) {
                throw new RuntimeException(e);
            }
        });
    }

    public static class AudioPlayerException extends Exception {
        public AudioPlayerException(String message) {
            super(message);
        }

        public AudioPlayerException(String message, Throwable cause) {
            super(message, cause);
        }
    }
}
