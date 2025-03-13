#!/bin/bash
# AI Assistant Helper Script for Main Branch Protection
# This script is designed to be called at the beginning of every AI interaction
# to automatically prevent work on the main branch

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-protect-main.sh [options]"
    echo ""
    echo "Options:"
    echo "  --auto-branch NAME  - Automatically create a feature branch with the given name if on main"
    echo "  --no-auto-branch    - Only check main branch status, but don't auto-create branch"
    echo "  --help              - Display this help message"
    echo ""
    echo "Example:"
    echo "  ./scripts/ai-protect-main.sh --auto-branch feature-name"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Parse arguments
AUTO_BRANCH=""
NO_AUTO_BRANCH=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --auto-branch)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --auto-branch requires a name parameter"
                show_usage
            fi
            AUTO_BRANCH="$2"
            shift 2
            ;;
        --no-auto-branch)
            NO_AUTO_BRANCH=true
            shift
            ;;
        --help)
            show_usage
            ;;
        *)
            log_message "Error: Unknown option: $1"
            show_usage
            ;;
    esac
done

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
log_message "Current branch: $CURRENT_BRANCH"

# Main branch protection logic
if [[ "$CURRENT_BRANCH" == "main" ]]; then
    log_message "⚠️ WARNING: Currently on main branch. Direct work on main is prohibited."
    
    if [[ "$NO_AUTO_BRANCH" == true ]]; then
        log_message "CRITICAL: Create a feature branch immediately before proceeding!"
        log_message "Run: ./scripts/ai-branch.sh --new-branch \"feature-name\" --is-related false"
        exit 1
    elif [[ -n "$AUTO_BRANCH" ]]; then
        log_message "ℹ️ PROTECTION ACTIVATED: Automatically creating feature branch: $AUTO_BRANCH"
        
        # Check for uncommitted changes
        if [[ -n $(git status --porcelain) ]]; then
            log_message "⚠️ Uncommitted changes detected. Stashing changes before creating branch..."
            git stash -u
            
            # Create new branch from main
            if ! ./scripts/ai-branch.sh --new-branch "$AUTO_BRANCH" --is-related false; then
                log_message "❌ Failed to create branch. Please create a feature branch manually."
                git stash pop
                exit 1
            fi
            
            # Apply stashed changes
            log_message "Applying stashed changes to new branch..."
            git stash pop
        else
            # Create new branch from main
            if ! ./scripts/ai-branch.sh --new-branch "$AUTO_BRANCH" --is-related false; then
                log_message "❌ Failed to create branch. Please create a feature branch manually."
                exit 1
            fi
        fi
        
        log_message "✅ Now working on branch: $(git branch --show-current)"
    else
        log_message "ERROR: Must provide either --auto-branch NAME or --no-auto-branch"
        log_message "Run with --help for usage information"
        exit 1
    fi
else
    log_message "✅ Working on branch: $CURRENT_BRANCH (not main) - Proceeding safely"
fi

exit 0 