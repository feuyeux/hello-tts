# hello_tts_python

This package consolidates the Microsoft Edge TTS and Google gTTS backends into a
single, easy-to-use facade. It provides a small CLI and an importable API.

1. Create and activate a virtual environment

```bash
cd hello_tts_python
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
```

2. CLI usage

````bash
source .venv/bin/activate

```bash
python hello_tts.py --backend edge --voice "en-US-JennyNeural" --text "Hello"
python hello_tts.py --backend google --voice "en" --text "Hello"
````

```bash
edge-tts -h
gtts-cli -h
```

References

- https://github.com/rany2/edge-tts
- https://github.com/pndurette/gTTS
