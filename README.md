# YASTwAI

Yet Another Subtitles Translation with AI - A Rust application for extracting and automatically translating video subtitles.

## Overview

YASTwAI is a command-line tool designed to extract subtitles from video files and translate them using AI translation services. The application maintains the format integrity of subtitles while providing a seamless translation experience. It uses a direct approach to translate subtitles without intermediate JSON representations, ensuring better translation quality and preserving formatting.

## Features

- Extract subtitles from video files (SRT format)
- Translate subtitles using AI translation services with a direct approach
- Preserve subtitle formatting and timing
- Configurable translation settings
- Concurrent processing for efficient translation
- Progress tracking for long-running operations

## Installation

### Prerequisites

- Rust and Cargo (1.85.0 or newer)
- FFmpeg (for subtitle extraction - currently placeheld in the implementation)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/yastwai.git
cd yastwai

# Build the application
cargo build --release

# The executable will be in target/release/yastwai
```

## Usage

1. Copy the example configuration file (conf.example.json) to create your own configuration or let the application create a default one on first run:

```bash
# Copy the example configuration
cp conf.example.json conf.json
```

The configuration file contains settings for the translation process:

```json
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama",
    "ollama": {
      "model": "mixtral:8x7b",
      "endpoint": "http://localhost:11434",
      "concurrent_requests": 8,
      "max_chars_per_request": 1000,
      "timeout_secs": 30
    },
    "openai": {
      "model": "gpt-4o-mini",
      "api_key": "",
      "endpoint": "https://api.openai.com/v1",
      "concurrent_requests": 5,
      "max_chars_per_request": 1500,
      "timeout_secs": 20
    },
    "anthropic": {
      "model": "claude-3-5-haiku",
      "api_key": "",
      "endpoint": "https://api.anthropic.com",
      "concurrent_requests": 3,
      "max_chars_per_request": 2000,
      "timeout_secs": 25
    },
    "common": {
      "system_prompt": "You are a professional translator specialized in subtitle adaptation. Your task is to translate while STRICTLY preserving the provided format. Your translation should be fluid and natural while respecting the original line structure. ALWAYS respond in plain text format only, never in markdown or other formatting."
    }
  },
  "log_level": "info"
}
```

2. Run the application:

```bash
# Specify the input video file as a command-line argument
./target/release/yastwai video.mkv

# With environment variable RUST_LOG for logging
RUST_LOG=debug ./target/release/yastwai video.mkv
```

## Configuration Options

| Option | Description |
|--------|-------------|
| `source_language` | Source language code (e.g., "en") |
| `target_language` | Target language code (e.g., "fr") |
| `translation.provider` | Translation provider: "ollama" (default), "openai", or "anthropic" |
| `translation.ollama.model` | Model name for Ollama translation (e.g., "mixtral:8x7b") |
| `translation.ollama.endpoint` | Endpoint URL for Ollama server |
| `translation.ollama.concurrent_requests` | Maximum number of concurrent requests for Ollama |
| `translation.ollama.max_chars_per_request` | Maximum characters per request for Ollama |
| `translation.ollama.timeout_secs` | Request timeout in seconds for Ollama |
| `translation.openai.model` | Model name for OpenAI translation (e.g., "gpt-4o-mini") |
| `translation.openai.api_key` | API key for OpenAI service |
| `translation.openai.endpoint` | Endpoint URL for OpenAI API |
| `translation.openai.concurrent_requests` | Maximum number of concurrent requests for OpenAI |
| `translation.openai.max_chars_per_request` | Maximum characters per request for OpenAI |
| `translation.openai.timeout_secs` | Request timeout in seconds for OpenAI |
| `translation.anthropic.model` | Model name for Anthropic translation (e.g., "claude-3-5-haiku") |
| `translation.anthropic.api_key` | API key for Anthropic service |
| `translation.anthropic.endpoint` | Endpoint URL for Anthropic API |
| `translation.anthropic.concurrent_requests` | Maximum number of concurrent requests for Anthropic |
| `translation.anthropic.max_chars_per_request` | Maximum characters per request for Anthropic |
| `translation.anthropic.timeout_secs` | Request timeout in seconds for Anthropic |
| `translation.common.system_prompt` | Prompt template for translation with placeholders for languages |
| `log_level` | Logging level (error, warn, info, debug, trace) |

## Integration with Translation Services

The application supports multiple translation providers:

1. **Ollama** (default): Local LLM service - no API key required
2. **OpenAI**: Requires an API key
3. **Anthropic**: Requires an API key

You'll need to:
1. Select your desired provider in the configuration
2. Configure the appropriate API key (if needed)
3. Adjust the model settings if necessary

## Translation Approach

YASTwAI uses a direct translation approach that:
- Preserves formatting and structure of subtitles
- Maintains subtitle timing information
- Uses a structured prompt format for reliable translations
- Processes subtitles in batches for efficiency
- Clearly delineates individual subtitle entries for precise translation

## Limitations and Future Work

- Currently only supports SRT subtitle format
- Extraction from video files is not fully implemented (placeholder)
- Limited error recovery for failed translations
- No support for multiple languages in a single run

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgements

- Thanks to all the open-source libraries used in this project
- Inspired by various subtitle translation tools and the need for an AI-powered solution
