#!/usr/bin/env dart

import 'dart:io';
import 'package:args/args.dart';
import 'package:path/path.dart' as path;
import 'package:logging/logging.dart';
import '../lib/hello_tts.dart';

/// Create output directory if it doesn't exist
Future<void> createOutputDirectory(String directory) async {
  final dir = Directory(directory);
  if (!await dir.exists()) {
    await dir.create(recursive: true);
  }
}

/// Generate a safe filename from text
String getSafeFilename(String text, {int maxLength = 50}) {
  // Remove or replace unsafe characters
  const safeChars =
      'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_. ';
  final filename =
      text.split('').map((c) => safeChars.contains(c) ? c : '_').join('');

  // Truncate if too long
  final truncated =
      filename.length > maxLength ? filename.substring(0, maxLength) : filename;

  // Remove trailing spaces and dots
  return truncated.replaceAll(RegExp(r'[\s.]+$'), '');
}

/// Display available voices grouped by language
Future<void> displayVoicesByLanguage(TTSProcessor client,
    {String? filterLanguage}) async {
  try {
    final voices = await client.getVoices();

    // Group voices by language
    final Map<String, List<Voice>> voicesByLanguage = <String, List<Voice>>{};

    for (final voice in voices) {
      final lang = voice.language;
      voicesByLanguage.putIfAbsent(lang, () => []).add(voice);
    }

    final logger = Logger('displayVoicesByLanguage');
    if (filterLanguage != null) {
      if (voicesByLanguage.containsKey(filterLanguage)) {
        final langVoices = voicesByLanguage[filterLanguage]!;
        logger.info(
            '${filterLanguage.toUpperCase()} Voices (${langVoices.length} voices):');

        // Show first 5 voices
        for (final voice in langVoices.take(5)) {
          logger
              .info('  ${voice.name} - ${voice.displayName} (${voice.gender})');
        }

        if (langVoices.length > 5) {
          logger.info('  ... and ${langVoices.length - 5} more voices');
        }
      } else {
        logger.warning('No voices found for language: $filterLanguage');
      }
    } else {
      // Group voices by language
      logger.info('Available voices by language:');

      for (final entry in voicesByLanguage.entries) {
        final lang = entry.key;
        final langVoices = entry.value;
        logger.info('${lang.toUpperCase()} (${langVoices.length} voices):');

        // Show first 5 voices
        for (final voice in langVoices.take(5)) {
          logger
              .info('  ${voice.name} - ${voice.displayName} (${voice.gender})');
        }

        if (langVoices.length > 5) {
          logger.info('  ... and ${langVoices.length - 5} more voices');
        }
      }
    }
  } catch (e) {
    final logger = Logger('displayVoicesByLanguage');
    logger.severe('Error displaying voices: $e');
  }
}

/// Main function
Future<void> main(List<String> arguments) async {
  final parser = ArgParser()
    ..addOption('text', abbr: 't', help: 'Text to synthesize', defaultsTo: 'Hello World! This is a demonstration of TTS in Dart.')
    ..addOption('voice',
        abbr: 'v', help: 'Voice to use', defaultsTo: 'en-US-AriaNeural')
    ..addOption('backend',
        abbr: 'b',
        help: 'TTS backend to use',
        defaultsTo: 'edge',
        allowed: ['edge', 'google'])
    ..addOption('output', abbr: 'o', help: 'Output file path')
    ..addOption('format',
        abbr: 'f',
        help: 'Audio format',
        defaultsTo: 'mp3',
        allowed: ['mp3', 'wav', 'ogg'])
    ..addOption('rate', abbr: 'r', help: 'Speech rate', defaultsTo: 'medium')
    ..addOption('pitch', abbr: 'p', help: 'Speech pitch', defaultsTo: 'medium')
    ..addOption('volume', help: 'Speech volume', defaultsTo: 'medium')
    ..addOption('language', abbr: 'l', help: 'Filter voices by language')
    ..addOption('config', abbr: 'c', help: 'Configuration file path')
    ..addOption('output-dir',
        help: 'Output directory for batch operations', defaultsTo: 'output')
    ..addFlag('list-voices', help: 'List available voices', negatable: false)
    ..addFlag('play',
        help: 'Play audio after synthesis (default: true)', defaultsTo: true)
    ..addFlag('no-play',
        help: 'Don\'t play audio after synthesis', negatable: false)
    ..addFlag('help', abbr: 'h', help: 'Show help', negatable: false)
    ..addFlag('verbose', help: 'Verbose output', negatable: false);

  try {
    final results = parser.parse(arguments);

    final rootLogger = Logger.root;
    rootLogger.level = Level.INFO;
    rootLogger.onRecord.listen((record) {
      // Unified log formatter: ISO timestamp, level, loggerName, message
      final ts = DateTime.fromMillisecondsSinceEpoch(
              record.time.millisecondsSinceEpoch,
              isUtc: true)
          .toIso8601String();
      stderr.writeln(
          '$ts [${record.level.name}] ${record.loggerName}: ${record.message}');
    });

    if (results['help'] as bool) {
      final helpLogger = Logger('help');
      helpLogger.info('Hello TTS - Dart Implementation');
      helpLogger.info('=================================');
      helpLogger.info('Supports both Edge TTS and Google TTS backends');
      helpLogger.info('');
      helpLogger.info('Usage: dart run bin/main.dart [options]');
      helpLogger.info('');
      helpLogger.info(parser.usage);
      helpLogger.info('');
      helpLogger.info('Examples:');
      helpLogger.info('  # Edge TTS examples');
      helpLogger.info(
          '  dart run bin/main.dart --backend edge --text "Hello World" --voice "en-US-JennyNeural"');
      helpLogger.info(
          '  dart run bin/main.dart --backend edge --text "你好世界" --voice "zh-CN-XiaoxiaoNeural"');
      helpLogger.info('  # Google TTS examples');
      helpLogger.info(
          '  dart run bin/main.dart --backend google --text "Hello World" --voice "en"');
      helpLogger.info(
          '  dart run bin/main.dart --backend google --text "你好世界" --voice "zh"');
      helpLogger.info('  # Voice listing');
      helpLogger.info('  dart run bin/main.dart --list-voices');
      helpLogger.info('  dart run bin/main.dart --list-voices --language en');
      return;
    }

    // Initialize TTS client with selected backend
    final backend = results['backend'] as String;
    final client = TTSProcessor(backend: backend);

    // Load configuration if provided
    if (results['config'] != null) {
      final configManager = ConfigManager();
      await configManager.loadFromFile(results['config'] as String);
      // Apply configuration to client
    }

    // Create output directory
    final outputDir = results['output-dir'] as String;
    await createOutputDirectory(outputDir);

    // List voices
    if (results['list-voices'] as bool) {
      await displayVoicesByLanguage(client,
          filterLanguage: results['language'] as String?);
      return;
    }

    // Synthesize text
    final text = results['text'] as String?;
    final voice = results['voice'] as String;
    final format = results['format'] as String;
    final play = results['play'] as bool && !(results['no-play'] as bool);

    // text will now have a default value, so this check is no longer needed
    // but we keep it for safety in case the default is somehow null
    if (text == null || text.isEmpty) {
      final logger = Logger('main');
      logger.severe('Error: --text must be provided');
      logger.info('Use --help for usage information');
      exit(1);
    }

    try {
      final logger = Logger('main');
      logger.info('🎤 Synthesizing speech...');
      logger.info('Backend: $backend');
      logger.info('Voice: $voice');
      logger.info('Format: $format');
      logger.info('Text: $text');

      final audioData =
          await client.synthesizeText(text, voice, format: format);

      // Determine output path
      String outputPath;
      if (results['output'] != null) {
        outputPath = results['output'] as String;
      } else {
        // Extract language from voice (e.g., 'en' from 'en-US-AriaNeural')
        final lang = voice.split('-')[0];
        final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
        final filename = 'edgetts_${lang}_dart_$timestamp.mp3';
        outputPath = path.join(outputDir, filename);
      }

      // Save audio file
      final file = File(outputPath);
      await file.writeAsBytes(audioData);

      logger.info('✅ Audio saved to: $outputPath');
      logger.info(
          '📊 File size: ${(audioData.length / 1024).toStringAsFixed(1)} KB');

      // Play audio (default behavior, unless --no-play is specified)
      if (play) {
        try {
          logger.info('🔊 Playing audio...');
          final player = AudioPlayer();
          await player.play(outputPath);
          logger.info('✅ Playback completed!');
        } catch (e) {
          logger.warning('⚠️  Could not play audio: $e');
          logger.info('💡 Audio file was saved successfully though.');
        }
      }
    } catch (e) {
      final logger = Logger('main');
      logger.severe('❌ Error during synthesis: $e');
      exit(1);
    }
  } catch (e) {
    final logger = Logger('main');
    logger.severe('❌ Error parsing arguments: $e');
    logger.info('Use --help for usage information');
    exit(1);
  }
}
