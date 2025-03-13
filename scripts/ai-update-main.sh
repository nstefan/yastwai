#!/bin/bash
# AI Assistant Helper Script for Updating Main Branch
# This script helps AI assistants fetch and update the main branch without requiring interactive input
# Follows the naming pattern of ai-*.sh for consistency

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-update-main.sh [options]"
    echo ""
    echo "Options:"
    echo "  --check-only      - Only check for updates without applying them"
    echo "  --rebase-current  - Also rebase current branch onto updated main"
    echo "  --help            - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
CHECK_ONLY=false
REBASE_CURRENT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --rebase-current)
            REBASE_CURRENT=true
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

# Get the current branch and main branch (always use cat to prevent pager)
CURRENT_BRANCH=$(git branch --show-current | cat)
MAIN_BRANCH="main"

log_message "Current branch: $CURRENT_BRANCH"

# Fetch all changes from remote
log_message "Fetching latest changes from remote..."
git fetch --all | cat

# Check for updates to main branch
LOCAL_MAIN_REV=$(git rev-parse $MAIN_BRANCH 2>/dev/null | cat)
REMOTE_MAIN_REV=$(git rev-parse origin/$MAIN_BRANCH 2>/dev/null | cat)

if [ "$LOCAL_MAIN_REV" = "$REMOTE_MAIN_REV" ]; then
    log_message "Main branch is already up to date with origin."
    
    if [ "$CHECK_ONLY" = true ]; then
        exit 0
    fi
else
    log_message "Updates available for main branch."
    
    if [ "$CHECK_ONLY" = true ]; then
        # Show what changes are available
        log_message "New commits available (showing last 5):"
        git log --oneline -n 5 $LOCAL_MAIN_REV..origin/$MAIN_BRANCH | cat
        exit 0
    fi
    
    # Need to update main
    log_message "Updating main branch..."
    
    # Check if there are uncommitted changes
    if [ -n "$(git status --porcelain | cat)" ]; then
        log_message "Error: You have uncommitted changes. Please commit or stash them before updating."
        exit 1
    fi
    
    # Remember current branch to return to it
    RETURN_TO_BRANCH=$CURRENT_BRANCH
    
    # Switch to main branch
    if [ "$CURRENT_BRANCH" != "$MAIN_BRANCH" ]; then
        log_message "Switching to $MAIN_BRANCH branch..."
        git checkout $MAIN_BRANCH 2>/dev/null || { log_message "Error: Failed to switch to branch '$MAIN_BRANCH'"; exit 1; }
    fi
    
    # Update main branch
    log_message "Pulling latest changes with rebase..."
    git pull --rebase origin $MAIN_BRANCH 2>&1 | cat
    if [ ${PIPESTATUS[0]} -ne 0 ]; then
        log_message "Error: Failed to update main branch. There might be conflicts."
        log_message "Please resolve manually and run this script again."
        exit 1
    fi
    
    log_message "Main branch successfully updated!"
    
    # If we need to rebase current branch as well
    if [ "$REBASE_CURRENT" = true ] && [ "$RETURN_TO_BRANCH" != "$MAIN_BRANCH" ]; then
        log_message "Rebasing $RETURN_TO_BRANCH onto updated $MAIN_BRANCH..."
        
        # Switch back to original branch
        git checkout $RETURN_TO_BRANCH 2>/dev/null || { log_message "Error: Failed to switch back to branch '$RETURN_TO_BRANCH'"; exit 1; }
        
        # Rebase onto updated main
        git rebase $MAIN_BRANCH 2>&1 | cat
        if [ ${PIPESTATUS[0]} -ne 0 ]; then
            log_message "Error: Rebase conflicts detected. Please resolve manually."
            log_message "After resolving conflicts, run: git rebase --continue"
            exit 1
        fi
        
        log_message "Successfully rebased $RETURN_TO_BRANCH onto updated $MAIN_BRANCH!"
    elif [ "$RETURN_TO_BRANCH" != "$MAIN_BRANCH" ]; then
        # Just switch back to original branch if we're not rebasing
        log_message "Switching back to $RETURN_TO_BRANCH..."
        git checkout $RETURN_TO_BRANCH 2>/dev/null || { log_message "Error: Failed to switch back to branch '$RETURN_TO_BRANCH'"; exit 1; }
    fi
fi

log_message "Current git status:"
git status | cat # Use cat to prevent pager

exit 0 