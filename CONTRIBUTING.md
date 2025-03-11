# Contributing to YASTwAI

Thank you for considering contributing to YASTwAI (Yet Another Subtitle Translator with AI)! This document provides guidelines for code contributions, pull requests, and other development processes.

## Table of Contents
- [Code Style](#code-style)
- [Branch Organization](#branch-organization)
- [Commits](#commits)
- [Development Workflow](#development-workflow)
- [Pull Requests](#pull-requests)
- [Automated Tools](#automated-tools)

## Code Style

- All code should follow Rust's official style guidelines
- Use functional programming approaches where appropriate
- Maintain immutability where possible
- Document all public functions and types
- Write clear error messages and proper error handling
- All code, comments, and documentation must be in English

## Branch Organization

- `main` - The primary branch containing stable code
- `feature/<feature-name>` - For new features
- `fix/<bug-description>` - For bug fixes
- `refactor/<component>` - For code refactoring
- `docs/<documentation-change>` - For documentation-only changes

## Commits

### Human Contributors

Use the `scripts/create-commit.sh` script to generate properly formatted commits:

```
./scripts/create-commit.sh <title> <short-description> <prompt> <chain-of-thoughts> <discussion>
```

### AI Assistants

AI assistants should use the optimized non-interactive script:

```
./scripts/ai-commit-helper.sh --title "Commit title" --description "Short description" --prompt "Original prompt" --thought-process "Line 1,Line 2" --discussion "Challenge faced,Solution implemented"
```

Each commit message should follow this structure:

```
<Concise summary as title>

Short description: <Brief description of the changes>

Prompt: <Original prompt or request>

Chain of thoughts: <Reasoning process used by the agent>

Discussion: <Challenges faced and how they were resolved>
```

## Development Workflow

YASTwAI follows a branch-based workflow with specific requirements and helper scripts to ensure consistency.

### Branch Management

1. Development happens on feature branches, not directly on `main`
2. Each feature or fix gets its own branch
3. Human contributors use the `branch-check.sh` script to manage branches:

```bash
./scripts/branch-check.sh
```

4. AI assistants should use the optimized non-interactive script:

```bash
./scripts/ai-branch-helper.sh --check-only                          # Check branch status only
./scripts/ai-branch-helper.sh --new-branch "feature-name" --is-related false  # Create new branch from main
```

These scripts help:
- Check if you're on the `main` branch
- Create a new branch when needed
- Check if your new work is related to the current branch
- Guide you through switching to a new branch when appropriate

### Making Changes

1. Make your changes, following the project's coding standards
2. Add tests for new functionality
3. Ensure all tests pass
4. Update documentation as needed
5. Create commits using the appropriate commit script
6. Push your branch to your fork
7. Create a pull request

### Code Quality Checks

For running Rust's linting tools:

1. Human contributors:
```bash
./scripts/run-clippy.sh
```

2. AI assistants:
```bash
./scripts/ai-clippy-helper.sh --check-only    # Check for issues
./scripts/ai-clippy-helper.sh --fix           # Auto-fix issues
```

### Standard Workflow Example

#### For Human Contributors

```bash
# 1. Check branch status and create a new branch if needed
./scripts/branch-check.sh
# Follow the prompts to create or use an appropriate branch

# 2. Make your code changes
# ...

# 3. Create a properly formatted commit
./scripts/create-commit.sh "Add new feature X" "Add support for feature X that does Y"
# Complete the description and discussion sections in your editor

# 4. Build and test
cargo build --release
cargo test
```

#### For AI Assistants

```bash
# 1. Check branch status and create a new branch if needed 
./scripts/ai-branch-helper.sh --new-branch "feature-x" --is-related false

# 2. Make code changes
# ...

# 3. Run linting checks
./scripts/ai-clippy-helper.sh --fix

# 4. Create a properly formatted commit
./scripts/ai-commit-helper.sh --title "Add feature X" --description "Add support for feature X that does Y" --prompt "Original request" --thought-process "First I did this,Then I did that" --discussion "Had to overcome this issue,Resolved by using this approach"

# 5. Build and test
cargo build --release
cargo test
```

### Special Considerations

- All work must be done in English (code, comments, documentation)
- Follow Rust best practices and coding standards
- When fixing tests, always check if the code is the problem before modifying the test
- Never use `#[allow(dead_code)]` directives except for test-related code

## Pull Requests

### Bot-Friendly PR Creation

For automated PR creation, especially for AI assistants, use the `ai-pr-helper.sh` script with the following syntax:

```bash
./scripts/ai-pr-helper.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2" --implementation "Detail 1,Detail 2"
```

Key parameters for AI PR helper:
- `--title` - PR title (required)
- `--overview` - Brief overview of the PR (required)
- `--key-changes` - Comma-separated list of key changes
- `--implementation` - Comma-separated list of implementation details
- `--files` - Comma-separated list of files changed (optional, will auto-detect if omitted)
- `--commits` - Comma-separated list of commit descriptions (optional, will auto-detect if omitted)
- `--draft` - Create as draft PR

This script avoids multiline command issues and creates well-structured PRs. It automatically handles creating the PR using either GitHub CLI if available, or directly through the GitHub API.

### PR Structure Best Practices

Well-structured PRs should include:

1. **Instructions** (AI-generated PRs only)
   - Prompt: Original request
   - Chain of thoughts: Reasoning process

2. **Overview**: Brief summary of what the PR accomplishes (2-3 sentences)

3. **Key Changes**: Bullet points of the most significant changes

4. **Implementation Details**: Technical approach and design decisions

5. **Testing**: How changes were tested

6. **Related Issues**: References to related issues or tickets

Use emojis for better readability:
- 🧠 For instructions section (AI PRs)
- 📌 For overview sections
- 🔍 For key changes
- 🧩 For implementation details
- 🔄 For migration notes
- ⚠️ For areas needing special attention
- 📝 For commit details
- 📁 For file changes

## Automated Tools

The repository includes several scripts to help with development:

### Standard Scripts (Interactive)
- `scripts/create-commit.sh` - For creating properly formatted commits (interactive)
- `scripts/branch-check.sh` - For managing branches (interactive)
- `scripts/run-clippy.sh` - For running the Rust linter (basic)

### AI-Optimized Scripts (Non-Interactive)
- `scripts/ai-commit-helper.sh` - For creating commits with named parameters
- `scripts/ai-branch-helper.sh` - For branch management with named parameters
- `scripts/ai-clippy-helper.sh` - For running Clippy with more options and better output
- `scripts/ai-pr-helper.sh` - For creating pull requests with proper descriptions

These tools ensure consistency across contributions and make it easier for both human developers and AI assistants to follow project conventions.
