#!/bin/bash
# AI Assistant Helper Script for PR Creation
# This script helps AI assistants create structured PR descriptions
# without having to deal with multiline command issues

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-pr-helper.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE        - PR title (required)"
    echo "  --overview TEXT      - Brief overview of the PR (required)"
    echo "  --key-changes TEXT   - Comma-separated list of key changes"
    echo "  --implementation TEXT- Implementation details"
    echo "  --files TEXT         - Comma-separated list of files changed (optional, will auto-detect if omitted)"
    echo "  --commits TEXT       - Comma-separated list of commit descriptions (optional, will auto-detect if omitted)"
    echo "  --help               - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
PR_TITLE=""
OVERVIEW=""
KEY_CHANGES=""
IMPLEMENTATION=""
FILES=""
COMMITS=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            PR_TITLE="$2"
            shift 2
            ;;
        --overview)
            OVERVIEW="$2"
            shift 2
            ;;
        --key-changes)
            KEY_CHANGES="$2"
            shift 2
            ;;
        --implementation)
            IMPLEMENTATION="$2"
            shift 2
            ;;
        --files)
            FILES="$2"
            shift 2
            ;;
        --commits)
            COMMITS="$2"
            shift 2
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

# Check required parameters
if [ -z "$PR_TITLE" ]; then
    log_message "Error: PR title is required"
    show_usage
fi

if [ -z "$OVERVIEW" ]; then
    log_message "Error: Overview is required"
    show_usage
fi

# Create temp file for PR description
PR_BODY_FILE=$(mktemp)

# Start building the PR description
echo "📌 **Overview**:" > "$PR_BODY_FILE"
echo "$OVERVIEW" >> "$PR_BODY_FILE"
echo "" >> "$PR_BODY_FILE"

# Add key changes if provided
if [ -n "$KEY_CHANGES" ]; then
    echo "🔍 **Key Changes**:" >> "$PR_BODY_FILE"
    IFS=',' read -ra CHANGES <<< "$KEY_CHANGES"
    for change in "${CHANGES[@]}"; do
        echo "- $change" >> "$PR_BODY_FILE"
    done
    echo "" >> "$PR_BODY_FILE"
fi

# Add implementation details if provided
if [ -n "$IMPLEMENTATION" ]; then
    echo "🧩 **Implementation Details**:" >> "$PR_BODY_FILE"
    IFS=',' read -ra DETAILS <<< "$IMPLEMENTATION"
    for detail in "${DETAILS[@]}"; do
        echo "- $detail" >> "$PR_BODY_FILE"
    done
    echo "" >> "$PR_BODY_FILE"
fi

# Add files section
echo "📁 **Files Changed**:" >> "$PR_BODY_FILE"
if [ -n "$FILES" ]; then
    IFS=',' read -ra FILE_LIST <<< "$FILES"
    for file in "${FILE_LIST[@]}"; do
        echo "- $file" >> "$PR_BODY_FILE"
    done
else
    # Auto-detect changed files
    BASE_BRANCH="main"
    CURRENT_BRANCH=$(git branch --show-current | cat)
    
    git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
        if [[ -f "$file" ]]; then
            echo "- $file" >> "$PR_BODY_FILE"
        fi
    done
fi
echo "" >> "$PR_BODY_FILE"

# Add commits section
echo "📝 **Commit Details**:" >> "$PR_BODY_FILE"
echo "📅 $(date '+%B %Y')" >> "$PR_BODY_FILE"
if [ -n "$COMMITS" ]; then
    IFS=',' read -ra COMMIT_LIST <<< "$COMMITS"
    for commit in "${COMMIT_LIST[@]}"; do
        echo "✅ $commit" >> "$PR_BODY_FILE"
    done
else
    # Auto-detect commits
    BASE_BRANCH="main"
    CURRENT_BRANCH=$(git branch --show-current | cat)
    
    git log --reverse --pretty=format:"✅ %s" "$BASE_BRANCH..$CURRENT_BRANCH" >> "$PR_BODY_FILE"
fi

# Display the generated PR description
log_message "Generated PR description:"
log_message "---------------------------------------------"
cat "$PR_BODY_FILE"
log_message "---------------------------------------------"

# Create the PR using the main script
log_message "Creating PR with title: $PR_TITLE"
./scripts/create-pr.sh --title "$PR_TITLE" --body-file "$PR_BODY_FILE"

# Clean up
rm -f "$PR_BODY_FILE"
log_message "Temporary files cleaned up"
exit 0 