#!/usr/bin/env bash

# ===============================================================
# ai-rules-symlinks.sh - Regenerate Cursor rule symlinks
# ===============================================================
# This script regenerates symbolic links for AI agent rule files
# from docs/agentrules/ to .cursor/rules/ with proper naming
# ===============================================================

set -e  # Exit immediately if a command exits with a non-zero status

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
NC="\033[0m" # No Color

# Directories
SOURCE_DIR="docs/agentrules"
TARGET_DIR=".cursor/rules"
WORKSPACE_ROOT="$(pwd)"

# Banner
echo -e "${GREEN}=======================================================${NC}"
echo -e "${GREEN}      AI Rules Symlinks Generator                      ${NC}"
echo -e "${GREEN}=======================================================${NC}"

# Check if source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo -e "${RED}Error: Source directory '$SOURCE_DIR' does not exist.${NC}"
    exit 1
fi

# Check if target directory exists, create if not
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${YELLOW}Target directory '$TARGET_DIR' does not exist. Creating it...${NC}"
    mkdir -p "$TARGET_DIR"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to create target directory '$TARGET_DIR'.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Created target directory '$TARGET_DIR'.${NC}"
else
    echo -e "${GREEN}✓ Target directory '$TARGET_DIR' exists.${NC}"
fi

# Clean existing .mdc files in target directory
echo -e "${YELLOW}Removing existing .mdc files in '$TARGET_DIR'...${NC}"
existing_files=$(find "$TARGET_DIR" -name "*.mdc" -type l)
if [ -n "$existing_files" ]; then
    echo "$existing_files" | while read -r file; do
        rm "$file"
        echo -e "${YELLOW}  Removed: $file${NC}"
    done
else
    echo -e "${YELLOW}  No existing .mdc files found.${NC}"
fi

# Find all *_mdc.txt files and create symlinks
echo -e "${GREEN}Creating symbolic links...${NC}"

# Use an array to store created links for counting
created_links=()

find "$SOURCE_DIR" -name "*_mdc.txt" -type f | while read -r source_file; do
    # Extract filename without path
    filename=$(basename "$source_file")
    
    # Extract part before underscore
    base_name=$(echo "$filename" | sed -E 's/(.*)_mdc\.txt/\1/')
    
    # Define target filename
    target_file="$TARGET_DIR/$base_name.mdc"
    
    # Create symbolic link
    ln -sf "$WORKSPACE_ROOT/$source_file" "$WORKSPACE_ROOT/$target_file"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}  ✓ Created link: $target_file -> $source_file${NC}"
        created_links+=("$target_file")
    else
        echo -e "${RED}  ✗ Failed to create link: $target_file -> $source_file${NC}"
    fi
done

# Count the number of created links
count=$(find "$TARGET_DIR" -name "*.mdc" -type l | wc -l)

# Summary
if [ $count -gt 0 ]; then
    echo -e "${GREEN}=======================================================${NC}"
    echo -e "${GREEN}✓ Successfully created $count symbolic links.${NC}"
    echo -e "${GREEN}=======================================================${NC}"
else
    echo -e "${YELLOW}=======================================================${NC}"
    echo -e "${YELLOW}! No matching *_mdc.txt files found in $SOURCE_DIR.${NC}"
    echo -e "${YELLOW}=======================================================${NC}"
fi

exit 0 