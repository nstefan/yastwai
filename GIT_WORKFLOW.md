# YASTwAI Git Workflow

This document explains the Git workflow for the YASTwAI project, including how to use the helper scripts to follow the yastai.mdc ruleset.

## Workflow Overview

The YASTwAI project follows a branch-based workflow:

1. Development happens on feature branches, not directly on `main`
2. Each feature or fix gets its own branch
3. Commits follow a specific format (described in `COMMIT_GUIDELINES.md`)
4. After development is complete, changes are merged back to `main`

## Helper Scripts

Two scripts are provided to help with this workflow:

### 1. Branch Management (`branch-check.sh`)

This script helps you manage branches according to the yastai.mdc requirements.

```bash
./scripts/branch-check.sh
```

The script will:
- Check if you're on the `main` branch
- Prompt you to create a new branch if needed
- Check if your new work is related to the current branch
- Guide you through switching to a new branch when appropriate

**When to use**: At the beginning of working on a new feature or fix.

### 2. Commit Creation (`create-commit.sh`)

This script helps you create commits with the proper format.

```bash
./scripts/create-commit.sh "Commit title" "Original prompt"
```

The script will:
- Create a template commit message with the required sections
- Open your text editor to complete the message
- Preview the message before committing
- Stage changes and create the commit

**When to use**: When you're ready to commit your changes.

## Standard Workflow Example

Here's an example of a standard workflow:

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

## Commit Message Format

Remember that all commits should follow this format:

```
<Concise summary as title>

Prompt: <Original prompt or request>

Description: <Detailed description of changes>

Discussion: <Challenges faced and how they were resolved>
```

See `COMMIT_GUIDELINES.md` for detailed information about each section.

## Special Considerations

- All work should be done in English
- Follow Rust best practices and coding standards
- When fixing tests, always check if the code is the problem before modifying the test
- Never use `#[allow(dead_code)]` directives except for test-related code

For more detailed information, refer to the yastai.mdc ruleset and the `COMMIT_GUIDELINES.md` document. 