# YASTWAI Development Guide

## ⚠️ CRITICAL WORKFLOW RULES ⚠️
- **NEVER commit directly to main branch** - Always create a feature branch first
- For ALL changes, follow this exact workflow:
  1. Create branch: `./scripts/ai-branch.sh feature_name`
  2. Make changes
  3. Commit: `./scripts/ai-commit.sh [options]`
  4. Create PR: `./scripts/ai-pr.sh`
  5. Only update main with: `./scripts/ai-update-main.sh`
- Never edit README.md directly, use ai-readme.sh

## Commands
```bash
# Branch Workflow (ALWAYS USE THESE)
./scripts/ai-branch.sh feature_name  # Create feature branch FIRST
./scripts/ai-commit.sh --model "model-name" "Title" "Description" "Prompt" "Process" "Discussion"
./scripts/ai-pr.sh  # Create pull request, NEVER push directly to main
./scripts/ai-update-main.sh  # Update main branch

# Build
cargo build --release  # Production build
cargo build  # Development build

# Test
cargo test  # Run all tests
cargo test test_name  # Run single test
YASTWAI_TEST_LOG=1 cargo test  # Tests with logging

# Lint/Format
./scripts/ai-clippy.sh --check-only  # Check linting
./scripts/ai-clippy.sh --fix  # Auto-fix lint issues
```

## Code Style
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Imports**: Group by std → external → internal, explicit imports (no globs)
- **Errors**: Use thiserror, proper propagation with `?`, context in messages
- **Documentation**: `///` for functions, `/*!` for modules
- **Tests**: Name pattern `test_functionName_withCondition_shouldBehavior`
- **Rust Patterns**: Embrace functional programming, immutability, tokio for async
- **Concurrency**: Use structured patterns, appropriate channel types (mpsc, broadcast)
- **SOLID**: Follow SOLID principles in design

## Additional Rules
- Test any new scripts following patterns in tests/scripts/
- Structure PRs with clear sections (Instructions, Overview, Changes)
- Only create PRs when explicitly requested using ai-pr.sh
- Maintain all code, comments, and docs in English