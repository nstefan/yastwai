# Agent Specification

## Project

**Name**: YASTwAI (Yet Another Subtitle Translator with AI)
**Purpose**: Async Rust CLI for extracting and translating video subtitles via AI providers
**Stack**: Rust 2024 (1.85+), Tokio, clap v4, rusqlite, reqwest

## Commands

| Command | Description |
|---------|-------------|
| `cargo build --release` | Production build |
| `cargo build` | Development build |
| `cargo test` | Run all tests |
| `./scripts/ai-clippy.sh --check-only` | Check linting |
| `./scripts/ai-clippy.sh --fix` | Auto-fix lint issues |
| `./scripts/ai-branch.sh feature_name` | Create feature branch |
| `./scripts/ai-pr.sh` | Create pull request |

## Architecture

- **Providers**: AI backends (Ollama, OpenAI, Anthropic) via trait interface
- **Translation**: Multi-pass pipeline with batching and quality validation
- **Subtitles**: FFmpeg extraction, SRT parsing/generation
- **Persistence**: SQLite for sessions and caching

## Conventions

- Never work directly on main branch
- Never use `#[allow(dead_code)]` outside tests

## Boundaries

### Safe
- Read any project file
- Modify `src/` and `tests/`
- Run cargo commands, helper scripts
- Create commits, push to feature branches

### Ask First
- Modify `Cargo.toml`
- Change database schema
- Add external dependencies

### Never
- Push to main directly
- Expose API keys in code
- Commit secrets or credentials
- Delete tests without approval
