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

YASTwAI is a command-line tool that extracts subtitles from videos and translates them using AI. Whether you're watching foreign films, studying languages, or preparing content for international audiences, YASTwAI makes subtitle translation simple and effective.

Built with Rust for performance and reliability, YASTwAI supports multiple AI translation providers and preserves the original subtitle formatting and timing.

## Key Features

- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step
- 🌐 **Multiple AI Providers** - Support for Ollama, OpenAI, Anthropic
- ⚡ **Concurrent Processing** - Efficient batch translation for faster results
- 🧠 **Smart Processing** - Preserves formatting and timing of your subtitles
- 🔄 **Direct Translation** - Translate existing SRT files without needing video
- 📊 **Progress Tracking** - See real-time progress for lengthy translations

## Installation

### Prerequisites

* Rust and Cargo (1.85.0 or newer)
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
* FFmpeg installed on your system (for subtitle extraction)
  ```sh
  # On Ubuntu/Debian
  sudo apt install ffmpeg
  
  # On macOS with Homebrew
  brew install ffmpeg
  
  # On Windows with Chocolatey
  choco install ffmpeg
  ```

### Build from Source

```sh
# Clone the repository
git clone https://github.com/nstefan/yastwai.git
cd yastwai

# Build the application
cargo build --release

# The executable will be in target/release/yastwai
```

## Quick Start

1. **Copy the example config** (or let YASTwAI create one for you):
   ```sh
   cp conf.example.json conf.json
   ```

2. **Run YASTwAI**:
   ```sh
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

## Configuration

YASTwAI uses a simple JSON configuration file with these main settings:

```json
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama"
  }
}
```

### Translation Providers

#### Ollama (Default, Local)
- 🏠 Free, runs locally on your machine
- 🔗 Install from [ollama.ai](https://ollama.ai/)
- 🧩 Pull your model: `ollama pull mixtral:8x7b`

#### OpenAI
- 🔑 Requires API key from [platform.openai.com](https://platform.openai.com/)
- 🧠 Models: gpt-4o-mini, gpt-4o, etc.

#### Anthropic
- 🔑 Requires API key from [anthropic.com](https://www.anthropic.com/)
- 🧠 Models: claude-3-haiku, claude-3-sonnet, etc.

See the example configuration file for more detailed options.

## Roadmap

- [ ] Support for more subtitle formats beyond SRT
- [ ] UI improvements for better progress visualization
- [ ] Additional AI providers
- [ ] Performance optimizations for large batch processing

See the [open issues](https://github.com/nstefan/yastwai/issues) for a full list of proposed features and known issues.

## Contributing

Contributions are welcome! If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also open an issue with the tag "enhancement".

Don't forget to give the project a star! Thanks!

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Acknowledgments

* [Cursor Editor](https://cursor.sh/) for making AI-powered development possible
* [FFmpeg](https://ffmpeg.org/) for the powerful media processing capabilities

<!-- MARKDOWN LINKS & IMAGES -->
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
