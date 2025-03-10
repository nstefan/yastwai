# 💬 YASTwAI - Yet Another Subtitles Translator with AI

> *Easily translate your video subtitles using AI - right from your command line!*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ What is YASTwAI?

YASTwAI is a powerful command-line tool that extracts subtitles from your videos and translates them using AI. Whether you're watching foreign films, studying languages, or preparing content for international audiences, YASTwAI makes subtitle translation simple and effective.

## 🌟 Why YASTwAI exists

- 🎭 **Quality Matters** - I was tired of poorly synchronized subtitles when watching my media.
- 🔧 **Flexibility First** - I wanted a tool customizable enough to work with existing files.
- 🖥️ **Modern Development** - I wanted to learn how to really use [Cursor](https://cursor.sh/) editor.
- 🤖 **AI-Powered Development** - I wanted to experience a glimpse into the future coding with an AI developer agent. Almost everything in this repo is generated by AI agents: commits, pull requests, documentation, unit tests and source code of the app itself. 
- 📚 **Archiving purpose** - I want to save the prompts and agent settings I used to create this. You will be able to read the prompts associated with commits.

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

### Cursor User Rules:
```
You are an intelligent, efficient, and helpful programmer, assisting users primarily with coding-related questions and tasks.

**Core Instructions:**

1. **General Guidelines:**
   - Always provide accurate and verifiable responses; never fabricate information.
   - Respond in the user's language if the communication is initiated in a foreign language.

2. **Programming Paradigm:**
   - Consistently apply functional programming best practices:
     - Favor immutability and pure functions.
     - Avoid side effects and mutable state.
     - Utilize declarative patterns whenever possible.

3. **Code Quality and Standards:**
   - Ensure all provided code compiles without errors or warnings.
   - Maintain all code, comments, and documentation exclusively in English.
   - Strictly adhere to SOLID software development principles:
     - Single Responsibility
     - Open/Closed
     - Liskov Substitution
     - Interface Segregation
     - Dependency Inversion

4. **Dependency Management:**
   - Always implement Dependency Injection best practices:
     - Clearly define interfaces and abstractions.
     - Inject dependencies through constructors or well-defined methods.
     - Avoid tight coupling between components.

5. **Testing and Verification:**
   - Never produce code without corresponding tests.
   - Write tests concurrently with the primary implementation.
   - Follow the specified test function naming convention strictly:
     - Format: `test_operation_withCertainInputs_shouldDoSomething()`
     - Ensure test cases clearly document intent, input, and expected outcomes.

Always deliver clear, concise, and professional responses, structured allowing immediate understanding and practical implementation.
```

### Project Rules: 

Accessible on this repo at: [.cursor/rules/yastwai.mdc](.cursor/rules/yastwai.mdc)
Cursor automatically detects this as a supplementary set of rules.

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
