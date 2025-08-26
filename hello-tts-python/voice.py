from dataclasses import dataclass
import json
import os
from typing import List, Optional


@dataclass
class Voice:
    name: str
    display_name: str = ''
    locale: str = ''
    gender: str = 'unknown'
    description: Optional[str] = None

    @classmethod
    def parse_voices_from_json_file(cls, json_file_path: str) -> List['Voice']:
        """Parse voices from a JSON file containing voice configuration data."""
        if not os.path.exists(json_file_path):
            raise FileNotFoundError(
                f"Voice config file not found: {json_file_path}")

        with open(json_file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)

        voices = []
        languages = data.get('languages', [])

        for lang_data in languages:
            edge_voice = lang_data.get('edge_voice')
            if edge_voice:
                voice = cls(
                    name=edge_voice,
                    display_name=lang_data.get('name', ''),
                    locale=lang_data.get('code', ''),
                    gender='Unknown',
                    description=None
                )
                voices.append(voice)

        return voices

    @classmethod
    def get_voices_by_language(cls, voices: List['Voice'], language: str) -> List['Voice']:
        """Filter voices by language code."""
        normalized_language = language.lower().strip()
        return [v for v in voices if v.locale.lower().startswith(normalized_language)]

    @classmethod
    def get_voices_by_locale(cls, voices: List['Voice'], locale: str) -> List['Voice']:
        """Filter voices by full locale."""
        normalized_locale = locale.lower().strip()
        return [v for v in voices if v.locale.lower() == normalized_locale]

    @classmethod
    def get_voices_by_gender(cls, voices: List['Voice'], gender: str) -> List['Voice']:
        """Filter voices by gender."""
        normalized_gender = gender.lower().strip()
        return [v for v in voices if v.gender.lower() == normalized_gender]

    def language_code(self) -> str:
        """Get language code from locale (e.g., 'en' from 'en-US')."""
        return self.locale.split('-')[0] if '-' in self.locale else self.locale

    def matches_language(self, language: str) -> bool:
        """Check if this voice matches the given language code."""
        return self.locale.lower() == language.lower() or self.language_code().lower() == language.lower()
