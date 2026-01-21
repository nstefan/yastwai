# Agent Specification - Entry Point

This specification works with any AI coding agent (Claude Code, Cursor, Copilot, etc.).

## Loading Instructions

This is a **Rust** project. Load:
- `agents/core.md` - Language-agnostic specification
- `agents/rust.md` - Rust-specific overlay

## Quick Start

| Command | Description |
|---------|-------------|
| `cargo build --release` | Production build |
| `cargo build` | Development build |
| `cargo test` | Run all tests |
| `./scripts/ai-clippy.sh --check-only` | Check linting |
| `./scripts/ai-clippy.sh --fix` | Auto-fix lint issues |
| `./scripts/ai-branch.sh feature_name` | Create feature branch |
| `./scripts/ai-pr.sh` | Create pull request |

## File Structure

```
yastwai/
├── AGENTS.md               # This file (entry point, source of truth)
├── CLAUDE.md -> AGENTS.md  # Symlink for Claude Code
├── .claude/
│   └── settings.json       # Claude Code settings (plansDirectory, permissions)
├── agents/
│   ├── core.md             # Language-agnostic spec (always load)
│   ├── rust.md             # Rust overlay for this project
│   ├── research/           # Index cards pointing to reference/
│   └── reference/          # Full offline content (SEARCH, don't load)
└── plans/                  # Implementation plans (use TEMPLATE.md)
    └── TEMPLATE.md         # Plan format template
```

## Section Map

| Section | File | Content |
|---------|------|---------|
| 0. Orientation | `agents/core.md` | Project context |
| 1. Goals | `agents/core.md` | Build, test, lint, run |
| 2. Code Style | `agents/core.md` | Universal principles |
| 3. Git Workflow | `agents/core.md` | Commits, branches, PRs |
| 4. Testing | `agents/core.md` | Testing philosophy |
| 5. Engineering | `agents/core.md` | Simplicity, anti-patterns |
| **6. Planning** | `agents/core.md` | **Use `plans/TEMPLATE.md`** |
| 7. Troubleshooting | `agents/core.md` | Debug strategy |
| **8. Skills Discovery** | `agents/core.md` | **Auto-suggest skills.sh packages** |
| **99999. Boundaries** | `agents/core.md` | **CRITICAL: Always/Ask/Never** |
| Rust Specifics | `agents/rust.md` | Tooling, patterns, conventions |

## Planning (IMPORTANT)

**When creating plans, ALWAYS use `plans/TEMPLATE.md` format and save to `plans/YYYYMMDD-topic.md`.**

This overrides any tool-specific planning instructions (e.g., Claude Code's built-in plan mode). The project's planning format takes precedence.

## Reference Materials (Search Only)

**Do not load reference/ into context.** Search when needed:

```bash
grep -r "Plan-Then-Execute" agents/reference/
grep -r "Lethal Trifecta" agents/reference/
grep -r "Reflection Loop" agents/reference/
```

| Reference File | Content |
|----------------|---------|
| `good-spec-full.md` | Six core areas, three-tier boundaries |
| `agentic-handbook-full.md` | 113 patterns, security framework |
| `agentic-patterns-full.md` | 130+ patterns by category |
| `ralph-wiggum-full.md` | Loop mechanics, steering techniques |

**When to search**: Pattern implementations, security guidance, multi-agent architectures, feedback loops, boundary setup.

---

## Project Overview

**YASTwAI** (Yet Another Subtitle Translator with AI) is an async Rust CLI that extracts subtitles from video files and translates them using multiple AI providers (Ollama, OpenAI, Anthropic).

### Architecture

- **Providers**: AI backend implementations (Ollama, OpenAI, Anthropic) via trait interface
- **Translation**: Multi-pass pipeline with document context, batching, and quality validation
- **Subtitle Processing**: FFmpeg integration for extraction, SRT parsing and generation
- **Session/Database**: SQLite persistence for sessions and caching
- **Configuration**: JSON-based config with provider-specific settings

### Tech Stack

- **Language**: Rust (Edition 2024, 1.85.0+)
- **Async Runtime**: Tokio (full features)
- **HTTP Client**: reqwest
- **CLI**: clap v4
- **Database**: rusqlite (bundled SQLite)
- **Serialization**: serde + serde_json
- **Error Handling**: thiserror + anyhow

### Helper Scripts

| Script | Description |
|--------|-------------|
| `ai-branch.sh` | Branch management with named parameters |
| `ai-clippy.sh` | Enhanced Clippy with fix options |
| `ai-pr.sh` | PR creation with structured descriptions |
| `ai-protect-main.sh` | Branch protection verification |
| `ai-update-main.sh` | Safe main branch updates |
| `ai-readme.sh` | Generate README.md |

### Critical Boundaries

**Always Safe**:
- Read any file, run `cargo build/test/clippy`, modify `src/` and `tests/`

**Ask First**:
- Modify `Cargo.toml`, change database schema, add external dependencies

**Never Do**:
- Work directly on main branch, expose API keys, use `#[allow(dead_code)]` outside tests

---

*This file follows the [AGENTS.md format](https://agents.md/) for AI coding agents. For human contributors, see [CONTRIBUTING.md](./CONTRIBUTING.md).*
