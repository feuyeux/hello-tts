from typing import Optional
import platform
import shutil
import subprocess
import tempfile
import os


class AudioError(Exception):
    pass


class AudioPlayer:
    def __init__(self, preferred_backend: Optional[str] = None):
        self.preferred_backend = preferred_backend
        self.backend = self._choose_backend()

    def _choose_backend(self) -> str:
        system = platform.system()
        if system == 'Darwin' and shutil.which('afplay'):
            return 'afplay'
        if system == 'Linux' and shutil.which('aplay'):
            return 'aplay'

        try:
            import pygame  # type: ignore
            return 'pygame'
        except Exception:
            pass
        try:
            from playsound import playsound  # type: ignore
            return 'playsound'
        except Exception:
            pass

        raise AudioError('No audio playback method available')

    def play_file(self, filename: str) -> None:
        if not os.path.exists(filename):
            raise AudioError(f'File not found: {filename}')

        if self.backend == 'afplay':
            subprocess.run(['afplay', filename], check=True)
        elif self.backend == 'aplay':
            subprocess.run(['aplay', filename], check=True)
        elif self.backend == 'pygame':
            import pygame  # type: ignore
            pygame.mixer.init()
            snd = pygame.mixer.Sound(filename)
            snd.play()
            while pygame.mixer.get_busy():
                pygame.time.wait(100)
            pygame.mixer.quit()
        elif self.backend == 'playsound':
            from playsound import playsound  # type: ignore
            playsound(filename)
        else:
            raise AudioError(f'Unknown backend: {self.backend}')

    def play_audio_data(self, audio_data: bytes, format_hint: str = 'mp3') -> None:
        with tempfile.NamedTemporaryFile(suffix=f'.{format_hint}', delete=False) as f:
            f.write(audio_data)
            fname = f.name
        try:
            self.play_file(fname)
        finally:
            try:
                os.unlink(fname)
            except Exception:
                pass
