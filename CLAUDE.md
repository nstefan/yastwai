# YASTWAI Development Guide

## Commands
```bash
# Build
cargo build --release  # Production build
cargo build  # Development build

# Test
cargo test  # Run all tests
cargo test test_name  # Run single test
YASTWAI_TEST_LOG=1 cargo test  # Tests with logging
cargo test unit::app_config_tests  # Test specific module

# Lint/Format
./scripts/ai-clippy.sh --check-only  # Check linting
./scripts/ai-clippy.sh --fix  # Auto-fix lint issues

# Workflow Scripts
./scripts/ai-branch.sh  # Create feature branch
./scripts/ai-commit.sh  # Create well-formatted commit
./scripts/ai-pr.sh  # Create pull request
./scripts/ai-update-main.sh  # Update main branch
./scripts/ai-readme.sh  # Generate README.md
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

## Workflow Rules
- Never work directly on main branch
- Never edit README.md directly, use ai-readme.sh
- Run linting before committing with ai-clippy.sh
- Test any new scripts following patterns in tests/scripts/
- Structure PRs with clear sections (Instructions, Overview, Changes)
- Only create PRs when explicitly requested using ai-pr.sh
- Maintain all code, comments, and docs in English