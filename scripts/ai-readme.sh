#!/bin/bash
set -e

# Get the script directory path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Use colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
QUIET=false
DRY_RUN=false

function print_usage {
    echo "Usage: $0 [OPTIONS]"
    echo "Generate a README.md file for the project based on codebase and docs"
    echo ""
    echo "Options:"
    echo "  --quiet         Only show errors"
    echo "  --dry-run       Don't write the README.md file, just output to stdout"
    echo "  --help          Display this help message"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --quiet)
            QUIET=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            print_usage
            exit 1
            ;;
    esac
done

# Function to log messages based on quiet mode
function log {
    if [ "$QUIET" = false ]; then
        echo -e "$1"
    fi
}

log "${BLUE}Starting README generator...${NC}"

# Get project information from Cargo.toml
PROJECT_NAME=$(grep -m 1 "name" "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2 || echo "yastwai")
PROJECT_VERSION=$(grep -m 1 "version" "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2 || echo "0.1.0")
RUST_MIN_VERSION=$(grep -m 1 "rust-version" "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2 || echo "1.85.0")
LICENSE=$(grep -m 1 "license" "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f 2 || echo "MIT")

# Extract dependencies from Cargo.toml
DEPENDENCIES=$(grep -A 100 "\[dependencies\]" "$PROJECT_ROOT/Cargo.toml" | grep -B 100 "\[dev-dependencies\]" | grep -v "\[dependencies\]" | grep -v "\[dev-dependencies\]" | grep -v "^#" | grep -v "^$" || echo "")

# Check if docs directory exists and get content
DOCS_DIR="$PROJECT_ROOT/docs"
if [ -d "$DOCS_DIR" ]; then
    log "${GREEN}Found docs directory${NC}"
else
    log "${YELLOW}No docs directory found${NC}"
fi

# Detect AI providers available in the codebase
PROVIDERS_DIR="$PROJECT_ROOT/src/providers"
PROVIDERS=""

if [ -d "$PROVIDERS_DIR" ]; then
    log "${GREEN}Found providers directory${NC}"
    for provider in "$PROVIDERS_DIR"/*.rs; do
        if [ -f "$provider" ]; then
            provider_name=$(basename "$provider" .rs)
            if [ "$PROVIDERS" == "" ]; then
                PROVIDERS="$provider_name"
            else
                PROVIDERS="$PROVIDERS, $provider_name"
            fi
        fi
    done
    log "${BLUE}Detected providers: $PROVIDERS${NC}"
else
    log "${YELLOW}No providers directory found${NC}"
fi

# Check for example config
EXAMPLE_CONFIG="$PROJECT_ROOT/conf.example.json"
if [ -f "$EXAMPLE_CONFIG" ]; then
    log "${GREEN}Found example config${NC}"
else
    log "${YELLOW}No example config found${NC}"
fi

# Generate features list
FEATURES=""
# Check for subtitle extraction capability
if grep -q "ffmpeg" "$PROJECT_ROOT/src" -r 2>/dev/null; then
    FEATURES="$FEATURES\n- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step"
fi
# Check for multiple providers
if [ "$PROVIDERS" != "" ]; then
    FEATURES="$FEATURES\n- 🌐 **Multiple AI Providers** - Support for ${PROVIDERS}"
fi
# Check for concurrent processing
if grep -q "tokio::spawn" "$PROJECT_ROOT/src" -r 2>/dev/null || grep -q "async" "$PROJECT_ROOT/src" -r 2>/dev/null; then
    FEATURES="$FEATURES\n- ⚡ **Concurrent Processing** - Efficient batch translation for faster results"
fi
# Add default features if none detected
if [ "$FEATURES" == "" ]; then
    FEATURES="- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step
- 🌐 **Multiple AI Providers** - Support for Ollama (local), OpenAI, and Anthropic
- 🧠 **Smart Processing** - Preserves formatting and timing of your subtitles
- ⚡ **Concurrent Processing** - Efficient batch translation for faster results
- 🔄 **Direct Translation** - Translate existing SRT files without needing video
- 📊 **Progress Tracking** - See real-time progress for lengthy translations"
fi

# Generate README content
README_CONTENT=$(cat <<EOL
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
${FEATURES}

## Installation

### Prerequisites

* Rust and Cargo ($RUST_MIN_VERSION or newer)
  \`\`\`sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  \`\`\`
* FFmpeg installed on your system (for subtitle extraction)
  \`\`\`sh
  # On Ubuntu/Debian
  sudo apt install ffmpeg
  
  # On macOS with Homebrew
  brew install ffmpeg
  
  # On Windows with Chocolatey
  choco install ffmpeg
  \`\`\`

### Build from Source

\`\`\`sh
# Clone the repository
git clone https://github.com/nstefan/yastwai.git
cd yastwai

# Build the application
cargo build --release

# The executable will be in target/release/yastwai
\`\`\`

## Quick Start

1. **Copy the example config** (or let YASTwAI create one for you):
   \`\`\`sh
   cp conf.example.json conf.json
   \`\`\`

2. **Run YASTwAI**:
   \`\`\`sh
   # Translate subtitles from a video file
   ./target/release/yastwai video.mkv

   # Process an entire directory
   ./target/release/yastwai videos/

   # Translate an SRT file directly
   ./target/release/yastwai subtitles.srt

   # Force overwrite existing translations
   ./target/release/yastwai -f video.mkv
   \`\`\`

3. **Find your translations** next to the original files as \`original_name.{target_language}.srt\`

## Configuration

YASTwAI uses a simple JSON configuration file with these main settings:

\`\`\`json
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama"
  }
}
\`\`\`

### Translation Providers

#### Ollama (Default, Local)
- 🏠 Free, runs locally on your machine
- 🔗 Install from [ollama.ai](https://ollama.ai/)
- 🧩 Pull your model: \`ollama pull mixtral:8x7b\`

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

Distributed under the $LICENSE License. See \`LICENSE\` for more information.

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
EOL
)

# Output the README content
if [ "$DRY_RUN" = true ]; then
    echo -e "$README_CONTENT"
    log "${GREEN}README content generated successfully (dry run)${NC}"
else
    echo -e "$README_CONTENT" > "$PROJECT_ROOT/README.md"
    log "${GREEN}README.md file generated successfully at $PROJECT_ROOT/README.md${NC}"
fi

exit 0 