# Agent Specification - Rust Overlay

## Project: YASTwAI

YASTwAI (Yet Another Subtitle Translator with AI) is an async Rust CLI that extracts subtitles from video files and translates them using multiple AI providers (Ollama, OpenAI, Anthropic) with context-aware translation and session persistence.

## Quick Reference

| Command | Description |
|---------|-------------|
| `cargo build --release` | Production build |
| `cargo build` | Development build |
| `cargo test` | Run all tests |
| `./scripts/ai-clippy.sh --check-only` | Check linting |
| `./scripts/ai-clippy.sh --fix` | Auto-fix lint issues |
| `./scripts/ai-branch.sh feature_name` | Create feature branch |
| `./scripts/ai-pr.sh` | Create pull request |
| `./scripts/ai-update-main.sh` | Update main branch |

## Architecture

The app follows a modular architecture with clear separation of concerns:

- **Providers** (`src/providers/`): AI backend implementations via trait interface
  - Ollama, OpenAI, Anthropic support
- **Translation** (`src/translation/`): Multi-pass pipeline with document context
  - Pipeline orchestration, context windows, quality validation
- **Subtitle Processing** (`src/subtitle_processor.rs`): FFmpeg integration for extraction
- **Session/Database** (`src/database/`, `src/session/`): SQLite persistence
- **Configuration** (`src/app_config.rs`): JSON-based config with provider settings

## Tech Stack

- **Language**: Rust (Edition 2024, 1.85.0+)
- **Async Runtime**: Tokio (full features)
- **HTTP Client**: reqwest
- **CLI**: clap v4
- **Database**: rusqlite (bundled SQLite)
- **Serialization**: serde + serde_json
- **Error Handling**: thiserror + anyhow

## Code Style

### Rust Conventions

```rust
// Use async/await with Tokio for all async operations
async fn translate_batch(&self, batch: &[Subtitle]) -> Result<Vec<Translation>> {
    let results = self.provider.process(batch).await?;
    Ok(results)
}

// Use thiserror for custom error types
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("Invalid subtitle format at line {line}: {message}")]
    Format { line: usize, message: String },
}

// Prefer trait-based design for extensibility
#[async_trait]
pub trait Provider: Send + Sync {
    async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse>;
}
```

### Import Organization

Group imports in this order with blank lines between:
1. `std` library
2. External crates
3. Internal modules

```rust
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::providers::Provider;
use crate::translation::TranslationService;
```

### Naming Conventions

- **Files**: `snake_case.rs` (e.g., `subtitle_processor.rs`)
- **Types/Traits**: `PascalCase` (e.g., `TranslationService`, `Provider`)
- **Functions/Variables**: `snake_case` (e.g., `process_subtitle`, `batch_size`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- **Tests**: `test_functionName_withCondition_shouldBehavior`

### File Organization

- One primary type per file
- Group by feature, not type
- Keep related types in same file if tightly coupled
- Use doc comments: `///` for items, `//!` for modules

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test subtitle_processor

# Run with output
cargo test -- --nocapture

# Run with logging enabled
YASTWAI_TEST_LOG=1 cargo test

# Run integration tests only
cargo test --test integration
```

### Test Structure

```
tests/
├── unit/              # Unit tests per module
├── integration/       # End-to-end scenarios
├── common/            # Shared test utilities and mocks
├── resources/         # Test data files
└── scripts/           # Script testing
```

### Test Conventions

```rust
#[tokio::test]
async fn test_translateBatch_withValidSubtitles_shouldReturnTranslations() {
    // Given
    let provider = MockProvider::new();
    let service = TranslationService::new(provider);
    let subtitles = vec![Subtitle::new("Hello")];

    // When
    let result = service.translate_batch(&subtitles).await;

    // Then
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}
```

## Rust-Specific Boundaries

### Always Safe (in addition to core.md)

- Run `cargo build`, `cargo test`, `cargo clippy`
- Modify files in `src/` and `tests/`
- Use helper scripts in `scripts/`
- Add new Rust files following existing patterns

### Ask First (in addition to core.md)

- Modify `Cargo.toml` (dependency changes)
- Change database schema in `src/database/schema.rs`
- Modify provider API integrations

### Never Do (in addition to core.md)

- Use `#[allow(dead_code)]` outside of tests
- Edit README.md directly (use `./scripts/ai-readme.sh`)
- Work directly on main branch (use feature branches)

## Helper Scripts

| Script | Description |
|--------|-------------|
| `ai-branch.sh` | Branch management with named parameters |
| `ai-clippy.sh` | Enhanced Clippy with fix options |
| `ai-pr.sh` | PR creation with structured descriptions |
| `ai-protect-main.sh` | Branch protection verification |
| `ai-update-main.sh` | Safe main branch updates |
| `ai-readme.sh` | Generate README.md |

### Windows Support

All scripts have Windows equivalents:
- `.ps1` PowerShell scripts: `pwsh -File scripts/ai-branch.ps1 ...`
- `.cmd` shims: `scripts\ai-branch.cmd ...`

## Common Tasks

### Creating a Feature Branch

```bash
./scripts/ai-branch.sh feature_name
```

### Creating a Pull Request

```bash
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" \
  --key-changes "Change 1,Change 2" --implementation "Detail 1,Detail 2"

# For draft PRs
./scripts/ai-pr.sh --title "PR Title" --overview "Brief" --key-changes "Change 1" --draft
```

### Running Linting

```bash
# Check only
./scripts/ai-clippy.sh --check-only

# Auto-fix
./scripts/ai-clippy.sh --fix
```

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

### Provider Support

- **Ollama**: Local AI models (default, free)
- **OpenAI**: GPT models (requires API key)
- **Anthropic**: Claude models (requires API key)

## Troubleshooting

### Build Failures

```bash
# Clean and rebuild
cargo clean && cargo build

# Check dependency issues
cargo update
```

### Branch Protection Errors

Always verify branch before work:
```bash
./scripts/ai-protect-main.sh
```

### Clippy Warnings

Address in source code, not with allow directives:
```bash
./scripts/ai-clippy.sh --fix
```

### Debugging

```bash
# Verbose logging
RUST_LOG=debug cargo run

# Test with logging
YASTWAI_TEST_LOG=1 cargo test

# Verify FFmpeg installation
ffmpeg -version
```

## Security Guidelines

- Store API keys in configuration file (never commit)
- Use separate config files for different environments
- Validate all external inputs
- Verify file types before processing
- Sanitize file paths to prevent traversal
