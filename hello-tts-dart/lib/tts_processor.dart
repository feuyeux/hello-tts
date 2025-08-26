import 'dart:convert';
import 'dart:typed_data';
import 'dart:io';
import 'hello_tts.dart';

/// TTS Processor supporting both Edge TTS and Google TTS backends
class TTSProcessor {
  List<Voice>? _cachedVoices;
  final String backend;

  TTSProcessor({this.backend = 'edge'});

  /// 获取本地语音列表
  Future<List<Voice>> getVoices() async {
    if (_cachedVoices != null) {
      return _cachedVoices!;
    }
    try {
      // Look for one of the known config locations (prefer unified tts_config.json)
      final candidatePaths = [
        '../shared/tts_config.json',
        '../../shared/tts_config.json',
        'shared/tts_config.json',
      ];

      File? file;
      for (final p in candidatePaths) {
        final f = File(p);
        if (await f.exists()) {
          file = f;
          break;
        }
      }

      if (file == null) {
        throw TTSError(
          'No voice configuration found. Searched: ${candidatePaths.join(', ')}',
        );
      }

      final content = await file.readAsString();
      final Map<String, dynamic> jsonMap = json.decode(content);
      final List<Voice> voices = [];

      // Parse from unified tts_config.json (languages array)
      if (jsonMap.containsKey('languages')) {
        final langs = jsonMap['languages'] as List<dynamic>;
        for (final item in langs) {
          final m = item as Map<String, dynamic>;
          final code = (m['code'] ?? '') as String;
          final name = (m['name'] ?? '') as String;
          final voiceVal = (m['voice'] ??
              m['edge_voice'] ??
              m['google_voice'] ??
              '') as String;
          if (voiceVal.isEmpty) continue;
          voices.add(
            Voice(
              name: voiceVal,
              displayName: name.isNotEmpty ? name : voiceVal,
              language: code.split('-')[0],
              gender: 'Unknown',
              locale: code,
              isNeural: voiceVal.toLowerCase().contains('neural'),
              isStandard: !voiceVal.toLowerCase().contains('neural'),
            ),
          );
        }
      }

      _cachedVoices = voices;
      return voices;
    } catch (e) {
      throw TTSError('Error loading local voices: $e');
    }
  }

  /// 文本转语音
  Future<Uint8List> synthesizeText(
    String text,
    String voiceName, {
    String format = 'mp3',
  }) async {
    try {
      if (backend == 'google') {
        return await _synthesizeViaGoogleTTS(text, voiceName, format);
      } else {
        return await _synthesizeViaEdgeTTS(text, voiceName, format);
      }
    } catch (e) {
      throw TTSError('Error during synthesis: $e');
    }
  }

  /// 调用 Python edge-tts 库进行语音合成
  Future<Uint8List> _synthesizeViaEdgeTTS(
    String text,
    String voiceName,
    String format,
  ) async {
    try {
      final tempDir = Directory.systemTemp;
      final tempFile = File(
        '${tempDir.path}/tts_output_${DateTime.now().millisecondsSinceEpoch}.mp3',
      );
      try {
        final result = await Process.run('edge-tts', [
          '--voice',
          voiceName,
          '--text',
          text,
          '--write-media',
          tempFile.path,
        ]);

        if (result.exitCode != 0) {
          throw TTSError('edge-tts command failed: ${result.stderr}');
        }

        if (!await tempFile.exists()) {
          throw TTSError('Output file was not created');
        }

        final audioData = await tempFile.readAsBytes();
        return Uint8List.fromList(audioData);
      } finally {
        if (await tempFile.exists()) {
          await tempFile.delete();
        }
      }
    } catch (e) {
      throw TTSError('Failed to synthesize via Edge TTS: $e');
    }
  }

  /// 调用 Python gTTS 库进行语音合成
  Future<Uint8List> _synthesizeViaGoogleTTS(
    String text,
    String voiceName,
    String format,
  ) async {
    try {
      // Extract language code from voice name robustly.
      // Examples:
      //  - "zh-CN-XiaoxiaoNeural" -> "zh-cn"
      //  - "en-US-AriaNeural" -> "en-us"
      //  - "en" -> "en"
      String langCode = 'en'; // default
      final vn = voiceName.replaceAll('_', '-');
      if (vn.contains('-')) {
        final parts = vn.split('-');
        // If second part looks like a region code (2 letters), include it: en-US -> en-us
        if (parts.length >= 2 && parts[1].length == 2) {
          langCode = '${parts[0]}-${parts[1]}'.toLowerCase();
        } else {
          // Otherwise use primary language subtag only
          langCode = parts[0].toLowerCase();
        }
      } else if (vn.isNotEmpty) {
        langCode = vn.toLowerCase();
      }

      final tempDir = Directory.systemTemp;
      final tempFile = File(
        '${tempDir.path}/gtts_output_${DateTime.now().millisecondsSinceEpoch}.mp3',
      );
      try {
        final result = await Process.run('gtts-cli', [
          text,
          '-l',
          langCode,
          '-o',
          tempFile.path,
        ]);

        if (result.exitCode != 0) {
          throw TTSError('gtts-cli command failed: ${result.stderr}');
        }

        if (!await tempFile.exists()) {
          throw TTSError('Output file was not created');
        }

        final audioData = await tempFile.readAsBytes();
        return Uint8List.fromList(audioData);
      } finally {
        if (await tempFile.exists()) {
          await tempFile.delete();
        }
      }
    } catch (e) {
      throw TTSError('Failed to synthesize via Google TTS: $e');
    }
  }

  /// 清理语音缓存
  void clearVoiceCache() {
    _cachedVoices = null;
  }

  /// Release any resources held by this processor (keeps API parity with tests)
  void dispose() {
    clearVoiceCache();
  }
}
