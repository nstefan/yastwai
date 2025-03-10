# 🎬 YASTwAI - Yet Another Subtitles Translator with AI

> *Easily translate your video subtitles using AI - right from your command line!*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ What is YASTwAI?

YASTwAI is a powerful command-line tool that extracts subtitles from your videos and translates them using AI. Whether you're watching foreign films, studying languages, or preparing content for international audiences, YASTwAI makes subtitle translation simple and effective.

## 🚀 Key Features

- 🎯 **Extract & Translate** - Pull subtitles from videos and translate them in one step
- 🌐 **Multiple AI Providers** - Support for Ollama (local), OpenAI, and Anthropic
- 🧠 **Smart Processing** - Preserves formatting and timing of your subtitles
- ⚡ **Concurrent Processing** - Efficient batch translation for faster results
- 🔄 **Direct Translation** - Translate existing SRT files without needing video
- 📊 **Progress Tracking** - See real-time progress for lengthy translations
- 🎛️ **Configurable** - Customize translation settings to your needs

## 📋 Prerequisites

- 🦀 Rust and Cargo (1.85.0 or newer)
- 🎞️ FFmpeg installed on your system (for subtitle extraction)

## 🔧 Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/yastwai.git
cd yastwai

# Build the application
cargo build --release

# The executable will be in target/release/yastwai
```

## 🏃‍♂️ Quick Start

1. **Copy the example config** (or let YASTwAI create one for you):

```bash
cp conf.example.json conf.json
```

2. **Run YASTwAI**:

```bash
# Translate subtitles from a video file
./target/release/yastwai video.mkv

# Process an entire directory
./target/release/yastwai videos/

# Translate an SRT file directly
./target/release/yastwai subtitles.srt

# Force overwrite existing translations
./target/release/yastwai -f video.mkv
```

3. **Find your translations** next to the original files as `original_name.{target_language}.srt`

## ⚙️ Configuration

YASTwAI uses a simple JSON configuration file with these main settings:

```json
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama",
    // Provider-specific settings...
  }
}
```

### 🔍 Main Options

| Setting | Description | Default |
|---------|-------------|---------|
| `source_language` | Source language code | `"en"` |
| `target_language` | Target language code | `"fr"` |
| `translation.provider` | Provider: `"ollama"`, `"openai"`, or `"anthropic"` | `"ollama"` |
| `log_level` | Logging level (`"error"`, `"info"`, `"debug"`, etc.) | `"info"` |
| `batch_size` | Characters to process per batch | `1000` |

See the full configuration file for provider-specific options.

## 🤖 Translation Providers

### Ollama (Default, Local)
- 🏠 Free, runs locally on your machine
- 🔗 Install from [ollama.ai](https://ollama.ai/)
- 🧩 Pull your model: `ollama pull mixtral:8x7b`

### OpenAI
- 🔑 Requires API key from [platform.openai.com](https://platform.openai.com/)
- 📝 Add key to config file
- 🧠 Models: gpt-4o-mini, gpt-4o, etc.

### Anthropic
- 🔑 Requires API key from [anthropic.com](https://www.anthropic.com/)
- 📝 Add key to config file
- 🧠 Models: claude-3-haiku, claude-3-sonnet, etc.

## 🛠️ Development

### Running Tests

```bash
# Run all tests
cargo test

# With logging
RUST_LOG=debug cargo test
```

### Linting

```bash
# Run clippy checks
cargo clippy

# Run with script (includes specific lint settings)
./scripts/run-clippy.sh
```

### Helper Scripts

```bash
# Create formatted commits
./scripts/create-commit.sh "Commit title" "Prompt" "Description" "Discussion"

# Create PRs
./scripts/ai-pr-helper.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2"
```

## 🔮 Future Improvements

- Support for more subtitle formats beyond SRT
- Multiple language translation in a single run
- Improved CI/CD pipeline (coming soon!)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 👏 Acknowledgements

- Thanks to all the open-source libraries used in this project
- Inspired by various subtitle translation tools and the need for an AI-powered solution
