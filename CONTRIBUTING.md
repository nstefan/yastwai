# Contributing to YASTwAI

Thank you for considering contributing to YASTwAI (Yet Another Subtitle Translator with AI)!

## Code Style

- Follow Rust's official style guidelines
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- All code, comments, and documentation must be in English

## Development Workflow

1. Fork the repository and clone your fork
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy --all-targets`
6. Commit your changes with a descriptive message
7. Push to your fork and create a pull request

## Branch Naming

- `feature/<description>` - New features
- `fix/<description>` - Bug fixes
- `refactor/<description>` - Code refactoring
- `docs/<description>` - Documentation changes

## Pull Request Guidelines

- Keep PRs focused on a single change
- Include a clear description of what and why
- Add tests for new functionality
- Update documentation as needed

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Building

```bash
# Development build
cargo build

# Release build
cargo build --release
```
