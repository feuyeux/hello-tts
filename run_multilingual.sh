#!/bin/bash

# If the script was invoked using 'bash -x' (xtrace enabled), turn it off
# so callers who accidentally run with -x don't get noisy '+ ...' traces.
if [[ $- == *x* ]]; then
    set +x
fi

# Enhanced Multilingual TTS Demo Script
# Supports individual language testing and multiple TTS backends

# Do not exit on error, allow all implementations to run

# Default values
IMPLEMENTATION=""
BACKEND="edge"
LANGUAGE=""
SHOW_HELP=false
# Verbosity: default concise; set VERBOSE=true to enable detailed logs
VERBOSE=false

# Function to show usage
show_usage() {
    echo "Enhanced Multilingual TTS Demo"
    echo "=============================="
    echo
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -i, --implementation IMPL    Specify implementation (python, dart, java, rust, all)"
    echo "  -b, --backend BACKEND        Specify TTS backend (edge, google) [default: edge]"
    echo "  -l, --language LANG          Specify language code (e.g., en-us, zh-cn)"
    echo "  -h, --help                   Show this help message"
    echo "  -v, --verbose                Verbose mode: show detailed logs"
    echo
    echo "Examples:"
    echo "  $0                           # Run all implementations with edge backend"
    echo "  $0 -i java                   # Run only Java implementation"
    echo "  $0 -i java -b google         # Run Java with Google TTS"
    echo "  $0 -i python -l en-us        # Run Python for English only"
    echo "  $0 -b google                 # Run all implementations with Google TTS"
    echo
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--implementation)
            IMPLEMENTATION="$2"
            shift 2
            ;;
        -b|--backend)
            BACKEND="$2"
            shift 2
            ;;
        -l|--language)
            LANGUAGE="$2"
            shift 2
            ;;
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Show help if requested
if [ "$SHOW_HELP" = true ]; then
    show_usage
    exit 0
fi

# Validate backend
if [[ "$BACKEND" != "edge" && "$BACKEND" != "google" ]]; then
    echo "ERROR: Backend must be 'edge' or 'google'"
    exit 1
fi

# Validate implementation
if [[ -n "$IMPLEMENTATION" && "$IMPLEMENTATION" != "python" && "$IMPLEMENTATION" != "dart" && "$IMPLEMENTATION" != "java" && "$IMPLEMENTATION" != "rust" && "$IMPLEMENTATION" != "all" ]]; then
    echo "ERROR: Implementation must be one of: python, dart, java, rust, all"
    exit 1
fi

# Set default to all if not specified
if [[ -z "$IMPLEMENTATION" ]]; then
    IMPLEMENTATION="all"
fi

if [ "$VERBOSE" = true ]; then
    echo "Multilingual TTS demo — Implementation: $IMPLEMENTATION | Backend: $BACKEND"
fi


# Determine available config files (prefer unified tts_config.json)
if [ -f "shared/tts_config.json" ]; then
    CONFIG_PREFERRED="shared/tts_config.json"
elif [ -f "shared/multilingual_config.json" ]; then
    CONFIG_PREFERRED="shared/multilingual_config.json"
elif [ -f "shared/edge_tts_voices.json" ] || [ -f "shared/google_tts_voices.json" ]; then
    CONFIG_PREFERRED="fallback"
else
    echo "ERROR: No voice configuration found in shared/. Expected shared/tts_config.json or legacy config files"
    exit 1
fi

# Load language definitions
load_languages() {
    if [[ "$CONFIG_PREFERRED" == "shared/tts_config.json" ]]; then
        LANGUAGES=()
        while IFS= read -r line; do
            LANGUAGES+=("$line")
        done < <(python3 - <<'PY'
import json
with open('shared/tts_config.json','r',encoding='utf-8') as f:
    j=json.load(f)
    for v in j.get('languages',[]):
        code=v.get('code')
        name=v.get('name','')
        flag=v.get('flag','')
        text=v.get('text','')
        edge=v.get('edge_voice','')
        google=v.get('google_voice','')
        if code:
            print(f"{code}||{name}||{flag}||{text}||{edge}||{google}")
PY
)
    elif [[ "$CONFIG_PREFERRED" == "shared/multilingual_config.json" ]]; then
        LANGUAGES=()
        while IFS= read -r line; do
            LANGUAGES+=("$line")
        done < <(python3 - <<'PY'
import json
with open('shared/multilingual_config.json','r',encoding='utf-8') as f:
    j=json.load(f)
    for v in j.get('languages',[]):
        code=v.get('code')
        name=v.get('name','')
        flag=v.get('flag','')
        text=v.get('text','')
        voice=v.get('voice','')
        alt=v.get('alt_voice','') or ''
        if code:
            print(f"{code}||{name}||{flag}||{text}||{voice}||{alt}")
PY
)
    else
        # fallback: per-backend files
        if [[ "$BACKEND" == "google" && -f "shared/google_tts_voices.json" ]]; then
            LANGUAGES=()
            while IFS= read -r line; do
                LANGUAGES+=("$line")
            done < <(python3 - <<'PY'
import json
with open('shared/google_tts_voices.json','r',encoding='utf-8') as f:
    j=json.load(f)
    for v in j.get('voices',[]):
        code=v.get('code')
        name=v.get('name','')
        if code:
            print(f"{code}||{name}||||")
PY
)
        elif [[ -f "shared/edge_tts_voices.json" ]]; then
            LANGUAGES=()
            while IFS= read -r line; do
                LANGUAGES+=("$line")
            done < <(python3 - <<'PY'
import json
with open('shared/edge_tts_voices.json','r',encoding='utf-8') as f:
    j=json.load(f)
    for v in j.get('languages',[]):
        code=v.get('code')
        name=v.get('name','')
        flag=v.get('flag','')
        text=v.get('text','')
        voice=v.get('voice','')
        alt=v.get('alt_voice','') or ''
        if code:
            print(f"{code}||{name}||{flag}||{text}||{voice}||{alt}")
PY
)
        else
            LANGUAGES=()
        fi
    fi

    LANG_COUNT=${#LANGUAGES[@]}
    if [[ $LANG_COUNT -eq 0 ]]; then
        echo "WARNING: No languages loaded from configuration"
    else
        if [ "$VERBOSE" = true ]; then
            echo "📋 $LANG_COUNT languages loaded"
        fi
    fi
}

# Populate LANGUAGES array now (used for informational purposes and future per-language runs)
load_languages

# Initialize counters
python_success=0
dart_success=0
java_success=0
rust_success=0

# Function to run Python implementation
run_python() {
    if [ "$VERBOSE" = true ]; then echo "[python] running..."; fi
    cd hello-tts-python
    if command -v python3 >/dev/null 2>&1; then
        export TTS_BACKEND="$BACKEND"
        local cmd="python3 multilingual_demo.py --backend $BACKEND"
        if [[ -n "$LANGUAGE" ]]; then
            cmd="$cmd --language $LANGUAGE"
        fi
        if eval "$cmd"; then python_success=1; else python_success=0; fi
    else
        python_success=0
    fi
    cd ..
}

# Function to run Dart implementation
run_dart() {
    cd hello-tts-dart
    if command -v dart >/dev/null 2>&1; then
        export TTS_BACKEND="$BACKEND"
    if [ "$VERBOSE" = true ]; then
        dart pub get
    else
        dart pub get >/dev/null 2>&1
    fi
    local cmd="dart run bin/multilingual_demo.dart --backend $BACKEND"
        if [[ -n "$LANGUAGE" ]]; then
            cmd="$cmd --language $LANGUAGE"
        fi
    if [ "$VERBOSE" = true ]; then
        if eval "$cmd"; then dart_success=1; else dart_success=0; fi
    else
        if eval "$cmd" >/dev/null 2>&1; then dart_success=1; else dart_success=0; fi
    fi
    else
    dart_success=0
    fi
    cd ..
}

# Function to run Java implementation
run_java() {
    cd hello-tts-java
    if command -v java >/dev/null 2>&1 && command -v mvn >/dev/null 2>&1; then
        if mvn compile -q >/dev/null 2>&1; then
            export TTS_BACKEND="$BACKEND"
            local cp=$(mvn dependency:build-classpath -q -Dmdep.outputFile=/dev/stdout 2>/dev/null)
            local cmd="java -Dtts.backend=\"$BACKEND\" -cp target/classes:${cp} org.feuyeux.edgetts.MultilingualDemo"
            if [ "$VERBOSE" = true ]; then
                if eval "$cmd"; then java_success=1; else java_success=0; fi
            else
                if eval "$cmd" >/dev/null 2>&1; then java_success=1; else java_success=0; fi
            fi
        else
            java_success=0
        fi
    else
        java_success=0
    fi
    cd ..
}

# Function to run Rust implementation
run_rust() {
    cd hello-tts-rust
    if command -v cargo >/dev/null 2>&1; then
        if [ "$VERBOSE" = true ]; then
            if cargo build --example multilingual_demo -q >/dev/null 2>&1; then
                export TTS_BACKEND="$BACKEND"
                local cmd="cargo run --example multilingual_demo -- --backend $BACKEND"
                if [[ -n "$LANGUAGE" ]]; then cmd="$cmd --language $LANGUAGE"; fi
                if eval "$cmd"; then rust_success=1; else rust_success=0; fi
            else
                rust_success=0
            fi
        else
            if cargo build --example multilingual_demo -q >/dev/null 2>&1; then
                export TTS_BACKEND="$BACKEND"
                local cmd="cargo run --example multilingual_demo -- --backend $BACKEND"
                if [[ -n "$LANGUAGE" ]]; then cmd="$cmd --language $LANGUAGE"; fi
                if eval "$cmd" >/dev/null 2>&1; then rust_success=1; else rust_success=0; fi
            else
                rust_success=0
            fi
        fi
    else
        rust_success=0
    fi
    cd ..
}

# Run implementations based on selection
case $IMPLEMENTATION in
    "python")
        run_python
        ;;
    "dart")
        run_dart
        ;;
    "java")
        run_java
        ;;
    "rust")
        run_rust
        ;;
    "all")
        run_python
        run_dart
        run_java
        run_rust
        ;;
esac

# Summary
total=$((python_success + dart_success + java_success + rust_success))
echo
echo "Results: python=$python_success dart=$dart_success java=$java_success rust=$rust_success"
if [ $total -gt 0 ]; then
    echo "One or more implementations succeeded"
    exit 0
else
    echo "All selected implementations failed"
    exit 1
fi
