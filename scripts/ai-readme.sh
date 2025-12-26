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

# Extract provider enum values from app_config.rs
PROVIDER_CONFIG="$PROJECT_ROOT/src/app_config.rs"
PROVIDERS_PRETTY=""
PROVIDERS_FOUND=false

if [ -f "$PROVIDER_CONFIG" ]; then
    log "${GREEN}Found provider configuration${NC}"
    
    # First, try to extract the enum variants
    ENUM_START=$(grep -n "pub enum TranslationProvider" "$PROVIDER_CONFIG" | cut -d ':' -f1)
    
    if [ -n "$ENUM_START" ]; then
        # Find the line where the enum ends by finding the first closing brace after the enum start
        ENUM_END=$(tail -n +$ENUM_START "$PROVIDER_CONFIG" | grep -n "^}" | head -1 | cut -d ':' -f1)
        ENUM_END=$((ENUM_START + ENUM_END - 1))
        
        # Extract the enum variants
        VARIANTS=$(sed -n "${ENUM_START},${ENUM_END}p" "$PROVIDER_CONFIG" | grep -o "#\[default\]\s*\|\s*[A-Za-z]\+," | sed 's/#\[default\]//g' | sed 's/,//g' | sed 's/^[[:space:]]*//g' | grep -v "^$")
        
        if [ -n "$VARIANTS" ]; then
            log "${BLUE}Found TranslationProvider enum variants${NC}"
            
            # Now extract display names from display_name method
            DISPLAY_NAME_START=$(grep -n "pub fn display_name" "$PROVIDER_CONFIG" | cut -d ':' -f1)
            
            if [ -n "$DISPLAY_NAME_START" ]; then
                # Find the end of the display_name method
                DISPLAY_NAME_END=$(tail -n +$DISPLAY_NAME_START "$PROVIDER_CONFIG" | grep -n "^[[:space:]]*}$" | head -1 | cut -d ':' -f1)
                DISPLAY_NAME_END=$((DISPLAY_NAME_START + DISPLAY_NAME_END - 1))
                
                # Extract display names
                DISPLAY_NAMES=""
                
                # Process each variant
                while read -r variant; do
                    # Find the corresponding display name
                    DISPLAY_NAME=$(sed -n "${DISPLAY_NAME_START},${DISPLAY_NAME_END}p" "$PROVIDER_CONFIG" | grep -o "Self::${variant}[[:space:]]*=>[[:space:]]*\"[^\"]*\"" | grep -o "\"[^\"]*\"" | tr -d '"')
                    
                    if [ -n "$DISPLAY_NAME" ]; then
                        if [ -z "$DISPLAY_NAMES" ]; then
                            DISPLAY_NAMES="$DISPLAY_NAME"
                        else
                            DISPLAY_NAMES="$DISPLAY_NAMES, $DISPLAY_NAME"
                        fi
                    fi
                done <<< "$VARIANTS"
                
                if [ -n "$DISPLAY_NAMES" ]; then
                    PROVIDERS_PRETTY="$DISPLAY_NAMES"
                    PROVIDERS_FOUND=true
                    log "${BLUE}Provider display names: $PROVIDERS_PRETTY${NC}"
                else
                    log "${YELLOW}Could not extract provider display names from enum${NC}"
                fi
            else
                log "${YELLOW}Could not find display_name method${NC}"
            fi
        else
            log "${YELLOW}Could not extract enum variants${NC}"
        fi
    else
        log "${YELLOW}Could not find TranslationProvider enum${NC}"
    fi
else
    log "${YELLOW}Could not find provider configuration${NC}"
fi

# Check for example config
EXAMPLE_CONFIG="$PROJECT_ROOT/conf.example.json"
if [ -f "$EXAMPLE_CONFIG" ]; then
    log "${GREEN}Found example config${NC}"
    # Read the example config content for dynamic inclusion in README
    EXAMPLE_CONFIG_CONTENT=$(cat "$EXAMPLE_CONFIG")
else
    log "${YELLOW}No example config found${NC}"
    # Fallback to a minimal example if conf.example.json doesn't exist
    EXAMPLE_CONFIG_CONTENT='{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama"
  }
}'
fi

# Create a preprocessed version of the JSON with proper escaping for markdown
# This ensures special characters and newlines render correctly
JSON_TEMP_FILE=$(mktemp)
echo "$EXAMPLE_CONFIG_CONTENT" > "$JSON_TEMP_FILE"

# Process the JSON to ensure proper escaping
processed_json=""
while IFS= read -r line; do
    # Escape backslashes first (double them)
    line="${line//\\/\\\\}"
    # Escape opening braces that are part of format tags like {\an8}
    line="${line//{\\/\\{\\}"
    # Add the processed line to our result
    processed_json="${processed_json}${line}\\n"
done < "$JSON_TEMP_FILE"

# Remove trailing newline
processed_json="${processed_json%\\n}"

# Replace the original content with the processed version
EXAMPLE_CONFIG_CONTENT="$processed_json"

# Clean up
rm -f "$JSON_TEMP_FILE"

# Generate features list
FEATURES=""
# Check for subtitle extraction capability
if grep -q "ffmpeg" "$PROJECT_ROOT/src" -r 2>/dev/null; then
    FEATURES="$FEATURES\n- 🎯 **Extract & Translate** - Pull subtitles from videos and translate in one step"
fi
# Add providers feature
if [ "$PROVIDERS_FOUND" = true ]; then
    FEATURES="$FEATURES\n- 🌐 **Multiple AI Providers** - Support for ${PROVIDERS_PRETTY}"
else
    FEATURES="$FEATURES\n- 🌐 **Multiple AI Providers** - Support for various AI translation backends"
fi
# Check for concurrent processing
if grep -q "tokio::spawn" "$PROJECT_ROOT/src" -r 2>/dev/null || grep -q "async" "$PROJECT_ROOT/src" -r 2>/dev/null; then
    FEATURES="$FEATURES\n- ⚡ **Concurrent Processing** - Efficient batch translation for faster results"
fi
# Add more features
FEATURES="$FEATURES\n- 🧠 **Smart Processing** - Preserves formatting and timing of your subtitles"
FEATURES="$FEATURES\n- 🔄 **Direct Translation** - Translate existing SRT files without needing video"
FEATURES="$FEATURES\n- 📊 **Progress Tracking** - See real-time progress for lengthy translations"

# Generate features list

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
* GitHub CLI (gh) for pull request operations
  \`\`\`sh
  # On Ubuntu/Debian
  sudo apt install gh
  
  # On macOS with Homebrew
  brew install gh
  
  # On Windows with Chocolatey
  choco install gh
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

YASTwAI uses a JSON configuration file with these settings:

\`\`\`json
${EXAMPLE_CONFIG_CONTENT}
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

## Contributing

Contributions are welcome! If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also open an issue with the tag "enhancement".

Several helper scripts are available in the \`scripts/\` directory to assist contributors:
- \`ai-branch.sh\` - Create and manage feature branches
- \`ai-update-main.sh\` - Safely update the main branch without interactive prompts
- \`ai-pr.sh\` - Generate formatted pull requests
- \`ai-clippy.sh\` - Run Rust code linting

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

<!-- NOTE: This README is automatically generated. Do not edit directly. -->
<!-- If you need to make changes, modify the generation script at scripts/ai-readme.sh instead. -->
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