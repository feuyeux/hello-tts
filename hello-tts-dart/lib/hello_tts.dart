/// Hello Edge TTS - Dart Implementation
/// 
/// This library provides a Dart interface for Microsoft Edge TTS service.
library hello_tts;

export 'tts_client.dart';
export 'tts_processor.dart';
export 'config_manager.dart';
export 'audio_player.dart';

/// Voice class representing a TTS voice
class Voice {
  final String name;
  final String displayName;
  final String language;
  final String gender;
  final String locale;
  final bool isNeural;
  final bool isStandard;

  const Voice({
    required this.name,
    required this.displayName,
    required this.language,
    required this.gender,
    required this.locale,
    this.isNeural = false,
    this.isStandard = false,
  });

  factory Voice.fromJson(Map<String, dynamic> json) {
    final locale = json['Locale'] ?? '';
    final language = locale.split('-')[0]; // Extract language code from locale
    
    return Voice(
      name: json['Name'] ?? '',
      displayName: json['DisplayName'] ?? '',
      language: language,
      gender: json['Gender'] ?? '',
      locale: locale,
      isNeural: (json['VoiceType'] ?? '').contains('Neural'),
      isStandard: (json['VoiceType'] ?? '').contains('Standard'),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'Name': name,
      'DisplayName': displayName,
      'Locale': language,
      'Gender': gender,
      'VoiceType': isNeural ? 'Neural' : 'Standard',
    };
  }

  @override
  String toString() {
    return 'Voice(name: $name, displayName: $displayName, language: $language, gender: $gender)';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is Voice && other.name == name;
  }

  @override
  int get hashCode => name.hashCode;
}

/// TTS Error class
class TTSError extends Error {
  final String message;
  final String? code;
  final dynamic originalError;

  TTSError(this.message, {this.code, this.originalError});

  @override
  String toString() {
    return 'TTSError: $message${code != null ? ' (Code: $code)' : ''}';
  }
}