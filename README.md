# YASTwAI

<!-- AI-METADATA
project_name: YASTwAI
project_description: Yet Another Subtitles Translator with AI
project_type: CLI tool
language: Rust
primary_purpose: Subtitle translation
license: MIT
-->

Yet Another Subtitles Translator with AI - A Rust application for extracting and automatically translating video subtitles.

## Overview

<!-- SECTION: overview -->
YASTwAI is a command-line tool designed to extract subtitles from video files and translate them using AI translation services. 
The application tries to maintain the format integrity of subtitles while providing a seamless translation experience.
<!-- END_SECTION: overview -->

## CI/CD Status

<!-- SECTION: ci-cd -->
GitHub Actions workflows have been temporarily removed and will be reintroduced in the future with an improved CI/CD pipeline. The planned checks will include:

- **Tests**: Running the full test suite
- **Clippy**: Running Rust's linting tool to ensure code quality
- **Build**: Verifying the project builds successfully on multiple platforms (Ubuntu, macOS, Windows)
- **PR Validation**: Ensuring PR title and description meet project standards

These checks will help maintain code quality and ensure that all contributions meet project standards.
<!-- END_SECTION: ci-cd -->

## Features

<!-- SECTION: features -->
- Extract subtitles from video files (SRT format)
- Translate subtitles using AI translation services with a direct approach
- Preserve subtitle formatting and timing
- Support for multiple translation providers (Ollama, OpenAI, Anthropic)
- ISO 639-1 and ISO 639-2 language code support
- Configurable translation settings with sensible defaults
- Concurrent processing for efficient translation
- Progress tracking for long-running operations
- Batch processing for translating multiple files
- Translated subtitles are always saved next to the original video files
- Option to force overwriting existing translations
<!-- END_SECTION: features -->

## Installation

<!-- SECTION: prerequisites -->
### Prerequisites

- Rust and Cargo (1.85.0 or newer)
- FFmpeg installed on your system (for subtitle extraction from video files)
<!-- END_SECTION: prerequisites -->

<!-- SECTION: installation -->
### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/yastwai.git
cd yastwai

# Build the application
cargo build --release

# The executable will be in target/release/yastwai
```
<!-- END_SECTION: installation -->

## Usage

<!-- SECTION: usage -->
1. Copy the example configuration file (conf.example.json) to create your own configuration or let the application create a default one on first run:

```bash
# Copy the example configuration
cp conf.example.json conf.json
```

2. Run the application:

```bash
# For a single video file
./target/release/yastwai video.mkv

# For a directory of video files
./target/release/yastwai /path/to/videos

# Force overwrite of existing translations
./target/release/yastwai -f video.mkv
./target/release/yastwai --force /path/to/videos

# With environment variable RUST_LOG for logging
RUST_LOG=debug ./target/release/yastwai video.mkv
```

The translated subtitles will be saved next to the original video files with a filename format of `original_name.{target_language}.srt`. By default, the tool will skip any video that already has a subtitle file in the target language. Use the `-f` or `--force` flag to overwrite existing translations.
<!-- END_SECTION: usage -->

## Configuration

<!-- SECTION: configuration -->
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
      "concurrent_requests": 2,
      "max_chars_per_request": 1000,
      "timeout_secs": 60
    },
    "openai": {
      "model": "gpt-4o-mini",
      "api_key": "your_api_key",
      "endpoint": "https://api.openai.com/v1",
      "concurrent_requests": 10,
      "max_chars_per_request": 2000,
      "timeout_secs": 60
    },
    "anthropic": {
      "model": "claude-3-haiku-20240307",
      "api_key": "your_api_key",
      "endpoint": "https://api.anthropic.com",
      "concurrent_requests": 2,
      "max_chars_per_request": 400,
      "timeout_secs": 60
    },
    "common": {
      "system_prompt": "You are a professional translator. Translate the following text from {source_language} to {target_language}. You MUST preserve formatting and maintain the original meaning and tone.",
      "rate_limit_delay_ms": 3000,
      "retry_count": 3,
      "retry_backoff_ms": 3000
    }
  },
  "log_level": "info",
  "batch_size": 1000
}
```
<!-- END_SECTION: configuration -->

## Configuration Options

<!-- SECTION: configuration_options -->
| Option | Description | Type | Default |
|--------|-------------|------|---------|
| `source_language` | Source language code (e.g., "en") | string | "en" |
| `target_language` | Target language code (e.g., "fr") | string | "fr" |
| `translation.provider` | Translation provider: "ollama" (default), "openai", or "anthropic" | string | "ollama" |
| `translation.ollama.model` | Model name for Ollama translation | string | "mixtral:8x7b" |
| `translation.ollama.endpoint` | Endpoint URL for Ollama server | string | "http://localhost:11434" |
| `translation.ollama.concurrent_requests` | Maximum number of concurrent requests for Ollama | integer | 2 |
| `translation.ollama.max_chars_per_request` | Maximum characters per request for Ollama | integer | 1000 |
| `translation.ollama.timeout_secs` | Request timeout in seconds for Ollama | integer | 60 |
| `translation.openai.model` | Model name for OpenAI translation | string | "gpt-4o-mini" |
| `translation.openai.api_key` | API key for OpenAI service | string | "" |
| `translation.openai.endpoint` | Endpoint URL for OpenAI API | string | "https://api.openai.com/v1" |
| `translation.openai.concurrent_requests` | Maximum number of concurrent requests for OpenAI | integer | 10 |
| `translation.openai.max_chars_per_request` | Maximum characters per request for OpenAI | integer | 2000 |
| `translation.openai.timeout_secs` | Request timeout in seconds for OpenAI | integer | 60 |
| `translation.anthropic.model` | Model name for Anthropic translation | string | "claude-3-haiku-20240307" |
| `translation.anthropic.api_key` | API key for Anthropic service | string | "" |
| `translation.anthropic.endpoint` | Endpoint URL for Anthropic API | string | "https://api.anthropic.com" |
| `translation.anthropic.concurrent_requests` | Maximum number of concurrent requests for Anthropic | integer | 2 |
| `translation.anthropic.max_chars_per_request` | Maximum characters per request for Anthropic | integer | 400 |
| `translation.anthropic.timeout_secs` | Request timeout in seconds for Anthropic | integer | 60 |
| `translation.common.system_prompt` | Prompt template for translation with placeholders for languages | string | *See config* |
| `translation.common.rate_limit_delay_ms` | Delay between API requests in milliseconds to avoid rate limiting | integer | 3000 |
| `translation.common.retry_count` | Number of retries for failed API requests | integer | 3 |
| `translation.common.retry_backoff_ms` | Backoff time in milliseconds between retries | integer | 3000 |
| `log_level` | Logging level (error, warn, info, debug, trace) | string | "info" |
| `batch_size` | Number of characters to process in a single batch | integer | 1000 |
<!-- END_SECTION: configuration_options -->

## Integration with Translation Services

<!-- SECTION: integration -->
The application supports multiple translation providers:

1. **Ollama** (default): Local LLM service - no API key required
   - Install Ollama from https://ollama.ai/
   - Start the Ollama service locally
   - Pull your preferred model: `ollama pull mixtral:8x7b`

2. **OpenAI**: Requires an API key from https://platform.openai.com/
   - Set your API key in the configuration file
   - Choose from available models like gpt-4o-mini, gpt-4o, etc.

3. **Anthropic**: Requires an API key from https://www.anthropic.com/
   - Set your API key in the configuration file
   - Choose from available models like claude-3-haiku, claude-3-sonnet, etc.
<!-- END_SECTION: integration -->

## Project Structure

<!-- SECTION: project_structure -->
The codebase is organized in these main modules:

- `app_config`: Configuration management
- `subtitle_processor`: Subtitle file handling and processing
- `translation_service`: AI-powered translation services
- `file_utils`: File system operations
- `app_controller`: Main application controller
- `language_utils`: ISO language code utilities
<!-- END_SECTION: project_structure -->

## Translation Approach

<!-- SECTION: translation_approach -->
YASTwAI uses a direct translation approach that:
- Preserves formatting and structure of subtitles
- Maintains subtitle timing information
- Uses a structured prompt format for reliable translations
- Processes subtitles in batches for efficiency
- Clearly delineates individual subtitle entries for precise translation
<!-- END_SECTION: translation_approach -->

## Limitations and Future Work

<!-- SECTION: limitations -->
- Currently only supports SRT subtitle format
- Extraction from video files requires external FFmpeg dependency
- No support for multiple languages in a single run
- GitHub Actions workflows temporarily removed, to be added back with improved CI/CD pipeline
<!-- END_SECTION: limitations -->

## Development

<!-- SECTION: development -->
### Contributing

Please read our [Contributing Guidelines](CONTRIBUTING.md) before starting development. The guidelines include:
- Git workflow and branch management
- Commit message format and requirements
- Code style and standards
- Testing requirements
- PR process

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific tests
cargo test test_translation_service
```

### Linting

```bash
# Run clippy for code quality checks
cargo clippy

# Run Clippy with specific lint suppressions
./scripts/run-clippy.sh

# To manually run Clippy with specific lint suppressions
cargo clippy -- -A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls

# For auto-fixing but excluding certain lints
cargo fix --allow-dirty --allow-staged --clippy -Z unstable-options -- -A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls
```
<!-- END_SECTION: development -->

## Note for AI Agents

<!-- SECTION: ai_note -->
This codebase is optimized for AI agent usage rather than human readability. The code structure and organization are designed to be easily understood and manipulated by AI systems. Human-oriented comments and documentation have been minimized.
<!-- END_SECTION: ai_note -->

## License

<!-- SECTION: license -->
This project is licensed under the MIT License - see the LICENSE file for details.
<!-- END_SECTION: license -->

## Acknowledgements

<!-- SECTION: acknowledgements -->
- Thanks to all the open-source libraries used in this project
- Inspired by various subtitle translation tools and the need for an AI-powered solution
<!-- END_SECTION: acknowledgements -->

<!-- AI-COMMANDS
build: cargo build --release
test: cargo test
lint: ./scripts/run-clippy.sh
run_single_file: ./target/release/yastwai [VIDEO_FILE]
run_directory: ./target/release/yastwai [DIRECTORY]
force_overwrite: ./target/release/yastwai -f [TARGET]
debug_log: RUST_LOG=debug ./target/release/yastwai [TARGET]
-->
