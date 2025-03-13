#!/bin/bash
# AI Assistant Helper Script for PR Creation
# This script helps AI assistants create structured PR descriptions using GitHub CLI
# Follows the naming pattern of ai-*.sh for consistency

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-pr.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE        - PR title (required)"
    echo "  --overview TEXT      - Brief overview of the PR (required)"
    echo "  --key-changes TEXT   - Comma-separated list of key changes"
    echo "  --implementation TEXT - Comma-separated list of implementation details"
    echo "  --base BRANCH        - Base branch to merge into (default: main)"
    echo "  --draft              - Create PR as draft (default: false)"
    echo "  --model MODEL        - Technical model name (required, e.g., claude-3-sonnet-20240229, not 'Claude 3 Sonnet')"
    echo "  --help               - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Check if gh CLI is installed
check_gh_cli() {
    if ! command -v gh &> /dev/null; then
        log_message "Error: GitHub CLI (gh) is not installed. Please install it first."
        log_message "Installation instructions: https://github.com/cli/cli#installation"
        exit 1
    fi
    
    # Check if gh is authenticated
    if ! gh auth status &> /dev/null; then
        log_message "Error: GitHub CLI (gh) is not authenticated. Please run 'gh auth login' first."
        exit 1
    fi
}

# Default values
PR_TITLE=""
OVERVIEW=""
KEY_CHANGES=""
IMPLEMENTATION=""
DRAFT=false
MODEL=""
BASE_BRANCH="main"  # Set default base branch explicitly

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --title requires a value"
                show_usage
            fi
            PR_TITLE="$2"
            shift 2
            ;;
        --overview)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --overview requires a value"
                show_usage
            fi
            OVERVIEW="$2"
            shift 2
            ;;
        --key-changes)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --key-changes requires a value"
                show_usage
            fi
            KEY_CHANGES="$2"
            shift 2
            ;;
        --implementation)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --implementation requires a value"
                show_usage
            fi
            IMPLEMENTATION="$2"
            shift 2
            ;;
        --base)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --base requires a value"
                show_usage
            fi
            BASE_BRANCH="$2"
            shift 2
            ;;
        --model)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --model requires a value"
                show_usage
            fi
            MODEL="$2"
            shift 2
            ;;
        --draft)
            DRAFT=true
            shift
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            log_message "Unknown option: $1"
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

if [ -z "$MODEL" ]; then
    log_message "Error: Model parameter is required"
    show_usage
fi

# Ensure MODEL doesn't have quotes that could break the script
MODEL=$(echo "$MODEL" | tr -d '"'"'")

# Check if GitHub CLI is installed and authenticated
check_gh_cli

# Create temp file for PR description
PR_BODY_FILE=$(mktemp)

# Check for PR template location
PR_TEMPLATE_PATHS=(
    "./.github/PULL_REQUEST_TEMPLATE.md"
    "./.github/pull_request_template.md"
)

PR_TEMPLATE=""
for template_path in "${PR_TEMPLATE_PATHS[@]}"; do
    if [ -f "$template_path" ]; then
        PR_TEMPLATE=$(cat "$template_path")
        log_message "Found PR template at $template_path"
        break
    fi
done

if [ -n "$PR_TEMPLATE" ]; then
    # Use PR template as base
    echo "$PR_TEMPLATE" > "$PR_BODY_FILE"
    
    # Find section markers in the template
    OVERVIEW_MARKER="## 📌 Overview"
    KEY_CHANGES_MARKER="## 🔍 Key Changes"
    IMPLEMENTATION_MARKER="## 🧩 Implementation Details"
    TESTING_MARKER="## 🧪 Testing"
    CHECKLIST_MARKER="## 🔎 Checklist"
    AI_MODEL_MARKER="## 🤖 AI Model"
    
    # Create a simplified PR description based on the template
    # Start with a clean description
    PR_DESCRIPTION=""
    
    # Add overview (always included)
    PR_DESCRIPTION+="$OVERVIEW_MARKER\n"
    PR_DESCRIPTION+="$OVERVIEW\n\n"
    
    # Add key changes (always included)
    PR_DESCRIPTION+="$KEY_CHANGES_MARKER\n"
    if [ -n "$KEY_CHANGES" ]; then
        IFS=',' read -ra CHANGES <<< "$KEY_CHANGES"
        for change in "${CHANGES[@]}"; do
            PR_DESCRIPTION+="- $change\n"
        done
        PR_DESCRIPTION+="\n"
    else
        PR_DESCRIPTION+="<!-- No key changes specified -->\n\n"
    fi
    
    # Add implementation details (only if provided)
    if [ -n "$IMPLEMENTATION" ]; then
        PR_DESCRIPTION+="$IMPLEMENTATION_MARKER\n"
        IFS=',' read -ra DETAILS <<< "$IMPLEMENTATION"
        for detail in "${DETAILS[@]}"; do
            PR_DESCRIPTION+="- $detail\n"
        done
        PR_DESCRIPTION+="\n"
    fi
    
    # Always include AI Model section (last)
    PR_DESCRIPTION+="$AI_MODEL_MARKER\n"
    PR_DESCRIPTION+="$MODEL\n"
    
    # Write the final PR description to the file
    echo -e "$PR_DESCRIPTION" > "$PR_BODY_FILE"
else
    # Fallback: manually construct PR description if template not found
    log_message "No PR template found, constructing default format"
    
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
    
    # Add AI model information at the end
    echo "🤖 **AI Model**: $MODEL" >> "$PR_BODY_FILE"
fi

# Display the generated PR description
log_message "Generated PR description:"
log_message "---------------------------------------------"
cat "$PR_BODY_FILE"
log_message "---------------------------------------------"

# Get current branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null | cat)
if [ -z "$CURRENT_BRANCH" ]; then
    log_message "Error: Not on any branch"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain 2>/dev/null | cat)" ]; then
    log_message "Error: You have uncommitted changes. Please commit or stash them before creating a PR."
    exit 1
fi

# Log branch information
log_message "Current branch: $CURRENT_BRANCH"
log_message "Base branch: $BASE_BRANCH"

# Function to safely push changes
safe_push() {
    local attempts=0
    local max_attempts=3
    
    while [ $attempts -lt $max_attempts ]; do
        if git push -u origin "$CURRENT_BRANCH" 2>/dev/null | cat; then
            log_message "Branch successfully pushed to remote."
            return 0
        else
            attempts=$((attempts + 1))
            if [ $attempts -lt $max_attempts ]; then
                log_message "Push failed. Retrying in 2 seconds... (Attempt $attempts/$max_attempts)"
                sleep 2
            else
                log_message "Error: Failed to push to remote after $max_attempts attempts."
                return 1
            fi
        fi
    done
    return 1
}

# Check remote branch status and push if needed
REMOTE_EXISTS=$(git ls-remote --heads origin "$CURRENT_BRANCH" 2>/dev/null | cat)

if [ -z "$REMOTE_EXISTS" ]; then
    log_message "Remote branch does not exist. Pushing changes..."
    if ! safe_push; then
        exit 1
    fi
else
    BEHIND_COUNT=$(git rev-list --count "$CURRENT_BRANCH..origin/$CURRENT_BRANCH" 2>/dev/null | cat)
    AHEAD_COUNT=$(git rev-list --count "origin/$CURRENT_BRANCH..$CURRENT_BRANCH" 2>/dev/null | cat)
    
    if [ "$BEHIND_COUNT" -gt 0 ]; then
        log_message "Your branch is behind the remote by $BEHIND_COUNT commit(s)."
        log_message "Attempting to rebase automatically..."
        
        if git pull --rebase origin "$CURRENT_BRANCH" 2>/dev/null | cat; then
            log_message "Successfully rebased against remote branch."
        else
            log_message "Error: Automatic rebase failed. Please resolve conflicts manually."
            exit 1
        fi
    fi
    
    if [ "$AHEAD_COUNT" -gt 0 ]; then
        log_message "Your branch is ahead of remote by $AHEAD_COUNT commit(s). Pushing changes..."
        if ! safe_push; then
            exit 1
        fi
    else
        log_message "Branch is up to date with remote. No need to push."
    fi
fi

# Get commit count
if [ -z "$BASE_BRANCH" ]; then
    BASE_BRANCH="main"
    log_message "Base branch was empty, using default: $BASE_BRANCH"
fi

COMMIT_COUNT=$(git rev-list --count "${BASE_BRANCH}..${CURRENT_BRANCH}" 2>/dev/null | cat)
log_message "Found $COMMIT_COUNT commits between $BASE_BRANCH and $CURRENT_BRANCH"

if [ -z "$COMMIT_COUNT" ] || [ "$COMMIT_COUNT" -eq 0 ]; then
    log_message "Warning: No commits found between $BASE_BRANCH and $CURRENT_BRANCH"
    log_message "This might be because:"
    log_message "1. Your branch has no commits"
    log_message "2. Your branch is not based off $BASE_BRANCH"
    log_message "3. Some other issue with git history"
    log_message "Continuing anyway, but the PR may be empty..."
fi

# Create the PR using GitHub CLI
log_message "Creating PR using GitHub CLI (gh)..."

# Build the gh command with all required options
GH_CMD="gh pr create --title \"$PR_TITLE\" --body-file \"$PR_BODY_FILE\" --base \"$BASE_BRANCH\""

# Add draft flag if needed
if [ "$DRAFT" = true ]; then
    GH_CMD="$GH_CMD --draft"
fi

# Execute the command
log_message "Executing: $GH_CMD"
PR_URL=$(eval "$GH_CMD")
PR_EXIT_CODE=$?

if [ $PR_EXIT_CODE -ne 0 ]; then
    log_message "Error: Failed to create pull request"
    log_message "GitHub CLI command failed with exit code: $PR_EXIT_CODE"
    exit 1
fi

log_message "Successfully created PR: $PR_URL"

# Clean up
rm -f "$PR_BODY_FILE"
log_message "Temporary files cleaned up"
log_message "PR creation process completed successfully."
exit 0 
