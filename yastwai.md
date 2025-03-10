# YASTwAI Rules and Guidelines

These rules provide guidance for the AI agent working with the YASTwAI codebase.

## Core Rules

1. Everything must be produced in english.
   - code
   - comments
   - documentation (README and other file headers)

2. Every command must not be interactive since we are dealing with a bot.
   - use proper arguments to prevent user interaction with commands.

3. Everytime the user sends a prompt, automatically commit changes at the end when you're confident everything is ok.
   - Check if we already are on a different branch than the "main" one. If not create a new branch.
   - Check if the prompt is still related to the current branch, if not create a new branch from the main one.
   - Use " | cat" to not stay stuck when checking status and branch names.
   - Use a concise summary as commit title
   - Use the full prompt made by the user as the commit description header as the "Prompt:" part.
   - Use the final summary as the rest of the commit description body as the "Description:" part.
   - Analyse all the faced difficulties and add a summary as the "Discussion:" part.
   - Use the ./scripts/create-commit.sh script to generate commits in the correct format.
   - Follow the detailed guidelines in CONTRIBUTING.md for reference.

4. Always finish by building the app and run unit tests in release mode.
   - Don't address warnings in test files as long as they run properly.
   - #[allow(dead_code)] directive is forbidden to fix warnings except to silence warnings related to tests in production code.

5. When a test breaks, be careful.
   - You must check first if it's the modification the problem before adapting the test to the new change.

6. For PR creation, ONLY create PRs when EXPLICITLY requested by the user.
   - NEVER create a PR automatically or as part of another task unless directly requested.
   - Use the scripts/create-pr.sh script ONLY when the user has asked to "create a PR" or similar explicit instructions.
   - Use the new bot-friendly format with the --body parameter to provide the complete PR description:
     ```bash
     ./scripts/create-pr.sh --body "## 📋 PR Overview\n\nThis PR implements feature X by...\n\n## 🔍 Implementation Details\n\n- Added new component\n- Fixed error handling" --template
     ```
   - Always use escaped newlines (\n) in the PR body for proper formatting
   - Include the --template flag for better PR structure
   - For larger PRs with complex changes, consider using the PR template

7. When a PR is explicitly requested, structure the PR body with emoji-enhanced sections:
   - 📌 **Overview**: Brief summary of what the PR accomplishes (2-3 sentences)
   - 🔍 **Key Changes**: Bullet points of the most significant changes
   - 🧩 **Implementation Details**: Technical approach and design decisions
   - 🔄 **Migration Notes**: Any changes requiring updates to existing code
   - ⚠️ **Areas of Attention**: Parts of the PR that need special review focus

8. Add appropriate emojis to PR sections for better readability:
   - 📝 Commit Details: Historical record of all commits
   - 📅 Date: For timestamps
   - ✅ Commit titles: For each commit
   - 📁 Files Changed: For listing modified files

9. Commit Message Structure Reference:
   ```
   <Concise summary as title>

   Prompt: <Original prompt or request>

   Description: <Detailed description of changes>

   Discussion: <Challenges faced and how they were resolved>
   ``` 