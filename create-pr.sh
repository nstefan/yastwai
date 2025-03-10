#!/bin/bash
# Script to create a GitHub Pull Request based on commits in the current branch
# Non-interactive version for automated use by bots

# Function to show usage
show_usage() {
    echo "Usage: ./create-pr.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE       - PR title (optional, will use first commit message if not provided)"
    echo "  --body FILE         - File containing PR body (optional)"
    echo "  --body-text TEXT    - PR body text (optional, can use escaped newlines)"
    echo "  --base BRANCH       - Base branch to merge into (default: main)"
    echo "  --draft             - Create PR as draft (default: false)"
    echo "  --no-generate       - Skip auto-generation of PR body (default: false)"
    echo "  --help              - Display this help message"
    echo ""
    echo "If neither --body nor --body-text is provided, a PR body will be generated from commit messages."
    exit 1
}

# Default values
BASE_BRANCH="main"
DRAFT=false
AUTO_GENERATE=true
PR_TITLE=""
PR_BODY=""
PR_BODY_FILE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            PR_TITLE="$2"
            shift 2
            ;;
        --body)
            PR_BODY_FILE="$2"
            shift 2
            ;;
        --body-text)
            PR_BODY=$(echo -e "$2")
            shift 2
            ;;
        --base)
            BASE_BRANCH="$2"
            shift 2
            ;;
        --draft)
            DRAFT=true
            shift
            ;;
        --no-generate)
            AUTO_GENERATE=false
            shift
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Get current branch
CURRENT_BRANCH=$(git branch --show-current)
if [ -z "$CURRENT_BRANCH" ]; then
    echo "Error: Not on any branch"
    exit 1
fi

# Ensure we have commits
COMMIT_COUNT=$(git rev-list --count "$BASE_BRANCH..$CURRENT_BRANCH")
if [ "$COMMIT_COUNT" -eq 0 ]; then
    echo "Error: No commits found between $BASE_BRANCH and $CURRENT_BRANCH"
    exit 1
fi

# Generate PR title if not provided
if [ -z "$PR_TITLE" ]; then
    PR_TITLE=$(git log -1 --pretty=%s "$CURRENT_BRANCH")
fi

# Generate PR body if needed
if [ -z "$PR_BODY" ] && [ -z "$PR_BODY_FILE" ] && [ "$AUTO_GENERATE" = true ]; then
    TEMP_BODY_FILE=$(mktemp)
    
    # Add summary header
    echo "# Changes in this PR" > "$TEMP_BODY_FILE"
    echo "" >> "$TEMP_BODY_FILE"
    
    # List all commits with their messages and details
    echo "## Commit Details" >> "$TEMP_BODY_FILE"
    echo "" >> "$TEMP_BODY_FILE"
    
    git log --reverse --pretty=format:"### %s%n%n**Date:** %ad%n%n%b%n" "$BASE_BRANCH..$CURRENT_BRANCH" >> "$TEMP_BODY_FILE"
    
    # List changed files
    echo "## Files Changed" >> "$TEMP_BODY_FILE"
    echo "" >> "$TEMP_BODY_FILE"
    git diff --stat "$BASE_BRANCH..$CURRENT_BRANCH" >> "$TEMP_BODY_FILE"
    
    PR_BODY_FILE="$TEMP_BODY_FILE"
fi

# Read body from file if provided
if [ -n "$PR_BODY_FILE" ]; then
    PR_BODY=$(cat "$PR_BODY_FILE")
    
    # Clean up temporary file if we created one
    if [ -n "$TEMP_BODY_FILE" ] && [ "$PR_BODY_FILE" = "$TEMP_BODY_FILE" ]; then
        rm "$TEMP_BODY_FILE"
    fi
fi

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed. Please install it first."
    echo "See: https://github.com/cli/cli#installation"
    exit 1
fi

# Create the PR
echo "Creating PR:"
echo "Title: $PR_TITLE"
echo "Base branch: $BASE_BRANCH"
echo "Current branch: $CURRENT_BRANCH"

# Construct the command
PR_CMD="gh pr create --base \"$BASE_BRANCH\" --head \"$CURRENT_BRANCH\" --title \"$PR_TITLE\""

if [ -n "$PR_BODY" ]; then
    # Save body to a temporary file for the command
    BODY_FILE=$(mktemp)
    echo "$PR_BODY" > "$BODY_FILE"
    PR_CMD="$PR_CMD --body-file \"$BODY_FILE\""
fi

if [ "$DRAFT" = true ]; then
    PR_CMD="$PR_CMD --draft"
fi

# Execute the command
eval "$PR_CMD"
RESULT=$?

# Clean up
if [ -n "$BODY_FILE" ]; then
    rm "$BODY_FILE"
fi

if [ $RESULT -eq 0 ]; then
    echo "Pull request created successfully!"
else
    echo "Failed to create pull request. Error code: $RESULT"
    exit $RESULT
fi 