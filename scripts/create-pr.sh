#!/bin/bash
# Script to create a GitHub Pull Request based on commits in the current branch
# Non-interactive version for automated use by bots
# This version does not rely on GitHub CLI (gh)

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/create-pr.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE       - PR title (optional, will be auto-generated if not provided)"
    echo "  --body-text TEXT    - PR body text (optional, will be auto-generated if not provided)"
    echo "  --body-file FILE    - File containing PR body (optional, overrides --body-text)"
    echo "  --base BRANCH       - Base branch to merge into (default: main)"
    echo "  --draft             - Create PR as draft (default: false)"
    echo "  --template          - Use PR template from scripts/pr-template.md (default: false)"
    echo "  --help              - Display this help message"
    exit 1
}

# Default values
BASE_BRANCH="main"
DRAFT=false
PR_TITLE=""
PR_BODY_TEXT=""
PR_BODY_FILE=""
USE_TEMPLATE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            PR_TITLE="$2"
            shift 2
            ;;
        --body-text)
            PR_BODY_TEXT="$2"
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

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: You have uncommitted changes. Please commit or stash them before creating a PR."
    exit 1
fi

# Check if there are unpushed changes that need to be pushed first
REMOTE_EXISTS=$(git ls-remote --heads origin $CURRENT_BRANCH 2>/dev/null | cat)
LOCAL_COMMIT=$(git rev-parse $CURRENT_BRANCH 2>/dev/null | cat)

if [ -z "$REMOTE_EXISTS" ]; then
    echo "Remote branch does not exist. Pushing changes..."
    if ! git push -u origin $CURRENT_BRANCH; then
        echo "Error: Failed to push to remote. Please check your connection and permissions."
        exit 1
    fi
    echo "Branch successfully pushed to remote."
else
    REMOTE_COMMIT=$(git ls-remote origin $CURRENT_BRANCH | awk '{print $1}' | cat)
    BEHIND_COUNT=$(git rev-list --count $CURRENT_BRANCH..origin/$CURRENT_BRANCH 2>/dev/null | cat)
    AHEAD_COUNT=$(git rev-list --count origin/$CURRENT_BRANCH..$CURRENT_BRANCH 2>/dev/null | cat)
    
    if [ "$BEHIND_COUNT" -gt 0 ]; then
        echo "Error: Your branch is behind the remote by $BEHIND_COUNT commit(s)."
        echo "Please pull the latest changes with 'git pull' before creating a PR."
        exit 1
    fi
    
    if [ "$AHEAD_COUNT" -gt 0 ]; then
        echo "Your branch is ahead of remote by $AHEAD_COUNT commit(s). Pushing changes..."
        if ! git push origin $CURRENT_BRANCH; then
            echo "Error: Failed to push to remote. Please check your connection and permissions."
            exit 1
        fi
        echo "Changes successfully pushed to remote."
    else
        echo "Branch is up to date with remote. No need to push."
    fi
fi

# Get commit count
COMMIT_COUNT=$(git rev-list --count "$BASE_BRANCH..$CURRENT_BRANCH" | cat)
if [ "$COMMIT_COUNT" -eq 0 ]; then
    echo "Error: No commits found between $BASE_BRANCH and $CURRENT_BRANCH"
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
        # Extract common file types that were modified
        FILE_EXTENSIONS=$(git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | grep -o '\.[^/.]*$' | sort | uniq | tr -d '.' | tr '\n' ',' | sed 's/,$//')
        
        # Extract key action verbs from commit messages
        ACTION_VERBS=$(echo "$COMMIT_MSGS" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' | sort | uniq | tr '\n' ',' | sed 's/,$//')
        
        # Get first and last commit messages
        FIRST_COMMIT=$(echo "$COMMIT_MSGS" | head -1)
        LAST_COMMIT=$(echo "$COMMIT_MSGS" | tail -1)
        
        # Create summary title based on number of commits and patterns
        if [[ "$FIRST_COMMIT" == "$LAST_COMMIT" ]]; then
            # Same message appears multiple times
            PR_TITLE="$FIRST_COMMIT"
        elif [[ -n "$ACTION_VERBS" && $(echo "$ACTION_VERBS" | tr -cd ',' | wc -c) -eq 0 ]]; then
            # Only one action verb across all commits
            MAIN_VERB=$(echo "$ACTION_VERBS" | tr ',' ' ')
            if [[ "$FILE_EXTENSIONS" == "sh" || "$COMMIT_MSGS" == *"script"* ]]; then
                PR_TITLE="$MAIN_VERB scripts for automation"
            else
                # Use branch name to add context but with the correct verb
                CLEAN_BRANCH=$(echo "$CURRENT_BRANCH" | sed 's/-/ /g' | sed 's/_/ /g')
                CONTEXT=$(echo "$CLEAN_BRANCH" | awk '{for(i=2;i<=NF;i++) printf "%s ", $i}')
                PR_TITLE="$MAIN_VERB $CONTEXT"
            fi
        else
            # Multiple action types - create summary from first and last commit
            FIRST_ACTION=$(echo "$FIRST_COMMIT" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' || echo "Update")
            LAST_ACTION=$(echo "$LAST_COMMIT" | grep -o -E '^(Add|Update|Fix|Remove|Refactor|Improve|Implement|Create|Move|Rename|Delete)' || echo "cleanup")
            
            # Extract main component being modified
            if [[ "$COMMIT_MSGS" == *"commit script"* || "$COMMIT_MSGS" == *"create-commit"* ]]; then
                COMPONENT="commit tooling"
            elif [[ "$COMMIT_MSGS" == *"PR"* || "$COMMIT_MSGS" == *"pull request"* || "$COMMIT_MSGS" == *"create-pr"* ]]; then
                COMPONENT="PR workflow"
            elif [[ "$COMMIT_MSGS" == *".github"* ]]; then
                COMPONENT="GitHub configuration"
            else
                # Fall back to using directory most frequently changed
                COMPONENT=$(git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | grep -o '^[^/]*' | sort | uniq -c | sort -nr | head -1 | awk '{print $2}')
            fi
            
            PR_TITLE="$FIRST_ACTION and $LAST_ACTION $COMPONENT"
        fi
        
        # Ensure title starts with capital letter
        PR_TITLE="$(echo "${PR_TITLE:0:1}" | tr '[:lower:]' '[:upper:]')${PR_TITLE:1}"
        
        # Ensure title isn't too long
        if [ ${#PR_TITLE} -gt 60 ]; then
            PR_TITLE="${PR_TITLE:0:57}..."
        fi
    fi
fi

# Generate a PR body that actually summarizes changes
if [ -z "$PR_BODY_FILE" ] && [ -z "$PR_BODY_TEXT" ]; then
    # Create a temporary file for the PR body
    PR_BODY_FILE=$(mktemp)
    
    if [ "$USE_TEMPLATE" = true ] && [ -f "scripts/pr-template.md" ]; then
        # Start with the template
        cp scripts/pr-template.md "$PR_BODY_FILE"
        
        # Replace the title placeholder with actual title
        sed -i '' "s/\[PR Title\]/$PR_TITLE/" "$PR_BODY_FILE"
        
        # Add auto-generated content after the template
        {
            echo ""
            echo "## Auto-generated Content"
            echo ""
            echo "### Files Changed"
            git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
                if [[ -f "$file" ]]; then
                    echo "- \`$file\`"
                fi
            done
            echo ""
            echo "### Commits"
            git log --reverse --pretty=format:"- %s" "$BASE_BRANCH..$CURRENT_BRANCH" | head -10
            if [ "$COMMIT_COUNT" -gt 10 ]; then
                echo "- ... and $((COMMIT_COUNT - 10)) more commits"
            fi
        } >> "$PR_BODY_FILE"
    else
        # Generate a useful summary without template
        {
            echo "## Summary"
            echo ""
            echo "This PR includes changes to:"
            echo ""
            echo "### Files Changed"
            git diff --name-only "$BASE_BRANCH..$CURRENT_BRANCH" | sort | uniq | while read -r file; do
                if [[ -f "$file" ]]; then
                    echo "- \`$file\`"
                fi
            done
            echo ""
            echo "### Commits"
            git log --reverse --pretty=format:"- %s" "$BASE_BRANCH..$CURRENT_BRANCH" | head -10
            if [ "$COMMIT_COUNT" -gt 10 ]; then
                echo "- ... and $((COMMIT_COUNT - 10)) more commits"
            fi
        } > "$PR_BODY_FILE"
    fi
elif [ -n "$PR_BODY_TEXT" ] && [ -z "$PR_BODY_FILE" ]; then
    # Create a temporary file with the provided body text
    PR_BODY_FILE=$(mktemp)
    echo "$PR_BODY_TEXT" > "$PR_BODY_FILE"
fi

# URL encode a string for use in a URL
url_encode() {
    # Use Python for more reliable URL encoding that preserves newlines
    python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.stdin.read(), safe=''))" <<< "$1"
}

# Format PR body with proper newlines
format_pr_body() {
    local body="$1"
    # First, replace escaped newlines with actual newlines
    echo -e "$body" | sed 's/ | /\n\n/g'
}

# Read the PR body
if [ -n "$PR_BODY_FILE" ]; then
    PR_BODY=$(cat "$PR_BODY_FILE")
elif [ -n "$PR_BODY_TEXT" ]; then
    # Create a temporary file with the provided body text, preserving newlines
    PR_BODY_FILE=$(mktemp)
    echo -e "$PR_BODY_TEXT" > "$PR_BODY_FILE"
    PR_BODY=$(cat "$PR_BODY_FILE")
else
    PR_BODY=$(format_pr_body "$PR_BODY")
fi

# Clean up temporary file if we created one
if [[ "$PR_BODY_FILE" == /tmp/* ]]; then
    rm "$PR_BODY_FILE"
fi

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
    echo "Error: Could not determine GitHub repository URL"
    exit 1
fi

# Create the PR URL with properly encoded body
ENCODED_TITLE=$(url_encode "$PR_TITLE")
ENCODED_BODY=$(url_encode "$PR_BODY")
PR_URL="$REPO_URL/compare/$BASE_BRANCH...$CURRENT_BRANCH?quick_pull=1"
PR_URL="${PR_URL}&title=${ENCODED_TITLE}"
PR_URL="${PR_URL}&body=${ENCODED_BODY}"

if [ "$DRAFT" = true ]; then
    PR_URL="${PR_URL}&draft=1"
fi

echo "Creating PR:"
echo "Title: $PR_TITLE"
echo "Base branch: $BASE_BRANCH"
echo "Current branch: $CURRENT_BRANCH"
echo ""
echo "PR Description (you can copy this manually if needed):"
echo "------------------------------------------------------"
echo -e "$PR_BODY"
echo "------------------------------------------------------"
echo ""

# Open the PR URL in the default browser
if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$PR_URL" >/dev/null 2>&1
elif command -v open >/dev/null 2>&1; then
    open "$PR_URL" >/dev/null 2>&1
else
    echo "Pull request URL: $PR_URL"
fi

echo "Pull request URL opened in browser."
exit 0 