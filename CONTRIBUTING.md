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

Use the `scripts/create-commit.sh` script to generate properly formatted commits:

```
./scripts/create-commit.sh <title> <prompt> <description> <discussion>
```

Each commit message should follow this structure:

```
<Concise summary as title>

Prompt: <Original prompt or request>

Description: <Detailed description of changes>

Discussion: <Challenges faced and how they were resolved>
```

## Development Workflow

YASTwAI follows a branch-based workflow with specific requirements and helper scripts to ensure consistency.

### Branch Management

1. Development happens on feature branches, not directly on `main`
2. Each feature or fix gets its own branch
3. Use the `branch-check.sh` script to manage branches:

```bash
./scripts/branch-check.sh
```

The script will:
- Check if you're on the `main` branch
- Prompt you to create a new branch if needed
- Check if your new work is related to the current branch
- Guide you through switching to a new branch when appropriate

### Making Changes

1. Make your changes, following the project's coding standards
2. Add tests for new functionality
3. Ensure all tests pass
4. Update documentation as needed
5. Create commits using the project's commit script
6. Push your branch to your fork
7. Create a pull request

### Standard Workflow Example

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

### Special Considerations

- All work must be done in English (code, comments, documentation)
- Follow Rust best practices and coding standards
- When fixing tests, always check if the code is the problem before modifying the test
- Never use `#[allow(dead_code)]` directives except for test-related code

## Pull Requests

### Bot-Friendly PR Creation

For automated PR creation, especially for AI assistants or bots, use the `create-pr.sh` script with the following syntax:

```bash
./scripts/create-pr.sh --body "Your PR description with \n for newlines" --template
```

Key parameters:
- `--title "PR Title"` - Optional, will auto-generate if omitted
- `--body "Description with \n for newlines"` - PR body with escaped newlines
- `--body-file path/to/file.md` - Alternative to --body, use a file instead
- `--template` - Use the template from scripts/pr-template.md
- `--compact` - Generate a more compact PR description
- `--summary-only` - Only include a brief summary of changes
- `--draft` - Create as draft PR

For AI assistants, we provide a specialized helper script that avoids multiline string issues:

```bash
./scripts/ai-pr-helper.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2"
```

Key parameters for AI PR helper:
- `--title` - PR title (required)
- `--overview` - Brief overview of the PR (required)
- `--key-changes` - Comma-separated list of key changes
- `--implementation` - Comma-separated list of implementation details
- `--files` - Comma-separated list of files changed (optional, will auto-detect if omitted)
- `--commits` - Comma-separated list of commit descriptions (optional, will auto-detect if omitted)

The alternative approach using the standard script:

```bash
./scripts/create-pr.sh --body "## 📋 PR Overview\n\nThis PR implements feature X by...\n\n## 🔍 Implementation Details\n\n- Added new component\n- Fixed error handling" --template
```

### PR Structure Best Practices

Well-structured PRs should include:

1. **Overview**: Brief summary of what the PR accomplishes (2-3 sentences)
2. **Key Changes**: Bullet points of the most significant changes
3. **Implementation Details**: Technical approach and design decisions
4. **Testing**: How changes were tested
5. **Related Issues**: References to related issues or tickets

Use emojis for better readability:
- 📌 For overview sections
- 🔍 For key changes
- 🧩 For implementation details
- 🔄 For migration notes
- ⚠️ For areas needing special attention
- 📝 For commit details
- 📁 For file changes

## Automated Tools

The repository includes several scripts to help with development:
- `scripts/create-commit.sh` - For creating properly formatted commits
- `scripts/create-pr.sh` - For creating pull requests with proper descriptions
- `scripts/run-clippy.sh` - For running the Rust linter

Use these tools to ensure consistency across contributions.
