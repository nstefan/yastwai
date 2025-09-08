### YASTwAI Project Audit Plan

This document outlines best-practice improvements and a staged plan to address issues found during a quick architecture and code quality review of the repository.

#### High-level findings
- Configuration and provider abstractions are modular and test-covered.
- Async hygiene gaps: blocking calls within async paths, mixed mutex types, and spawning nested runtimes.
- Logging inconsistency: mixed log-level casing; a few panic-prone `unwrap/expect` usages in non-critical paths.
- CLI parsing is manual; error handling mixes `anyhow` with a richer domain error module that is underused.
- Performance opportunities: translation cache exists but is unused; concurrency/rate limits not fully driven by config; token usage stats not aggregated across batches.
- Dependencies: `reqwest` includes the "blocking" feature although a blocking client does not appear to be used.

---

### Staged plan

#### Stage 1 — Safety, correctness, and consistency (low risk, high value)
- Standardize log levels
  - Normalize `LogEntry.level` casing (use one convention everywhere).
  - Ensure counts for ERROR/WARN are accurate.
- Remove panics in non-critical paths
  - Replace `.expect(...)` and `.unwrap()` in non-critical code paths with error handling.
  - Examples: progress bar template creation and semaphore acquisition.
- Async hygiene
  - Replace `std::sync::Mutex` with `tokio::sync::Mutex` in async code paths (e.g., log capture).
  - Replace `std::thread::spawn + tokio::runtime::Runtime::new().block_on` with `tokio::spawn`.
- Config validation
  - Call `config.validate()?` right after loading/overriding config in `main`.
- Dependencies
  - Remove `reqwest` "blocking" feature if not required.

Success criteria:
- No `.expect` or `.unwrap` in non-critical paths.
- Uniform log level strings; error/warn counts are correct.
- No std mutexes in async context.
- Clippy is clean without new warnings.

#### Stage 2 — Async I/O and non-blocking subprocesses
- File and process I/O
  - Convert blocking uses of `std::process::Command` to `tokio::process::Command` where used in async flows, or offload via `tokio::task::spawn_blocking`.
- Structured concurrency
  - Use `tokio::select!` for cancellation where long-running external calls are involved.

Success criteria:
- No long blocking operations in async tasks; better responsiveness on large directory runs.

#### Stage 3 — Config-driven concurrency, retries, and rate limits
- Concurrency
  - Wire `TranslationConfig::optimal_concurrent_requests()` to `TranslationOptions.max_concurrent_requests` so `BatchTranslator` respects config, not defaults.
- Retries/backoff
  - Apply `retry_count` and `retry_backoff_ms` from config consistently for all providers.
- Rate limiting
  - Respect provider-specific rate limits (OpenAI/Anthropic); optionally allow user-configured throttling for Ollama.

Success criteria:
- Concurrency matches config values.
- Backoff/retry and throttling verified by tests.

#### Stage 4 — Performance features
- Translation cache
  - Integrate `TranslationCache` to avoid redundant translation calls within a run.
- Token usage aggregation
  - Aggregate token usage across batches and display accurate totals at the end.

Success criteria:
- Fewer duplicate API calls; visible performance gains on repeated segments.
- Accurate final token usage summary.

#### Stage 5 — CLI and developer ergonomics
- CLI
  - Migrate manual CLI parsing to `clap` derive for better UX and validation.
- Errors
  - Keep `anyhow` in the binary; prefer domain errors (`errors.rs`) inside library modules.
- Logging
  - Keep custom logger but provide an option to use env logger for simpler environments.

Success criteria:
- Cleaner argument handling and help.
- More precise test assertions via typed errors.

#### Stage 6 — Tests, lint, and docs
- Tests
  - Add tests for `FormatPreserver` edge cases and `file_utils::detect_file_type`.
  - Add tests for retry/backoff and rate-limiter behavior with time control.
- Lints
  - Remove broad `#![allow(...)]` in `main.rs` or scope them narrowly after addressing causes.
- Docs
  - Add `docs/ARCHITECTURE.md` and `docs/CONFIGURATION.md` as the code evolves.
  - Regenerate `README.md` via `scripts/ai-readme.sh` after notable changes per project rules.

Success criteria:
- Stable CI with tests and clippy.
- Clear architecture and configuration documentation.

---

### Hotspots to prioritize early
- `src/app_controller.rs`: Replace nested runtime spawn with `tokio::spawn`; normalize log levels; remove panic-prone `.expect`.
- `src/translation/core.rs`: Normalize log levels; wire config for concurrency/retry; avoid `unwrap()` in clone path.
- `src/translation/batch.rs`: Avoid `.unwrap()` on semaphore acquire; prefer `tokio::sync::Mutex` for log capture.
- `src/file_utils.rs`: Avoid blocking process I/O in async flows.
- `Cargo.toml`: Remove unused `reqwest` "blocking" feature.

---

### Execution notes
- Implement changes stage-by-stage and commit separately for easy review.
- After provider/config changes, regenerate `README.md` with `./scripts/ai-readme.sh` and commit.


