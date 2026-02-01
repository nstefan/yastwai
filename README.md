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

Copy `conf.example.json` to `conf.json` and edit it to configure:

- **Languages** - Set `source_language` and `target_language` (ISO codes)
- **Provider** - Choose between `ollama`, `openai`, `anthropic`, or `lmstudio`
- **Model** - Set the model for your provider
- **API key** - Required for OpenAI and Anthropic

See `conf.example.json` for all available options.

### Supported Providers

| Provider | Description |
|----------|-------------|
| **Ollama** | Local LLM server (default, free) |
| **OpenAI** | GPT models via API |
| **Anthropic** | Claude models via API |
| **LM Studio** | Local OpenAI-compatible server (also works with vLLM) |

## Contributing

Contributions welcome! Please open an issue or pull request.

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
