# Commit Message Guidelines for YASTwAI

This document outlines the commit message format and best practices for the YASTwAI project, following the standards defined in the `yastai.mdc` ruleset.

## Commit Message Structure

Each commit message should follow this structure:

```
<Concise summary as title>

Prompt: <Original prompt or request>

Description: <Detailed description of changes>

Discussion: <Challenges faced and how they were resolved>
```

### Components

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

## Example Commit Message

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

## Using the Commit Script

A helper script `create-commit.sh` is provided to streamline the commit message creation process:

```bash
./create-commit.sh "Commit title" "Original prompt"
```

The script will:
1. Create a template commit message
2. Open your default editor to complete the message
3. Show a preview of the final message
4. Stage changes and create the commit

## Branching Guidelines

When working on a new feature or fix:

1. Check if you're already on a feature branch (other than "main")
2. If working on a new feature, create a new branch from main
3. Ensure the branch name reflects the purpose of the changes
4. When ready, commit your changes following the message format outlined above

## Final Steps

After committing changes:

1. Build the application in release mode
2. Run unit tests to ensure everything works correctly
3. Fix any issues identified during testing
4. Create a new commit for any fixes needed

## Special Considerations

- Keep all code, comments, and documentation in English
- Never use `#[allow(dead_code)]` directives except for test-related code
- When a test breaks, first check if the code change is problematic before modifying the test 