#!/bin/bash
# Script to create a GitHub Pull Request based on commits in the current branch
# Non-interactive version for automated use by bots
# This version does not rely on GitHub CLI (gh)

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
    echo "  --no-template       - Skip using PR template (default: false)"
    echo "  --help              - Display this help message"
    echo ""
    echo "If neither --body nor --body-text is provided, a PR body will be generated from commit messages."
    echo "By default, the PR template from scripts/pr-template.md will be used if available."
    exit 1
}

# Default values
BASE_BRANCH="main"
DRAFT=false
AUTO_GENERATE=true
USE_TEMPLATE=true
PR_TITLE=""
PR_BODY=""
PR_BODY_FILE=""
PR_TEMPLATE_PATH="scripts/pr-template.md"

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
        --no-template)
            USE_TEMPLATE=false
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
CURRENT_BRANCH=$(git branch --show-current | cat)
if [ -z "$CURRENT_BRANCH" ]; then
    echo "Error: Not on any branch"
    exit 1
fi

# Ensure we have commits
COMMIT_COUNT=$(git rev-list --count "$BASE_BRANCH..$CURRENT_BRANCH" | cat)
if [ "$COMMIT_COUNT" -eq 0 ]; then
    echo "Error: No commits found between $BASE_BRANCH and $CURRENT_BRANCH"
    exit 1
fi

# Generate PR title if not provided
if [ -z "$PR_TITLE" ]; then
    PR_TITLE=$(git log -1 --pretty=%s "$CURRENT_BRANCH" | cat)
fi

# Generate PR body if needed
if [ -z "$PR_BODY" ] && [ -z "$PR_BODY_FILE" ] && [ "$AUTO_GENERATE" = true ]; then
    TEMP_BODY=$(mktemp)
    
    # Start with PR template if available and requested
    if [ "$USE_TEMPLATE" = true ] && [ -f "$PR_TEMPLATE_PATH" ]; then
        cat "$PR_TEMPLATE_PATH" > "$TEMP_BODY"
        echo "" >> "$TEMP_BODY"
        echo "---" >> "$TEMP_BODY"
        echo "" >> "$TEMP_BODY"
    else
        # Add summary header
        echo "# Changes in this PR" > "$TEMP_BODY"
        echo "" >> "$TEMP_BODY"
    fi
    
    # List all commits with their messages and details
    echo "## Commit Details" >> "$TEMP_BODY"
    echo "" >> "$TEMP_BODY"
    
    git log --reverse --pretty=format:"### %s%n%n**Date:** %ad%n%n%b%n" "$BASE_BRANCH..$CURRENT_BRANCH" >> "$TEMP_BODY"
    
    # List changed files
    echo "## Files Changed" >> "$TEMP_BODY"
    echo "" >> "$TEMP_BODY"
    git diff --stat "$BASE_BRANCH..$CURRENT_BRANCH" >> "$TEMP_BODY"
    
    PR_BODY_FILE="$TEMP_BODY"
fi

# Read body from file if provided
if [ -n "$PR_BODY_FILE" ]; then
    PR_BODY=$(cat "$PR_BODY_FILE")
    
    # Clean up temporary file if we created one
    if [[ "$PR_BODY_FILE" == /tmp/* ]]; then
        rm "$PR_BODY_FILE"
    fi
fi

# Encode the PR body for URL
encode_url_param() {
    echo -n "$1" | perl -pe 's/([^A-Za-z0-9])/sprintf("%%%02X", ord($1))/ge'
}

# Get the GitHub repo URL
get_github_url() {
    # Get the remote URL
    REMOTE_URL=$(git config --get remote.origin.url | cat)
    
    # Remove .git suffix if present and convert SSH URL to HTTPS URL if needed
    if [[ "$REMOTE_URL" == git@github.com:* ]]; then
        # Convert from SSH format (git@github.com:user/repo.git) to HTTPS format
        REPO_PATH=${REMOTE_URL#git@github.com:}
        REPO_PATH=${REPO_PATH%.git}
        echo "https://github.com/$REPO_PATH"
    elif [[ "$REMOTE_URL" == https://github.com/* ]]; then
        # Already HTTPS format, just remove .git if present
        echo "${REMOTE_URL%.git}"
    else
        # Unknown format, return as is
        echo "$REMOTE_URL"
    fi
}

# Construct the PR URL
construct_pr_url() {
    local base_url="$1"
    local base_branch="$2"
    local head_branch="$3"
    local title="$4"
    local body="$5"
    local is_draft="$6"
    
    # Encode parameters for URL
    local encoded_title=$(encode_url_param "$title")
    local encoded_body=$(encode_url_param "$body")
    
    # Construct the URL
    local url="${base_url}/compare/${base_branch}...${head_branch}?quick_pull=1&title=${encoded_title}&body=${encoded_body}"
    
    # Add draft parameter if needed
    if [ "$is_draft" = true ]; then
        url="${url}&draft=1"
    fi
    
    echo "$url"
}

# Get the GitHub repository URL
REPO_URL=$(get_github_url)
if [ -z "$REPO_URL" ]; then
    echo "Error: Could not determine GitHub repository URL"
    exit 1
fi

echo "Creating PR:"
echo "Title: $PR_TITLE"
echo "Base branch: $BASE_BRANCH"
echo "Current branch: $CURRENT_BRANCH"
echo "Repository: $REPO_URL"
if [ "$USE_TEMPLATE" = true ] && [ -f "$PR_TEMPLATE_PATH" ]; then
    echo "Using PR template: $PR_TEMPLATE_PATH"
fi

# Construct the PR URL
PR_URL=$(construct_pr_url "$REPO_URL" "$BASE_BRANCH" "$CURRENT_BRANCH" "$PR_TITLE" "$PR_BODY" "$DRAFT")

# Check which browser opening command is available
if command -v open &> /dev/null; then
    # macOS
    open "$PR_URL"
elif command -v xdg-open &> /dev/null; then
    # Linux
    xdg-open "$PR_URL"
elif command -v start &> /dev/null; then
    # Windows
    start "$PR_URL"
else
    echo "Could not detect browser opening command. Please open this URL manually:"
    echo "$PR_URL"
    # Copy URL to clipboard if possible
    if command -v pbcopy &> /dev/null; then
        echo "$PR_URL" | pbcopy
        echo "URL copied to clipboard."
    elif command -v xclip &> /dev/null; then
        echo "$PR_URL" | xclip -selection clipboard
        echo "URL copied to clipboard."
    fi
fi

echo "Pull request URL opened in browser."
echo "URL: $PR_URL" 