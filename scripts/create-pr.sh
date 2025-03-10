#!/bin/bash
# Script to create a GitHub Pull Request based on commits in the current branch
# Bot-friendly version optimized for automated use by AI assistants
# Does not rely on GitHub CLI (gh) and handles escaped newlines properly

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/create-pr.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE       - PR title (optional, will be auto-generated if not provided)"
    echo "  --body TEXT         - PR body text with \\n for newlines (bot-friendly format)"
    echo "  --body-file FILE    - File containing PR body (alternative to --body)"
    echo "  --base BRANCH       - Base branch to merge into (default: main)"
    echo "  --draft             - Create PR as draft (default: false)"
    echo "  --template          - Use PR template from scripts/pr-template.md (default: false)"
    echo "  --compact           - Generate a compact PR body without template sections"
    echo "  --summary-only      - Include only the high-level summary and file list"
    echo "  --help              - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
BASE_BRANCH="main"
DRAFT=false
PR_TITLE=""
PR_BODY=""
PR_BODY_FILE=""
USE_TEMPLATE=false
COMPACT_MODE=false
SUMMARY_ONLY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            PR_TITLE="$2"
            shift 2
            ;;
        --body)
            PR_BODY="$2"
            shift 2
            ;;
        --body-file)
            PR_BODY_FILE="$2"
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
        --template)
            USE_TEMPLATE=true
            shift
            ;;
        --compact)
            COMPACT_MODE=true
            shift
            ;;
        --summary-only)
            SUMMARY_ONLY=true
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
    log_message "Error: Not on any branch"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    log_message "Error: You have uncommitted changes. Please commit or stash them before creating a PR."
    exit 1
fi

# Function to safely push changes
safe_push() {
    local attempts=0
    local max_attempts=3
    
    while [ $attempts -lt $max_attempts ]; do
        if git push -u origin "$CURRENT_BRANCH" 2>/dev/null; then
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
        
        if git pull --rebase origin "$CURRENT_BRANCH"; then
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
COMMIT_COUNT=$(git rev-list --count "$BASE_BRANCH..$CURRENT_BRANCH" | cat)
if [ "$COMMIT_COUNT" -eq 0 ]; then
    log_message "Error: No commits found between $BASE_BRANCH and $CURRENT_BRANCH"
    exit 1
fi

# Generate a meaningful PR title if not provided
if [ -z "$PR_TITLE" ]; then
    # Get all commit messages
    COMMIT_MSGS=$(git log --pretty=format:"%s" "$BASE_BRANCH..$CURRENT_BRANCH")
    
    # For a single commit, just use its message
    if [ "$COMMIT_COUNT" -eq 1 ]; then
        PR_TITLE=$(echo "$COMMIT_MSGS" | head -1)
    else
        # Extract key action verbs from commit messages
        ACTION_VERBS=$(echo "$COMMIT_MSGS" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' | sort | uniq | tr '\n' ',' | sed 's/,$//')
        
        # Get first and last commit messages
        FIRST_COMMIT=$(echo "$COMMIT_MSGS" | head -1)
        LAST_COMMIT=$(echo "$COMMIT_MSGS" | tail -1)
        
        # Create summary title
        if [[ "$FIRST_COMMIT" == "$LAST_COMMIT" ]]; then
            # Same message appears multiple times
            PR_TITLE="$FIRST_COMMIT"
        elif [[ -n "$ACTION_VERBS" && $(echo "$ACTION_VERBS" | tr -cd ',' | wc -c) -eq 0 ]]; then
            # Only one action verb across all commits
            MAIN_VERB=$(echo "$ACTION_VERBS" | tr ',' ' ')
            # Extract context from branch name
            CLEAN_BRANCH=$(echo "$CURRENT_BRANCH" | sed -E 's/(feature|fix|refactor|docs|chore)\///g' | sed 's/-/ /g' | sed 's/_/ /g')
            PR_TITLE="$MAIN_VERB $CLEAN_BRANCH"
        else
            # Multiple action types - create summary from first and last commit
            FIRST_ACTION=$(echo "$FIRST_COMMIT" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' || echo "Update")
            LAST_ACTION=$(echo "$LAST_COMMIT" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' || echo "improve")
            
            # Identify the main component being modified
            COMPONENT=$(git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | grep -o '^[^/]*' | sort | uniq -c | sort -nr | head -1 | awk '{print $2}')
            
            PR_TITLE="$FIRST_ACTION and $LAST_ACTION $COMPONENT"
        fi
    fi
    
    # Ensure title starts with capital letter and isn't too long
    PR_TITLE="$(echo "${PR_TITLE:0:1}" | tr '[:lower:]' '[:upper:]')${PR_TITLE:1}"
    if [ ${#PR_TITLE} -gt 60 ]; then
        PR_TITLE="${PR_TITLE:0:57}..."
    fi
fi

# Function to create a temporary file containing the PR body
create_pr_body() {
    local temp_file=$(mktemp)
    
    # If body is directly provided, use that with proper newline conversion
    if [ -n "$PR_BODY" ]; then
        # Convert escaped newlines to actual newlines
        echo -e "$PR_BODY" > "$temp_file"
        echo "$temp_file"
        return
    fi
    
    # If body file is provided, just use that file
    if [ -n "$PR_BODY_FILE" ] && [ -f "$PR_BODY_FILE" ]; then
        echo "$PR_BODY_FILE"
        return
    fi
    
    # If we get here, we need to generate a PR body
    
    # Start with template if requested (unless in compact or summary mode)
    if [ "$USE_TEMPLATE" = true ] && [ -f "scripts/pr-template.md" ] && [ "$COMPACT_MODE" = false ] && [ "$SUMMARY_ONLY" = false ]; then
        cp "scripts/pr-template.md" "$temp_file"
        # Replace the title placeholder with actual title
        sed -i.bak "s/\[PR Title\]/$PR_TITLE/" "$temp_file" && rm -f "$temp_file.bak"
        
        # Add auto-generated content after the template
        {
            echo ""
            echo "## 🤖 Auto-generated Content"
            echo ""
            echo "### 📁 Files Changed"
            git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
                if [[ -f "$file" ]]; then
                    echo "- \`$file\`"
                fi
            done
            echo ""
            echo "### ✅ Commits"
            git log --reverse --pretty=format:"- %s" "$BASE_BRANCH..$CURRENT_BRANCH" | head -10
            if [ "$COMMIT_COUNT" -gt 10 ]; then
                echo "- ... and $((COMMIT_COUNT - 10)) more commits"
            fi
        } >> "$temp_file"
    else
        # Generate a compact or regular summary
        {
            if [ "$SUMMARY_ONLY" = true ]; then
                # Very minimal summary
                echo "## 📋 Summary"
                echo ""
                echo "This PR modifies:"
                git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
                    if [[ -f "$file" ]]; then
                        echo "- \`$file\`"
                    fi
                done
            else
                # More detailed summary
                echo "## 📋 Summary"
                echo ""
                echo "This PR includes changes to:"
                echo ""
                echo "### 📁 Files Changed"
                git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
                    if [[ -f "$file" ]]; then
                        echo "- \`$file\`"
                    fi
                done
                echo ""
                echo "### ✅ Commits"
                git log --reverse --pretty=format:"- %s" "$BASE_BRANCH..$CURRENT_BRANCH" | head -10
                if [ "$COMMIT_COUNT" -gt 10 ]; then
                    echo "- ... and $((COMMIT_COUNT - 10)) more commits"
                fi
                
                if [ "$COMPACT_MODE" = false ]; then
                    # Add additional context for regular mode
                    echo ""
                    echo "### 📊 Changes Overview"
                    GIT_STATS=$(git diff --stat "$BASE_BRANCH..$CURRENT_BRANCH" | tail -1)
                    echo "- $GIT_STATS"
                fi
            fi
        } > "$temp_file"
    fi
    
    echo "$temp_file"
}

# Create the PR body
PR_BODY_FILE=$(create_pr_body)
PR_BODY_CONTENT=$(cat "$PR_BODY_FILE")

# Clean up temporary file if it was created by this script
if [[ "$PR_BODY_FILE" == /tmp/* ]]; then
    rm -f "$PR_BODY_FILE"
fi

# URL encode function that preserves newlines
url_encode() {
    python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=''))" <<< "$1"
}

# Get the GitHub repo URL
get_github_url() {
    REMOTE_URL=$(git config --get remote.origin.url | cat)
    
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

# Open the PR URL in the default browser
if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$PR_URL" >/dev/null 2>&1
elif command -v open >/dev/null 2>&1; then
    open "$PR_URL" >/dev/null 2>&1
else
    log_message "Pull request URL: $PR_URL"
fi

log_message "Pull request URL opened in browser."
log_message "PR creation process completed successfully."
exit 0 