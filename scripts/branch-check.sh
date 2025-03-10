#!/bin/bash
# Script to check and manage branches according to yastai.mdc requirements

# Get the current branch
CURRENT_BRANCH=$(git branch --show-current)
MAIN_BRANCH="main"

# Check if we're on the main branch
if [ "$CURRENT_BRANCH" = "$MAIN_BRANCH" ]; then
    echo "Currently on main branch. Creating a new branch is recommended."
    read -p "Enter new branch name (or press Enter to stay on main): " NEW_BRANCH
    
    if [ -n "$NEW_BRANCH" ]; then
        git checkout -b "$NEW_BRANCH"
        echo "Created and switched to new branch: $NEW_BRANCH"
    else
        echo "Staying on main branch. This is not recommended for development work."
    fi
else
    echo "Currently on branch: $CURRENT_BRANCH"
    
    read -p "Is your new work related to this branch? (y/n): " IS_RELATED
    
    if [ "$IS_RELATED" = "y" ] || [ "$IS_RELATED" = "Y" ]; then
        echo "Continuing work on branch: $CURRENT_BRANCH"
    else
        echo "Work not related to current branch. Please create a new branch from main."
        
        # Check if there are uncommitted changes
        if [ -n "$(git status --porcelain)" ]; then
            echo "Warning: You have uncommitted changes. Please commit or stash them before switching branches."
            exit 1
        fi
        
        # Switch to main and pull latest changes
        git checkout "$MAIN_BRANCH"
        git pull
        
        # Create new branch
        read -p "Enter new branch name: " NEW_BRANCH
        
        if [ -n "$NEW_BRANCH" ]; then
            git checkout -b "$NEW_BRANCH"
            echo "Created and switched to new branch: $NEW_BRANCH"
        else
            echo "No branch name provided. Staying on $MAIN_BRANCH."
        fi
    fi
fi

echo ""
echo "Current git status:"
git status 