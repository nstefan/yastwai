#!/bin/bash
# AI Assistant Helper Script for Branch Management
# This script helps AI assistants manage git branches without requiring interactive input
# Follows the naming pattern of ai-*-helper.sh for consistency

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-branch-helper.sh [options]"
    echo ""
    echo "Options:"
    echo "  --check-only         - Only check current branch status without creating a new branch"
    echo "  --new-branch NAME    - Create a new branch with the specified name from main"
    echo "  --is-related BOOL    - Specify if new work is related to current branch (true/false)"
    echo "  --force              - Force branch creation even if there are uncommitted changes"
    echo "  --help               - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
CHECK_ONLY=false
NEW_BRANCH=""
IS_RELATED=true
FORCE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --new-branch)
            NEW_BRANCH="$2"
            shift 2
            ;;
        --is-related)
            IS_RELATED="$2"
            shift 2
            ;;
        --force)
            FORCE=true
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

# Get the current branch and main branch
CURRENT_BRANCH=$(git branch --show-current | cat)
MAIN_BRANCH="main"

log_message "Current branch: $CURRENT_BRANCH"

# Check if we're on the main branch
if [ "$CURRENT_BRANCH" = "$MAIN_BRANCH" ]; then
    log_message "Currently on main branch."
    
    if [ "$CHECK_ONLY" = true ]; then
        log_message "Check-only mode: Not creating a new branch."
        exit 0
    fi
    
    if [ -z "$NEW_BRANCH" ]; then
        log_message "Error: On main branch but no new branch name provided with --new-branch."
        log_message "Creating a new branch is recommended for development work."
        exit 1
    fi
    
    # Create new branch from main
    git checkout -b "$NEW_BRANCH"
    log_message "Created and switched to new branch: $NEW_BRANCH"
else
    log_message "Currently on branch: $CURRENT_BRANCH"
    
    if [ "$CHECK_ONLY" = true ]; then
        log_message "Check-only mode: Not creating a new branch."
        exit 0
    fi
    
    if [ "$IS_RELATED" = "true" ]; then
        log_message "Work is related to current branch. Continuing on: $CURRENT_BRANCH"
    else
        log_message "Work not related to current branch. Need to create a new branch from main."
        
        # Check if there are uncommitted changes
        if [ -n "$(git status --porcelain)" ] && [ "$FORCE" = false ]; then
            log_message "Error: You have uncommitted changes. Use --force to override or commit/stash changes first."
            exit 1
        fi
        
        if [ -z "$NEW_BRANCH" ]; then
            log_message "Error: No new branch name provided with --new-branch."
            exit 1
        fi
        
        # Switch to main and pull latest changes
        git checkout "$MAIN_BRANCH"
        git pull
        
        # Create new branch
        git checkout -b "$NEW_BRANCH"
        log_message "Created and switched to new branch: $NEW_BRANCH"
    fi
fi

log_message "Current git status:"
git status | cat

exit 0 