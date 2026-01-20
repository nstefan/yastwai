# YASTwAI - AI Agent Specification

YASTwAI (Yet Another Subtitle Translator with AI) is an async Rust application that extracts subtitles from video files and translates them using multiple AI providers (Ollama, OpenAI, Anthropic).

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

- **Providers**: AI backend implementations (Ollama, OpenAI, Anthropic) via trait interface
- **Translation**: Multi-pass pipeline with document context, batching, and quality validation
- **Subtitle Processing**: FFmpeg integration for extraction, SRT parsing and generation
- **Session/Database**: SQLite persistence for sessions and caching
- **Configuration**: JSON-based config with provider-specific settings

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

### Naming Conventions

- **Variables/Functions**: `snake_case` (e.g., `process_subtitle`, `batch_size`)
- **Types/Traits**: `PascalCase` (e.g., `TranslationService`, `Provider`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
- **Tests**: `test_functionName_withCondition_shouldBehavior`

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

### File Organization

- One primary type per file
- Group by feature, not type
- Keep related types in same file if tightly coupled
- Use doc comments: `///` for items, `//!` for modules

## Engineering Preferences

- **Simplicity over compatibility**: Make the simplest change possible. Don't add migration paths, backwards-compatibility shims, or deprecation layers. Delete old code, rename freely, refactor aggressively.
- **Readability over cleverness**: Code should be obvious. Prefer larger, clearer changes over minimal but obscure ones.
- **SOLID principles**: Follow in all designs.
- **Explicit function names over comments**: Do not add comments unless absolutely necessary.
- **Functional patterns**: Embrace immutability, avoid side effects where possible.
- **No dead code**: Never use `#[allow(dead_code)]` except in tests.

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

### Test Requirements

- Write tests concurrently with implementation
- Use mock providers for isolated testing
- Include edge case and error scenario coverage
- All new scripts require corresponding tests in `tests/scripts/`

## Git Workflow

**CRITICAL: Never commit directly to main branch.**

```bash
# 1. Create feature branch FIRST
./scripts/ai-branch.sh feature_name

# 2. Make changes and commit
git add -A && git commit -m "feat: Description"

# 3. Create PR when ready
./scripts/ai-pr.sh

# 4. Only update main with
./scripts/ai-update-main.sh
```

### Commit Format

```
<type>: <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Commit Guidelines

- Keep commits small and focused on a single change
- Ensure tests pass before committing
- Write clear, descriptive commit messages
- Push frequently to avoid large divergences

### Git Safety

- Never force-push without explicit direction
- Pipe git output through `cat` to prevent pager: `git log | cat`
- All git commands must use non-interactive flags

## Planning

When asked to plan a refactor or feature, write the plan locally in `plans/` using filename `plans/YYYYMMDD-topic.md`.

### Plan Creation

- Plans should be as detailed as possible and include interfaces/protocols when needed
- Use the filename convention `plans/YYYYMMDD-topic.md` with lowercase kebab-case topic
- See `plans/TEMPLATE.md` for the standard plan format
- Use checklist steps (`[ ]` / `[x]`) for tracking progress

### Progressive Updates During Implementation

**Agents must update plans progressively as implementation proceeds.** This is critical for maintaining context across sessions and enabling handoffs.

Update the plan's `Implementation State` section:
- **After completing each step**: Mark the step as `[x]` complete
- **When starting a new step**: Update "Current step" to reflect active work
- **After each commit**: Add the commit hash and description to the Commits list
- **When encountering issues**: Document blockers, workarounds, or scope changes
- **At session end**: Ensure state reflects exactly where work stopped

**IMPORTANT: Avoiding the commit-hash loop.** When recording commits in plan files:
1. First commit the implementation changes (without the commit hash in the plan)
2. Then update the plan file with the commit hash as a SEPARATE commit
3. NEVER use `git commit --amend` after updating a plan with a commit hash

Required fields to maintain:
```markdown
## Implementation State

**State**: In Progress | Complete | Blocked
**Current step**: Step N - Description
**Last updated**: YYYY-MM-DD

### Completed Steps
- [x] Step 1: Description (commit abc123)
- [ ] Step 2: Description

### Additional Notes
- Document any deviations from the original plan
- Record bug fixes or improvements discovered during implementation

### Commits
- `abc1234` feat: First change
- `def5678` fix: Bug discovered during implementation
```

## Boundaries

### Always Safe

- Read any file in the project
- Run `cargo build`, `cargo test`, `cargo clippy`
- Modify files in `src/` and `tests/`
- Add new Rust files following existing patterns
- Run `git status`, `git diff`, `git log`, `git add`, `git commit`
- Use helper scripts in `scripts/`

### Ask First

- Modify `Cargo.toml` (dependency changes)
- Change database schema
- Modify security-related code
- Add new external dependencies

### Never Do

- Expose API keys or secrets in code
- Commit configuration files with credentials
- Work directly on main branch
- Use `#[allow(dead_code)]` outside of tests
- Edit README.md directly (use `./scripts/ai-readme.sh`)
- Force-push without explicit direction

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

## Pull Request Guidelines

### Creating PRs

```bash
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2" --implementation "Detail 1,Detail 2"

# For draft PRs
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1" --draft
```

### PR Structure

- **Overview**: Brief summary (2-3 sentences)
- **Key Changes**: Bullet points of significant changes
- **Implementation Details**: Technical approach
- **Migration Notes**: Breaking changes (if any)
- **Areas of Attention**: Review focus areas

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

### Common Issues

- **Branch protection errors**: Always verify branch before work
- **Test failures**: Check if code changed before modifying tests
- **Clippy warnings**: Address in source code, not with allow directives
- **Build failures**: Ensure all dependencies are up to date

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

---

*This file follows the [AGENTS.md format](https://agents.md/) for AI coding agents. For human contributors, see [CONTRIBUTING.md](./CONTRIBUTING.md).*
