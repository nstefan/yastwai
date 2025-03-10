# Contributing to YASTwAI

Thank you for your interest in contributing to YASTwAI (Yet Another Subtitles Translation with AI)! This document provides guidelines and workflows to help you contribute effectively.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Commit Guidelines](#commit-guidelines)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

Contributors are expected to maintain a respectful and inclusive environment. Be considerate of different perspectives and experiences, and focus on constructive collaboration.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Set up the development environment
4. Create a feature branch from `main`

```bash
git clone https://github.com/YOUR-USERNAME/yastwai.git
cd yastwai
git checkout -b feature/your-feature-name
```

## Development Workflow

1. Make your changes, following the project's coding standards
2. Add tests for new functionality
3. Ensure all tests pass
4. Update documentation as needed
5. Commit your changes using the project's commit guidelines
6. Push your branch to your fork
7. Create a pull request

## Pull Request Process

### Using the Automated PR Script

YASTwAI provides a `create-pr.sh` script to automate PR creation with smart defaults:

#### Basic Usage

```bash
./scripts/create-pr.sh
```

This will:
- Use the first commit message as the PR title
- Generate a PR body from all commit details and file changes
- Target the `main` branch
- Open your browser with the PR creation page pre-filled

#### Advanced Options

```bash
./scripts/create-pr.sh --title "Custom PR Title" --body FILE --base main --draft
```

Available options:
- `--title TITLE`: Set a custom PR title
- `--body FILE`: Use contents of FILE as PR body 
- `--base BRANCH`: Set the base branch to merge into (default: main)
- `--draft`: Create as draft PR
- `--help`: Display help message

### PR Requirements

All pull requests should:
- Have a clear and descriptive title
- Include a comprehensive description of changes
- Reference any related issues
- Pass all CI checks
- Be reviewed by at least one maintainer
- Follow the PR template provided

## Commit Guidelines

YASTwAI follows the standards defined in the `yastwai.mdc` ruleset for commit messages. Each commit should provide clear information about what changes were made, why they were made, and any challenges encountered.

### Commit Message Structure

Each commit message should follow this structure:

```
<Concise summary as title>

Prompt: <Original prompt or request>

Description: <Detailed description of changes>

Discussion: <Challenges faced and how they were resolved>
```

#### Components

1. **Title**: A concise, one-line summary of the changes (50 characters or less)
   - Use imperative mood (e.g., "Add feature" not "Added feature")
   - Capitalize the first word
   - No period at the end

2. **Prompt**: The original user request that led to these changes
   - Include the complete text of the prompt
   - For non-prompted changes, briefly describe the reason for the change

3. **Description**: A detailed explanation of what was changed and why
   - List specific modifications made
   - Explain the rationale behind implementation choices
   - Use bullet points for clarity when appropriate
   - Keep each line under 72 characters

4. **Discussion**: A summary of challenges encountered and their solutions
   - Include any technical difficulties faced
   - Mention alternate approaches considered
   - Document any lessons learned
   - Note any future improvements or follow-up tasks

### Example Commit Message

```
Add selective Clippy lint configuration

Prompt: Read forum thread and provide guidance on how to update my project to follow this recommendation.

Description: Added configuration to selectively suppress Clippy auto-fixes for specific lints:
1. Created a clippy.toml file with proper configurations
2. Added global lint configurations in lib.rs and main.rs to prevent unwanted auto-fixes
3. Created a run-clippy.sh script to run Clippy with specific flags
4. Updated README.md with instructions for using the new Clippy configuration

Discussion: Faced issues with the correct syntax for clippy.toml (using hyphens instead of underscores) and the proper command for cargo fix. The solution follows the forum recommendation by explicitly allowing certain lints in the codebase to prevent automatic fixing while still showing the warnings.
```

### Using the Commit Script

A helper script `./scripts/create-commit.sh` is provided to streamline the commit message creation process:

```bash
./scripts/create-commit.sh "Commit title" "Original prompt"
```

The script will:
1. Create a template commit message
2. Open your default editor to complete the message
3. Show a preview of the final message
4. Stage changes and create the commit

### Branching Guidelines

When working on a new feature or fix:

1. Check if you're already on a feature branch (other than "main")
2. If working on a new feature, create a new branch from main
3. Ensure the branch name reflects the purpose of the changes
4. When ready, commit your changes following the message format outlined above

### Final Steps

After committing changes:

1. Build the application in release mode
2. Run unit tests to ensure everything works correctly
3. Fix any issues identified during testing
4. Create a new commit for any fixes needed

## Coding Standards

- Keep all code, comments, and documentation in English
- Never use `#[allow(dead_code)]` directives except for test-related code
- When a test breaks, first check if the code change is problematic before modifying the test
- Follow Rust's official style guidelines
- Use functional programming patterns where appropriate
- Maintain immutability when possible

## Testing

## Documentation
