[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<div align="center">
  <h1>YASTwAI</h1>
  <p><strong>Y</strong>et <strong>A</strong>nother <strong>S</strong>ub<strong>t</strong>itle translator <strong>w</strong>ith <strong>AI</strong></p>
  <p>
    <a href="#about">About</a> •
    <a href="#key-features">Features</a> •
    <a href="#installation">Installation</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#configuration">Configuration</a> •
    <a href="#contributing">Contributing</a> •
    <a href="#license">License</a>
  </p>
</div>

## About

YASTwAI is a command-line tool that extracts subtitles from videos and translates them using AI. Built with Rust for performance, it preserves formatting and timing, and supports multiple translation providers.

## Key Features
- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step
- 🌐 **Multiple AI Providers** - Support for Ollama, OpenAI, Anthropic, LM Studio (including vLLM and OpenAI-compatible servers)
- ⚡ **Parallel Processing** - Fast concurrent batch translation with configurable parallelism
- 🧠 **Context-Aware Translation** - Includes previous entries as context for consistency (tu/vous, genders)
- 💾 **Session Persistence** - Resume interrupted translations automatically
- 🔄 **Direct Translation** - Translate existing SRT files without needing video
- 📊 **Progress Tracking** - See real-time progress for lengthy translations

## Installation

### Prerequisites

* Rust and Cargo (1.85.0 or newer)
* FFmpeg installed on your system
* GitHub CLI (gh) for pull request operations

### Build from Source

```sh
git clone https://github.com/nstefan/yastwai.git
cd yastwai
cargo build --release
```

## Quick Start

```sh
cp conf.example.json conf.json
./target/release/yastwai video.mkv
./target/release/yastwai videos/
./target/release/yastwai subtitles.srt
./target/release/yastwai -f video.mkv
```

## Configuration

```json
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama",
    "available_providers": [
      {
        "type": "ollama",
        "model": "mixtral:8x7b",
        "endpoint": "http://localhost:11434",
        "concurrent_requests": 2,
        "max_chars_per_request": 1000,
        "timeout_secs": 60
      },
      {
        "type": "openai",
        "model": "gpt-4o-mini",
        "api_key": "your_api_key",
        "endpoint": "https://api.openai.com/v1",
        "concurrent_requests": 10,
        "max_chars_per_request": 2000,
        "timeout_secs": 60
      },
      {
        "type": "lmstudio",
        "model": "local-model",
        "api_key": "lm-studio",
        "endpoint": "http://localhost:1234/v1",
        "concurrent_requests": 4,
        "max_chars_per_request": 1000,
        "timeout_secs": 60
      },
      {
        "type": "anthropic",
        "model": "claude-3-haiku-20240307",
        "api_key": "your_api_key",
        "endpoint": "https://api.anthropic.com",
        "concurrent_requests": 2,
        "max_chars_per_request": 400,
        "timeout_secs": 60,
        "rate_limit": 45
      }
    ],
    "common": {
      "system_prompt": "You are an expert subtitle translator specializing in {source_language} to {target_language} translation. Translate precisely while preserving formatting, line breaks, and special tags. Maintain consistency in formal/informal address and character genders throughout.",
      "rate_limit_delay_ms": 500,
      "retry_count": 3,
      "retry_backoff_ms": 1000,
      "temperature": 0.3,
      "parallel_mode": true,
      "entries_per_request": 3,
      "context_entries_count": 3
    }
  },
  "session": {
    "enabled": true,
    "auto_resume": true
  },
  "log_level": "info"
}
```

### Translation Providers

- **Ollama** - Local LLM server (default, free)
- **OpenAI** - GPT models via API
- **Anthropic** - Claude models via API
- **LM Studio** - Local OpenAI-compatible server
- **vLLM** - High-performance inference server (use lmstudio provider type)

## Contributing

Helper scripts in `scripts/`: `ai-branch`, `ai-update-main`, `ai-pr`, `ai-clippy`

## License

Distributed under the MIT License. See `LICENSE` for details.

[contributors-shield]: https://img.shields.io/github/contributors/nstefan/yastwai.svg?style=for-the-badge
[contributors-url]: https://github.com/nstefan/yastwai/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/nstefan/yastwai.svg?style=for-the-badge
[forks-url]: https://github.com/nstefan/yastwai/network/members
[stars-shield]: https://img.shields.io/github/stars/nstefan/yastwai.svg?style=for-the-badge
[stars-url]: https://github.com/nstefan/yastwai/stargazers
[issues-shield]: https://img.shields.io/github/issues/nstefan/yastwai.svg?style=for-the-badge
[issues-url]: https://github.com/nstefan/yastwai/issues
[license-shield]: https://img.shields.io/github/license/nstefan/yastwai.svg?style=for-the-badge
[license-url]: https://github.com/nstefan/yastwai/blob/master/LICENSE
