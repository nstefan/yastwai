#!/bin/bash
# AI Assistant Helper Script for PR Creation
# This script helps AI assistants create structured PR descriptions
# without having to deal with multiline command issues
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
    echo "  --base BRANCH        - Base branch to merge into (default: main)"
    echo "  --draft              - Create PR as draft (default: false)"
    echo "  --model MODEL        - Technical model name (required, e.g., claude-3-sonnet-20240229, not 'Claude 3 Sonnet')"
    echo "  --no-browser         - Don't open browser after PR creation (for testing/automation only)"
    echo "  --help               - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to find and extract sections from PR template
extract_section() {
    local template="$1"
    local section_marker="$2"
    local next_section_marker="$3"
    
    # Extract the section including marker
    if [[ -n "$next_section_marker" ]]; then
        echo "$template" | sed -n "/$section_marker/,/$next_section_marker/p" | sed '$d'
    else
        # If no next section marker, extract to end
        echo "$template" | sed -n "/$section_marker/,\$p"
    fi
}

# Default values
PR_TITLE=""
OVERVIEW=""
KEY_CHANGES=""
IMPLEMENTATION=""
FILES=""
DRAFT=false
MODEL=""
BASE_BRANCH="main"  # Set default base branch explicitly
OPEN_BROWSER=true   # Default to opening browser

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
        --no-browser)
            OPEN_BROWSER=false
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

# Create temp file for PR description
PR_BODY_FILE=$(mktemp)

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
    
    # Clean up backup file
    rm -f "${PR_BODY_FILE}.bak"
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

# Get current branch - add | cat to avoid pager
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null | cat)
if [ -z "$CURRENT_BRANCH" ]; then
    log_message "Error: Not on any branch"
    exit 1
fi

# Check for uncommitted changes - add | cat to avoid pager
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

# Get commit count - add | cat to avoid pager
# Explicitly check for empty base branch and provide fallback
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

# URL encode function that preserves newlines
url_encode() {
    python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=''))" <<< "$1"
}

# Get the GitHub repo URL
get_github_url() {
    REMOTE_URL=$(git config --get remote.origin.url 2>/dev/null | cat)
    
    if [[ "$REMOTE_URL" == git@github.com:* ]]; then
        REPO_PATH=${REMOTE_URL#git@github.com:}
        REPO_PATH=${REPO_PATH%.git}
        echo "https://github.com/$REPO_PATH"
    elif [[ "$REMOTE_URL" == https://github.com/* ]]; then
        echo "${REMOTE_URL%.git}"
    else
        echo "$REMOTE_URL"
    fi
}

# Get PR body content
PR_BODY_CONTENT=$(cat "$PR_BODY_FILE")

# Get the GitHub repository URL
REPO_URL=$(get_github_url)
if [ -z "$REPO_URL" ]; then
    log_message "Error: Could not determine GitHub repository URL"
    exit 1
fi

# Create the PR URL with properly encoded body
ENCODED_TITLE=$(url_encode "$PR_TITLE")
ENCODED_BODY=$(url_encode "$PR_BODY_CONTENT")
PR_URL="$REPO_URL/compare/$BASE_BRANCH...$CURRENT_BRANCH?quick_pull=1"
PR_URL="${PR_URL}&title=${ENCODED_TITLE}"
PR_URL="${PR_URL}&body=${ENCODED_BODY}"

if [ "$DRAFT" = true ]; then
    PR_URL="${PR_URL}&draft=1"
fi

log_message "Creating PR:"
log_message "Title: $PR_TITLE"
log_message "Base branch: $BASE_BRANCH"
log_message "Current branch: $CURRENT_BRANCH"
log_message ""
log_message "PR Description (you can copy this manually if needed):"
log_message "------------------------------------------------------"
echo -e "$PR_BODY_CONTENT"
log_message "------------------------------------------------------"
log_message ""
log_message "Pull request URL: $PR_URL"

# Only attempt to open the URL if we're not in a non-interactive environment and OPEN_BROWSER is true
if [ "$OPEN_BROWSER" = true ] && ([ -n "$DISPLAY" ] || [ "$(uname)" == "Darwin" ]); then
    # Open the PR URL in the default browser
    if command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$PR_URL" >/dev/null 2>&1 || log_message "Could not open browser automatically"
    elif command -v open >/dev/null 2>&1; then
        open "$PR_URL" >/dev/null 2>&1 || log_message "Could not open browser automatically"
    fi
fi

# Clean up
rm -f "$PR_BODY_FILE"
log_message "Temporary files cleaned up"
log_message "PR creation process completed successfully."
exit 0 
