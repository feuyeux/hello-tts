package org.feuyeux.tts.tts;

import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Semaphore;

/**
 * TTS client implementation supporting both Edge TTS and Google TTS
 */
@Slf4j
public record TtsProcessor(String backend) {
    public TtsProcessor(String backend) {
        this.backend = backend;
        log.info("TTSClient 实例已创建，后端: {}", backend);
    }

    public CompletableFuture<byte[]> synthesizeText(String text, String voice) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                log.info("开始文本合成 - 后端: {}, 语音: {}, 文本长度: {} 字符", backend, voice, text.length());
                byte[] result;
                if ("google".equals(backend)) {
                    result = synthesizeViaGoogleTTS(text, voice);
                } else {
                    result = synthesizeViaEdgeTTS(text, voice);
                }
                log.info("文本合成完成，音频数据大小: {} 字节", result.length);
                return result;
            } catch (Exception e) {
                log.error("合成失败: {}", e.getMessage());
                throw new RuntimeException("TTS synthesis failed: " + e.getMessage(), e);
            }
        });
    }

    private byte[] synthesizeViaEdgeTTS(String text, String voice) throws Exception {
        File tempFile = File.createTempFile("tts_output_", ".mp3");
        tempFile.deleteOnExit();
        try {
            // Try edge-tts command first
            ProcessBuilder pb = new ProcessBuilder(
                    "edge-tts",
                    "--voice", voice,
                    "--text", text,
                    "--write-media", tempFile.getAbsolutePath()
            );
            Process process = pb.start();
            int exitCode = process.waitFor();
            log.info("edge-tts 进程退出码: {}", exitCode);
            if (exitCode != 0) {
                log.warn("直接调用 edge-tts 失败，尝试使用 python -m edge_tts");
                ProcessBuilder pythonPb = new ProcessBuilder(
                        "python", "-m", "edge_tts",
                        "--voice", voice,
                        "--text", text,
                        "--write-media", tempFile.getAbsolutePath()
                );
                Process pythonProcess = pythonPb.start();
                int pythonExitCode = pythonProcess.waitFor();
                log.info("python -m edge_tts 进程退出码: {}", pythonExitCode);
                if (pythonExitCode != 0) {
                    throw new RuntimeException("Edge TTS failed with exit code: " + pythonExitCode);
                }
            }
            if (tempFile.exists() && tempFile.length() > 0) {
                byte[] audioData = Files.readAllBytes(tempFile.toPath());
                log.debug("音频文件读取成功，大小: {} 字节", audioData.length);
                return audioData;
            } else {
                throw new RuntimeException("Audio file was not generated or is empty");
            }
        } finally {
            if (tempFile.exists()) {
                if (tempFile.delete()) {
                    log.debug("edgeTTS临时文件已删除: {}", tempFile.getAbsolutePath());
                }
            }
        }
    }

    private byte[] synthesizeViaGoogleTTS(String text, String voice) throws Exception {
        File tempFile = File.createTempFile("gtts_output_", ".mp3");

        try {
            // Try gtts-cli command
            // gtts-cli expects text as a positional arg and uses -l and -o flags
            ProcessBuilder processBuilder = new ProcessBuilder(
                    "gtts-cli",
                    text,
                    "-l", extractLangFromVoice(voice),
                    "-o", tempFile.getAbsolutePath()
            );

            processBuilder.redirectErrorStream(true);
            Process process = processBuilder.start();
            int exitCode = process.waitFor();

            log.info("gtts-cli 进程退出码: {}", exitCode);

            if (exitCode != 0) {
                throw new RuntimeException("Google TTS command failed with exit code: " + exitCode);
            }

            if (tempFile.exists() && tempFile.length() > 0) {
                byte[] audioData = Files.readAllBytes(tempFile.toPath());
                log.info("音频文件读取成功，大小: {} 字节", audioData.length);
                return audioData;
            } else {
                throw new RuntimeException("Audio file was not generated or is empty");
            }

        } finally {
            if (tempFile.exists()) {
                if (tempFile.delete()) {
                    log.debug("gTTS临时文件已删除: {}", tempFile.getAbsolutePath());
                }
            }
        }
    }

    /**
     * Extract a short language code for gtts-cli from a voice string.
     * Examples:
     * - "en-US-AriaNeural" -> "en"
     * - "zh-CN-XiaoxiaoNeural" -> "zh"
     */
    private String extractLangFromVoice(String voice) {
        if (voice == null || voice.isEmpty()) {
            return "en";
        }
        try {
            String[] parts = voice.split("-");
            if (parts.length > 0) {
                return parts[0].toLowerCase();
            }
        } catch (Exception e) {
            // fallthrough
        }
        return "en";
    }

    /**
     * Save audio data to file
     */
    public CompletableFuture<Void> saveAudio(CompletableFuture<byte[]> audioFuture, String filename) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                // Ensure output directory exists
                File file = new File(filename);
                File parentDir = file.getParentFile();
                if (parentDir != null && !parentDir.exists()) {
                    if (parentDir.mkdirs()) {
                        log.info("创建输出目录: {}", parentDir.getAbsolutePath());
                    }
                }

                // Write audio data to file
                Files.write(file.toPath(), audioFuture.get());
                log.info("音频数据已保存: {}", file.getAbsolutePath());
                return null;
            } catch (Exception e) {
                log.error("保存音频文件失败: {}", e.getMessage());
                throw new RuntimeException("Failed to save audio file: " + e.getMessage(), e);
            }
        });
    }

    public CompletableFuture<List<byte[]>> batchSynthesizeText(String[] texts, String voice) {
        return CompletableFuture.supplyAsync(() -> {
            List<byte[]> results = new ArrayList<>();
            for (int i = 0; i < texts.length; i++) {
                try {
                    log.info("Processing batch item {}/{}", i + 1, texts.length);
                    results.add(synthesizeText(texts[i], voice).get());
                } catch (Exception e) {
                    log.error("Failed to synthesize batch item {}: {}", i + 1, e.getMessage());
                    throw new RuntimeException("Failed to synthesize batch item " + (i + 1), e);
                }
            }

            log.info("批量合成完成，总共 {} 项", results.size());
            return results;
        });
    }

    /**
     * Convert multiple texts to audio data concurrently using specified voice
     */
    public CompletableFuture<List<byte[]>> batchSynthesizeConcurrent(String[] texts, String voice, int maxConcurrent) {
        return CompletableFuture.supplyAsync(() -> {
            List<CompletableFuture<IndexedResult>> futures = new ArrayList<>();
            Semaphore semaphore = new Semaphore(maxConcurrent);

            for (int i = 0; i < texts.length; i++) {
                final int index = i;
                final String text = texts[i];

                CompletableFuture<IndexedResult> future = CompletableFuture.supplyAsync(() -> {
                    try {
                        semaphore.acquire();
                        log.info("处理并发项 {}/{}: {}...", index + 1, texts.length, text.substring(0, Math.min(50, text.length())));

                        CompletableFuture<byte[]> audioFuture = synthesizeText(text, voice);
                        byte[] audioData = audioFuture.get();
                        return new IndexedResult(index, audioData);
                    } catch (Exception e) {
                        log.error("合成并发项 {} 失败: {}", index + 1, e.getMessage());
                        throw new RuntimeException("Failed to synthesize concurrent item " + (index + 1) + ": " + e.getMessage(), e);
                    } finally {
                        semaphore.release();
                    }
                });

                futures.add(future);
            }

            // Wait for all futures to complete
            try {
                List<IndexedResult> indexedResults = new ArrayList<>();
                for (CompletableFuture<IndexedResult> future : futures) {
                    indexedResults.add(future.get());
                }

                // Sort by index to maintain order
                indexedResults.sort(Comparator.comparingInt(a -> a.index));

                // Extract audio data
                List<byte[]> results = new ArrayList<>();
                for (IndexedResult result : indexedResults) {
                    results.add(result.audioData);
                }

                log.info("并发批量合成完成，总共 {} 项", results.size());
                return results;
            } catch (Exception e) {
                log.error("并发批量处理时发生错误: {}", e.getMessage());
                throw new RuntimeException("Error in concurrent batch processing: " + e.getMessage(), e);
            }
        });
    }

    /**
         * Helper class for maintaining order in concurrent processing
         */
        private record IndexedResult(int index, byte[] audioData) {
    }
}