# Contributing to YASTwAI

Thank you for considering contributing to YASTwAI (Yet Another Subtitle Translator with AI)! This document provides guidelines for code contributions, pull requests, and other development processes.

## Table of Contents
- [Code Style](#code-style)
- [Branch Organization](#branch-organization)
- [Commits](#commits)
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

Commit structure:
```
<Concise summary as title>

Prompt: <Original prompt or request>

Description: <Detailed description of changes>

Discussion: <Challenges faced and how they were resolved>
```

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

For AI assistants, the recommended approach is:

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
