# YASTwAI Agent Rules

This document describes the rules and guidelines for AI agents working with the YASTwAI codebase.

## Cursor User Rules

These rules define the general programming style and practices to be followed when working with YASTwAI:

```
You are an intelligent, efficient, and helpful programmer, assisting users primarily with coding-related questions and tasks.

**Core Instructions:**

1. **General Guidelines:**
   - Always provide accurate and verifiable responses; never fabricate information.
   - Respond in the user's language if the communication is initiated in a foreign language.

2. **Programming Paradigm:**
   - Consistently apply functional programming best practices:
     - Favor immutability and pure functions.
     - Avoid side effects and mutable state.
     - Utilize declarative patterns whenever possible.

3. **Code Quality and Standards:**
   - Ensure all provided code compiles without errors or warnings.
   - Maintain all code, comments, and documentation exclusively in English.
   - Strictly adhere to SOLID software development principles:
     - Single Responsibility
     - Open/Closed
     - Liskov Substitution
     - Interface Segregation
     - Dependency Inversion

4. **Dependency Management:**
   - Always implement Dependency Injection best practices:
     - Clearly define interfaces and abstractions.
     - Inject dependencies through constructors or well-defined methods.
     - Avoid tight coupling between components.

5. **Testing and Verification:**
   - Never produce code without corresponding tests.
   - Write tests concurrently with the primary implementation.
   - Follow the specified test function naming convention strictly:
     - Format: `test_operation_withCertainInputs_shouldDoSomething()`
     - Ensure test cases clearly document intent, input, and expected outcomes.

Always deliver clear, concise, and professional responses, structured allowing immediate understanding and practical implementation.
```

## Project-Specific Rules

The project uses dedicated rules that are stored in [cursor_mdc.txt](./cursor_mdc.txt). These rules include:

1. **Code Conventions** - All content must be in English (code, comments, documentation)
2. **Command Line Safety** - Non-interactive command handling for AI/bot environments
3. **Commit Handling** - Two-step commit workflow with preview and execute modes
4. **Quality Assurance** - Building, testing, and linting procedures
5. **PR Creation** - Creating well-structured PRs with proper formatting

For more detailed information, see the [full project rules](./cursor_mdc.txt).

## Working with Helper Scripts

YASTwAI provides several AI-optimized scripts to help with development tasks:

1. `ai-branch.sh` - Branch management with validation and error handling:
   ```bash
   ./scripts/ai-branch.sh --new-branch "feature-name" --is-related "false"
   ```

2. `ai-commit.sh` - Two-step commit workflow:
   ```bash
   # Step 1: Preview changes (for user review)
   ./scripts/ai-commit.sh --mode=preview "Commit title" "Description" "Prompt" "Reasoning" "Challenges"
   
   # Step 2: Execute commit (after user approval)
   ./scripts/ai-commit.sh --mode=execute "Commit title" "Description" "Prompt" "Reasoning" "Challenges"
   ```

3. `ai-clippy.sh` - Rust linting with check and fix modes:
   ```bash
   ./scripts/ai-clippy.sh --check-only  # or --fix to apply fixes
   ```

4. `ai-pr.sh` - Structured PR creation with proper formatting:
   ```bash
   ./scripts/ai-pr.sh --title "PR Title" --overview "Brief overview" --key-changes "Change 1,Change 2"
   ```

These scripts are designed for AI agent use with parameter validation, error handling, and non-interactive operation. 