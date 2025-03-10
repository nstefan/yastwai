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
./create-pr.sh
```

This will:
- Use the first commit message as the PR title
- Generate a PR body from all commit details and file changes
- Target the `main` branch
- Open your browser with the PR creation page pre-filled

#### Advanced Options

```bash
./create-pr.sh --title "Custom PR Title" --body-text "Custom PR description" --base main --draft
```

Available options:
- `--title TITLE`: Set a custom PR title
- `--body FILE`: Use contents of FILE as PR body 
- `--body-text TEXT`: Use TEXT as PR body (supports escaped newlines with \n)
- `--base BRANCH`: Set the base branch to merge into (default: main)
- `--draft`: Create as draft PR
- `--no-generate`: Skip auto-generation of PR body
- `--no-template`: Skip using the GitHub PR template
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

YASTwAI uses structured commit messages to maintain a clear project history. You can use the provided `scripts/create-commit.sh` script for consistent commit formatting:

```bash
./scripts/create-commit.sh --title "Brief description" --prompt "Original task" --description "Detailed explanation" --discussion "Challenges faced"
```