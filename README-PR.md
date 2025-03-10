# Pull Request Automation for YASTwAI

This document explains how to use the `create-pr.sh` script to automate Pull Request creation for the YASTwAI project.

## PR for Current Branch: `update-create-commit-script`

The current branch contains changes that:

1. Added a non-interactive `create-commit.sh` script optimized for bot usage
2. Updated the Cursor rules for clarity and better organization

## Using `create-pr.sh`

The `create-pr.sh` script automates PR creation with smart defaults while providing flexibility through command-line options.

### Basic Usage

To create a PR using the current branch with automatically generated title and description:

```bash
./create-pr.sh
```

This will:
- Use the first commit message as the PR title
- Generate a PR body from all commit details and file changes
- Target the `main` branch
- Create the PR using GitHub CLI

### Advanced Options

```bash
./create-pr.sh --title "Custom PR Title" --body-text "Custom PR description" --base main --draft
```

#### Available Options

- `--title TITLE`: Set a custom PR title
- `--body FILE`: Use contents of FILE as PR body 
- `--body-text TEXT`: Use TEXT as PR body (supports escaped newlines with \n)
- `--base BRANCH`: Set the base branch to merge into (default: main)
- `--draft`: Create as draft PR
- `--no-generate`: Skip auto-generation of PR body
- `--help`: Display help message

### PR Body Auto-Generation

When auto-generating the PR body, the script:

1. Creates a "Changes in this PR" header
2. Lists all commits with their full details (including Prompt, Description, and Discussion sections)
3. Includes a summary of all changed files

### Requirements

- GitHub CLI (`gh`) must be installed and authenticated
- Git command-line tools

## Example: Current Branch PR

For the current `update-create-commit-script` branch, a good PR creation command would be:

```bash
./create-pr.sh --title "Add non-interactive commit and PR automation tools" --body-text "This PR adds automation for creating structured commits and pull requests in a non-interactive way, optimized for bot usage. It includes:\n\n- A new create-commit.sh script that accepts parameters via command line\n- A new create-pr.sh script for automated PR creation\n- Updates to Cursor rules for better clarity and organization\n\nBoth scripts are designed to work in non-interactive environments and follow the project's commit message format guidelines."
```

Or for automatic PR generation based on commit history:

```bash
./create-pr.sh
``` 