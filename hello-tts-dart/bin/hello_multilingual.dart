#!/usr/bin/env dart

import 'dart:io';
import 'dart:convert';
import 'dart:typed_data';
import 'package:path/path.dart' as path;
import 'package:logging/logging.dart';
import '../lib/hello_tts.dart';

final Logger logger = Logger('multilingual_demo');

/// Language configuration structure
class LanguageConfig {
  final String code;
  final String name;
  final String flag;
  final String text;
  final String voice;
  final String? altVoice;

  LanguageConfig({
    required this.code,
    required this.name,
    required this.flag,
    required this.text,
    required this.voice,
    this.altVoice,
  });

  factory LanguageConfig.fromJson(Map<String, dynamic> json) {
    return LanguageConfig(
      code: json['code'] as String,
      name: json['name'] as String,
      flag: json['flag'] as String,
      text: json['text'] as String,
      voice: json['voice'] as String,
      altVoice: json['alt_voice'] as String?,
    );
  }
}

String backend = Platform.environment['TTS_BACKEND'] ?? 'edge';

/// Load language configuration from JSON file
Future<List<LanguageConfig>> loadLanguageConfig(String backend) async {
  // Prefer unified tts_config.json if present (keeps parity with the shell script)
  final unifiedPath = '../shared/tts_config.json';

  try {
    File file;
    if (await File(unifiedPath).exists()) {
      file = File(unifiedPath);
      final content = await file.readAsString();
      final Map<String, dynamic> config = json.decode(content);
      final List<dynamic> languagesJson = config['languages'] ?? [];
      // tts_config.json uses 'google_voice'/'edge_voice' fields; normalize to 'voice'
      return languagesJson
          .map((v) {
            final map = v as Map<String, dynamic>;
            final voice = (backend.toLowerCase() == 'google')
                ? (map['google_voice'] ?? map['voice'] ?? '')
                : (map['edge_voice'] ?? map['voice'] ?? '');
            return LanguageConfig(
              code: map['code'] ?? '',
              name: map['name'] ?? '',
              flag: map['flag'] ?? '',
              text: map['text'] ?? '',
              voice: voice,
              altVoice: map['alt_voice'] ?? '',
            );
          })
          .where((lc) => lc.code.isNotEmpty)
          .toList();
    } else {
      logger.severe('❌ Configuration file not found (tried $unifiedPath');
      return [];
    }
  } catch (e) {
    logger.severe('❌ Error loading configuration: $e');
    return [];
  }
}

/// Create output directory if it doesn't exist
Future<void> createOutputDirectory(String directory) async {
  final dir = Directory(directory);
  if (!await dir.exists()) {
    await dir.create(recursive: true);
  }
}

/// Generate audio for a single language
Future<bool> generateAudioForLanguage(
  TTSProcessor client,
  LanguageConfig languageConfig,
  String outputDir, {
  bool playAudio = false,
}) async {
  final langCode = languageConfig.code;
  final langName = languageConfig.name;
  final flag = languageConfig.flag;
  final text = languageConfig.text;
  final voice = languageConfig.voice;
  final altVoice = languageConfig.altVoice;

  logger.info('\n$flag $langName (${langCode.toUpperCase()})');
  logger.info('Text: $text');
  logger.info('Voice: $voice');

  try {
    // Try primary voice first
    Uint8List? audioData;
    String usedVoice = voice;

    try {
      audioData = await client.synthesizeText(text, voice);
    } catch (e) {
      logger.warning('Primary voice failed: $e');
      if (altVoice != null) {
        logger.info('Trying alternative voice: $altVoice');
        try {
          audioData = await client.synthesizeText(text, altVoice);
          usedVoice = altVoice;
        } catch (e2) {
          logger.warning('Alternative voice also failed: $e2');
          rethrow;
        }
      } else {
        rethrow;
      }
    }

    // Generate filename
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final langPrefix = langCode.split('-')[0]; // e.g., 'zh' from 'zh-cn'
    final filename = '${langPrefix}_dart_${backend}_$timestamp.mp3';
    final outputPath = path.join(outputDir, filename);

    // Save audio
    final file = File(outputPath);
    await file.writeAsBytes(audioData);

    logger.info('✅ Generated: $filename');
    logger.info('📁 Saved to: $outputPath');
    logger.info('🎤 Used voice: $usedVoice');

    // Play audio if requested
    if (playAudio) {
      try {
        logger.info('🔊 Playing audio...');
        final player = AudioPlayer();
        await player.play(outputPath);
        logger.info('✅ Playback completed');
      } catch (e) {
        logger.warning('⚠️  Could not play audio: $e');
      }
    }

    return true;
  } catch (e) {
    logger.severe('❌ Failed to generate audio for $langName: $e');
    return false;
  }
}

/// Main function for multilingual demo
Future<void> main(List<String> args) async {
  // Configure logging
  Logger.root.level = Level.INFO;
  Logger.root.onRecord.listen((record) {
    print('${record.level.name}: ${record.message}');
  });

  if (args.isNotEmpty) {
    for (int i = 0; i < args.length; i++) {
      if (args[i] == '--backend' && i + 1 < args.length) {
        backend = args[i + 1];
        break;
      }
    }
  }

  final languages = await loadLanguageConfig(backend);
  if (languages.isEmpty) {
    logger.severe('❌ Failed to load language configuration or no engine found');
    exit(1);
  }

  logger.info('📋 Found ${languages.length} languages to process');

  // Create output directory
  const outputDir = 'output';
  await createOutputDirectory(outputDir);
  final outputPath = path.absolute(outputDir);
  logger.info('📁 Output directory: $outputPath');

  // Initialize TTS client
  late TTSProcessor client;
  try {
    client = TTSProcessor(backend: backend);
    logger.info('✅ TTS client initialized with $backend backend');
  } catch (e) {
    logger.severe('❌ Failed to initialize TTS client: $e');
    exit(1);
  }

  // Process each language
  int successfulCount = 0;
  int failedCount = 0;
  final startTime = DateTime.now();

  for (int i = 0; i < languages.length; i++) {
    final languageConfig = languages[i];
    logger.info('\n📍 Processing language ${i + 1}/${languages.length}');

    final success = await generateAudioForLanguage(
      client,
      languageConfig,
      outputDir,
      playAudio: false, // Set to true if you want to play each audio
    );

    if (success) {
      successfulCount++;
    } else {
      failedCount++;
    }

    // Small delay between languages to be polite to the service
    if (i < languages.length - 1) {
      logger.info('⏳ Waiting before next language...');
      await Future.delayed(const Duration(seconds: 2));
    }
  }

  // Summary
  final endTime = DateTime.now();
  final duration = endTime.difference(startTime);

  logger.info('\n🏁 Processing Complete!');
  logger.info('=' * 40);
  logger.info('✅ Successful: $successfulCount');
  logger.info('❌ Failed: $failedCount');
  logger.info(
      '⏱️  Total time: ${duration.inSeconds}.${duration.inMilliseconds % 1000} seconds');
  logger.info('📁 Output files saved in: $outputPath');

  if (successfulCount > 0) {
    logger.info(
        '\n🎉 Successfully generated audio files for $successfulCount languages!');
    logger
        .info('You can find all generated MP3 files in the output directory.');
  }

  exit(failedCount == 0 ? 0 : 1);
}
