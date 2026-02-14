# CLAUDE.md

## Session Start

Read the latest handoff in docs/summaries/ if one exists. Load only the files that handoff references — not all summaries. If no handoff exists, ask: what is the project, what type of work, what is the target deliverable.

Before starting work, state: what you understand the project state to be, what you plan to do this session, and any open questions.

## Identity

You work on **YASTwAI** (Yet Another Subtitle Translator with AI), an async Rust CLI for extracting and translating video subtitles via AI providers. Stack: Rust 2024 (1.85+), Tokio, clap v4, rusqlite, reqwest.

## Rules

1. Do not mix unrelated project contexts in one session.
2. Write state to disk, not conversation. After completing meaningful work, write a summary to docs/summaries/ using templates from templates/claude-templates.md.
3. Before compaction or session end, write to disk: every decision with rationale, every open question, every file path, exact next action.
4. When switching work types (research > writing > review), write a handoff and suggest a new session.
5. Do not silently resolve open questions. Mark them OPEN or ASSUMED.
6. Do not bulk-read documents. Process one at a time per docs/context/processing-protocol.md.
7. Sub-agent returns must be structured. Use output contracts from templates/claude-templates.md.

## Where Things Live

- templates/claude-templates.md — summary, handoff, decision, analysis, task, output contract templates
- docs/summaries/ — active session state
- docs/context/ — reusable domain knowledge
- docs/archive/ — processed raw files (do not read unless told)
- output/deliverables/ — final outputs

## Commands

| Command | Description |
|---------|-------------|
| `cargo build --release` | Production build |
| `cargo build` | Development build |
| `cargo test` | Run all tests |
| `./scripts/ai-clippy.sh --check-only` | Check linting |
| `./scripts/ai-clippy.sh --fix` | Auto-fix lint issues |

## Architecture

- **Providers**: AI backends (Ollama, OpenAI, Anthropic) via trait interface
- **Translation**: Multi-pass pipeline with batching and quality validation
- **Subtitles**: FFmpeg extraction, SRT parsing/generation
- **Persistence**: SQLite for sessions and caching

## Conventions

- Never work directly on main branch
- Never use `#[allow(dead_code)]` outside tests

## Codex Auto-Delegation

Proactively spawn the **Codex Analyst** agent (`subagent_type: "Codex Analyst"`) for complex analysis — multi-file bugs, architecture tracing, large diffs, performance analysis. Uses gpt-5.3-codex with xhigh reasoning, read-only.

## Boundaries

### Safe
- Read any project file, modify `src/` and `tests/`, run cargo commands and helper scripts, create commits

### Ask First
- Modify `Cargo.toml`, change database schema, add external dependencies

### Never
- Push to main directly, expose API keys in code, commit secrets, delete tests without approval
