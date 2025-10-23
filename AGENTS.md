# AGENTS.md

## Project Overview

YASTwAI (Yet Another Subtitle Translator with AI) is an async Rust application that extracts subtitles from video files and translates them using multiple AI providers. The project emphasizes performance, modularity, and concurrent processing with support for Ollama, OpenAI, and Anthropic providers.

## Quick Start Commands

```bash
# Build the application
cargo build --release

# Run the application
./target/release/yastwai video.mkv

# Run with configuration
./target/release/yastwai -c conf.json video.mkv

# Process entire directory 
./target/release/yastwai videos/

# Run tests
cargo test

# Run clippy linting
./scripts/ai-clippy.sh --check-only
```

## Development Environment Setup

### Prerequisites
- Rust 1.85.0 or newer (Edition 2024)
- FFmpeg (for subtitle extraction)
- GitHub CLI (`gh`) for pull request operations

### Initial Setup
```bash
# Clone and build
git clone https://github.com/nstefan/yastwai.git
cd yastwai
cargo build --release

# Copy example configuration
cp conf.example.json conf.json
```

## Critical Development Rules

### 🚨 BRANCH PROTECTION - HIGHEST PRIORITY
- **NEVER** work directly on the main branch under ANY circumstances
- **ALWAYS** run branch verification as the FIRST action in every interaction:
  ```bash
  ./scripts/ai-protect-main.sh --no-auto-branch
  ```
  Windows:
  ```powershell
  pwsh -File scripts/ai-protect-main.ps1 --no-auto-branch
  # or
  scripts\ai-protect-main.cmd --no-auto-branch
  ```
- If on main branch, **IMMEDIATELY** create a feature branch:
  ```bash
  ./scripts/ai-protect-main.sh --auto-branch "descriptive-feature-name"
  ```
  Windows:
  ```powershell
  pwsh -File scripts/ai-protect-main.ps1 --auto-branch "descriptive-feature-name"
  # or
  scripts\ai-protect-main.cmd --auto-branch "descriptive-feature-name"
  ```

### Branch Management
```bash
# Check current branch status
./scripts/ai-branch.sh --check-only

# Create new feature branch from main
./scripts/ai-branch.sh --new-branch "feature-name" --is-related false

# Update main branch
./scripts/ai-update-main.sh
```
Windows equivalents:
```powershell
# PowerShell
pwsh -File scripts/ai-branch.ps1 --check-only
pwsh -File scripts/ai-branch.ps1 --new-branch "feature-name" --is-related false
pwsh -File scripts/ai-update-main.ps1

# Or use .cmd shims
scripts\ai-branch.cmd --check-only
scripts\ai-branch.cmd --new-branch "feature-name" --is-related false
scripts\ai-update-main.cmd
```

### Commit Process
```bash
# Stage changes
git add -A

# Create commit using helper script (required)
./scripts/ai-commit.sh --model "model-name" "Title" "Description" "Prompt" "Thought Process" "Discussion"
```
Windows equivalents:
```powershell
# PowerShell
pwsh -File scripts/ai-commit.ps1 -Model "model-name" "Title" "Description" "Prompt" "Thought Process" "Discussion"

# Or use .cmd shim
scripts\ai-commit.cmd -Model "model-name" "Title" "Description" "Prompt" "Thought Process" "Discussion"
```

## Code Style & Standards

### Language Requirements
- All code, comments, and documentation **MUST** be in English
- Use functional programming patterns where possible
- Maintain immutability and avoid side effects
- Follow SOLID principles strictly

### Rust-Specific Guidelines
- Use async/await patterns with Tokio runtime
- Implement proper error handling with `Result<T, E>` and `?` operator
- Use `thiserror` for custom errors, `anyhow` for general error handling
- Prefer trait-based design for extensibility
- Write tests concurrently with implementation
- Test function naming: `test_operation_withCertainInputs_shouldDoSomething()`

### Linting
```bash
# Run Clippy checks
./scripts/ai-clippy.sh --check-only

# Auto-fix issues
./scripts/ai-clippy.sh --fix
```
Windows equivalents:
```powershell
pwsh -File scripts/ai-clippy.ps1 --check-only
pwsh -File scripts/ai-clippy.ps1 --fix
# Or use .cmd shims: scripts\ai-clippy.cmd --check-only / --fix
```

## Project Structure

```
src/
├── main.rs              # CLI entry point and argument handling
├── app_config.rs        # Configuration management
├── app_controller.rs    # Main workflow orchestration
├── subtitle_processor.rs # SRT parsing and subtitle extraction
├── file_utils.rs        # File system operations
├── language_utils.rs    # Language code validation
├── providers/           # AI provider implementations
│   ├── mod.rs          # Provider trait and common types
│   ├── ollama.rs       # Ollama provider
│   ├── openai.rs       # OpenAI provider
│   └── anthropic.rs    # Anthropic provider
└── translation/        # Translation service and batching
    ├── mod.rs          # Translation service orchestration
    ├── core.rs         # Core translation logic
    ├── batch.rs        # Batch processing
    ├── cache.rs        # Translation caching
    └── formatting.rs   # Output formatting
```

## Testing Guidelines

### Test Organization
```
tests/
├── unit/              # Unit tests per module
├── integration/       # End-to-end scenarios
├── common/           # Shared test utilities and mocks
├── resources/        # Test data files
└── scripts/          # Script testing
```

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test subtitle_processor

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Test Requirements
- **NEVER** produce code without corresponding tests
- Write tests concurrently with primary implementation
- Use mock providers for isolated testing
- Include edge case and error scenario coverage
- All new scripts require both shell script tests and Rust test integration

## Key Dependencies & Features

### Core Dependencies
- `tokio` - Async runtime with full features
- `reqwest` - HTTP client for AI provider APIs
- `clap` - CLI argument parsing
- `serde` + `serde_json` - Configuration and data serialization
- `anyhow` + `thiserror` - Error handling
- `regex` - Text processing for subtitles

### Provider Support
- **Ollama**: Local AI models (default, free)
- **OpenAI**: GPT models (requires API key)
- **Anthropic**: Claude models (requires API key)

## Configuration

### Main Config File (`conf.json`)
```json
{
  "source_language": "en",
  "target_language": "fr", 
  "translation": {
    "provider": "ollama",
    "available_providers": [...]
  },
  "log_level": "info",
  "batch_size": 1000
}
```

### Environment Variables
- Override config via CLI: `-c custom_config.json`
- Provider-specific settings in config file
- API keys should be set in configuration (not environment vars)

## Automated Workflow

### Build & Test Automation
After modifying source files, always:
1. Build the application: `cargo build --release`
2. Run unit tests: `cargo test`
3. Run clippy: `./scripts/ai-clippy.sh --check-only`

### Git Command Safety
- All git commands **MUST** use non-interactive flags
- Pipe output through `cat` to prevent pager activation:
  ```bash
  git log | cat
  git diff | cat
  git status | cat
  ```

## Helper Scripts

### AI-Optimized Scripts (Use These)
- `ai-branch.sh` - Branch management with named parameters
- `ai-commit.sh` - Non-interactive commit workflow
- `ai-clippy.sh` - Enhanced Clippy with fix options
- `ai-pr.sh` - PR creation with structured descriptions
- `ai-protect-main.sh` - Branch protection verification
- `ai-update-main.sh` - Safe main branch updates
- `ai-readme.sh` - Generate README.md (don't edit README directly)

Windows usage notes:
- All helper scripts have Windows equivalents: `.ps1` PowerShell scripts and `.cmd` shims in `scripts/`.
- Prefer PowerShell form when possible (more explicit), e.g. `pwsh -File scripts/ai-pr.ps1 ...`.
- `.cmd` shims provide convenient invocation from Windows shells: e.g. `scripts\ai-pr.cmd ...`.
- Model detection on Windows: `pwsh -File scripts/ai-cursor-model.ps1` (or `scripts\ai-cursor-model.cmd`).

### Script Testing
- All scripts have corresponding tests in `tests/scripts/`
- Tests include mock functions for Git operations
- Integration tests in `tests/script_tests.rs`

## Pull Request Guidelines

### Creating PRs
```bash
# Use the helper script
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2" --implementation "Detail 1,Detail 2"

# For draft PRs
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2" --draft
```

### PR Structure
- 🧠 **Instructions** (for AI-generated PRs)
- 📌 **Overview**: Brief summary (2-3 sentences)
- 🔍 **Key Changes**: Bullet points of significant changes
- 🧩 **Implementation Details**: Technical approach
- 🔄 **Migration Notes**: Breaking changes
- ⚠️ **Areas of Attention**: Review focus areas

## File Management Rules

### Special Files
- **README.md**: Auto-generated, use `./scripts/ai-readme.sh` to update
- **.mdc files**: Symlinks to source files, never edit directly
- **Cargo.toml**: Changes trigger README regeneration

### Prohibited Actions
- Direct editing of README.md
- Direct editing of .mdc symlink files
- Using `#[allow(dead_code)]` except in tests
- Working directly on main branch
- Using raw git commands without safety flags

## Performance Considerations

### Async Patterns
- Use `tokio::spawn` for independent tasks
- Implement proper timeout handling
- Use bounded channels for backpressure
- Prefer structured concurrency

### Memory Management
- Stream large files where possible
- Clean up temporary files
- Use bounded buffer sizes
- Monitor resource usage in tests

## Troubleshooting

### Common Issues
- **Branch protection errors**: Always verify branch before work
- **Test failures**: Check if code changed before modifying tests  
- **Clippy warnings**: Address in source code, not with allow directives
- **Build failures**: Ensure all dependencies are up to date

### Debugging
- Use `RUST_LOG=debug` for verbose logging
- Check configuration file syntax with JSON validator
- Verify FFmpeg installation for subtitle extraction
- Test AI provider connectivity separately

## Security Guidelines

### API Key Management
- Store API keys in configuration file
- Never commit API keys to repository
- Use separate config files for different environments
- Validate all external inputs

### Input Validation
- Verify file types before processing
- Sanitize file paths to prevent traversal
- Validate subtitle content format
- Handle malformed input gracefully

---

*This file follows the [AGENTS.md format](https://agents.md/) for AI coding agents. For human contributors, see [CONTRIBUTING.md](./CONTRIBUTING.md).*
