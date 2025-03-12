# AI Agent Guide for YASTwAI

This guide is specifically designed for AI agents working with the YASTwAI codebase. It provides structured information about available scripts, rules, and best practices.

## Available Scripts

### 1. Branch Management (`ai-branch.sh`)
```bash
./scripts/ai-branch.sh [options]
```
**Options:**
- `--check-only` - Only check branch status
- `--new-branch <name>` - Create new branch
- `--is-related <true|false>` - Whether work is related to current branch

**Usage Pattern:**
```bash
# Check branch status
./scripts/ai-branch.sh --check-only

# Create new branch
./scripts/ai-branch.sh --new-branch "feature-name" --is-related false
```

### 2. Commit Management (`ai-commit.sh`)
```bash
./scripts/ai-commit.sh <title> <description> <prompt> <thought-process> <discussion>
```
**Required Parameters:**
- `title` - Concise summary
- `description` - Brief description
- `prompt` - Original user request
- `thought-process` - AI reasoning process
- `discussion` - Challenges and solutions

**Usage Pattern:**
```bash
# Create commit
./scripts/ai-commit.sh "Title" "Description" "Prompt" "Thought Process" "Discussion"
```

### 3. Pull Request Creation (`ai-pr.sh`)
```bash
./scripts/ai-pr.sh [options]
```
**Required Options:**
- `--title` - PR title
- `--overview` - Brief overview

**Optional Options:**
- `--key-changes` - Comma-separated list of changes
- `--implementation` - Comma-separated implementation details
- `--files` - Comma-separated changed files
- `--commits` - Comma-separated commit descriptions
- `--draft` - Create as draft PR

**Usage Pattern:**
```bash
./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2" --implementation "Detail 1,Detail 2"
```

### 4. Code Quality (`ai-clippy.sh`)
```bash
./scripts/ai-clippy.sh [options]
```
**Options:**
- `--check-only` - Run checks without fixing
- `--fix` - Attempt to fix issues automatically

**Usage Pattern:**
```bash
# Check for issues
./scripts/ai-clippy.sh --check-only

# Fix issues
./scripts/ai-clippy.sh --fix
```

## Rule Sets

### 1. Project Rules
- All code, comments, and documentation must be in English
- Commands must be non-interactive (use appropriate flags)
- Use `ai-cursor-model.sh` to detect current AI model

### 2. Commit Rules
- Never commit directly to main branch
- Always stage modified files with `git add`
- Create new branch if prompt is unrelated to current work
- Include complete thought process in commit messages

### 3. PR Rules
- Only create PRs when explicitly requested
- Use `ai-pr.sh` script for PR creation
- Structure PR descriptions with emojis:
  - 🧠 Instructions
  - 📌 Overview
  - 🔍 Key Changes
  - 🧩 Implementation Details
  - 🔄 Migration Notes
  - ⚠️ Areas of Attention
  - 📝 Commit Details
  - 📁 Files Changed

### 4. Rust Development Rules
- Follow functional programming principles
- Use async programming with tokio
- Implement proper error handling
- Write concurrent code safely
- Include tests for all new code
- Never use `#[allow(dead_code)]` except in tests

## Best Practices

1. **Branch Management**
   - Always check branch status before making changes
   - Create new branches for unrelated work
   - Branch names should be descriptive and follow conventions

2. **Code Changes**
   - Run clippy checks before committing
   - Ensure all tests pass
   - Update documentation as needed
   - Follow Rust's naming conventions

3. **Commits**
   - Use the commit script with clear, descriptive parameters
   - Include complete thought process
   - Reference related issues/PRs

4. **Pull Requests**
   - Create well-structured descriptions
   - Use emoji conventions consistently
   - Include all required sections

5. **Testing**
   - Write tests concurrently with implementation
   - Run tests in release mode
   - Fix code issues before modifying tests

## Error Handling

1. **Branch Errors**
   - If on main branch, create new feature branch
   - If work is unrelated, create new branch

2. **Commit Errors**
   - If commit fails, check and fix parameter formatting
   - Ensure all required parameters are provided correctly

3. **PR Errors**
   - If PR creation fails, verify all required parameters
   - If PR needs updates, use appropriate GitHub API calls

4. **Code Quality Errors**
   - Address clippy warnings before proceeding
   - Fix test failures by checking code first 