import re
from pathlib import Path


def create_output_directory(path: str) -> None:
    Path(path).mkdir(parents=True, exist_ok=True)


def get_safe_filename(text: str, max_length: int = 50, ext: str = 'mp3') -> str:
    safe = re.sub(r'[^a-zA-Z0-9_.-]', '_', text)[:max_length]
    if not safe:
        safe = 'output'
    return f"{safe}.{ext}"
