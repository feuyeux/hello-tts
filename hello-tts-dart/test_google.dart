import 'dart:io';
import 'lib/tts_processor.dart';

void main() async {
  print('🔍 Testing Dart Google TTS Backend');
  print('===================================');
  
  try {
    final processor = TTSProcessor(backend: 'google');
    
    print('📝 Testing with English text...');
    final result = await processor.generateAudio(
      'Hello from Dart Google TTS test',
      voiceName: 'en-US-AriaNeural', // This will be converted to 'en' for Google
      language: 'en-us',
      outputPath: 'output/dart_google_test.mp3'
    );
    
    if (result) {
      print('✅ Success! Audio generated');
      print('📁 Output: output/dart_google_test.mp3');
    } else {
      print('❌ Failed to generate audio');
    }
  } catch (e) {
    print('💥 Error: $e');
  }
}
