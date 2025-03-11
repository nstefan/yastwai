#!/bin/bash
# Simple AI Assistant Helper Script for Commit Creation
# This script provides a simpler interface for AI assistants to create commits
# Uses positional arguments for ease of use

set -e  # Exit on error

# Function to show usage with clear examples
show_usage() {
    echo "Usage: ./scripts/ai-commit-simple.sh TITLE DESCRIPTION PROMPT [THOUGHT_PROCESS] [DISCUSSION]"
    echo ""
    echo "Arguments:"
    echo "  TITLE            - Commit title (required)"
    echo "  DESCRIPTION      - Short description of changes (required)"
    echo "  PROMPT           - Original prompt that generated changes (required)"
    echo "  THOUGHT_PROCESS  - Reasoning process (optional)"
    echo "  DISCUSSION       - Challenges faced (optional)"
    echo ""
    echo "EXAMPLE FOR AI AGENTS:"
    echo './scripts/ai-commit-simple.sh "Update documentation" "Reorganized docs" "commit changes" "Analyzed structure and made changes" "Created new branch to avoid main"'
    echo ""
    echo "NOTE: Always use quotes around each argument to handle spaces correctly."
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Check if help is requested
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    show_usage
fi

# Check if we have at least the required arguments
if [ $# -lt 3 ]; then
    log_message "Error: At least 3 arguments required (TITLE, DESCRIPTION, PROMPT)"
    show_usage
fi

# Assign arguments to variables
COMMIT_TITLE="$1"
DESCRIPTION="$2"
PROMPT="$3"
THOUGHT_PROCESS="${4:-}"  # Optional
DISCUSSION="${5:-}"       # Optional

# Stage all changes
log_message "Staging all changes..."
git add .

# Create a temporary file for the commit message
TEMP_FILE=$(mktemp)

# Build the commit message
echo "$COMMIT_TITLE" > "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "Short description: $DESCRIPTION" >> "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "Prompt: $PROMPT" >> "$TEMP_FILE"
echo "" >> "$TEMP_FILE"

if [ -n "$THOUGHT_PROCESS" ]; then
    echo "Chain of thoughts: " >> "$TEMP_FILE"
    echo "$THOUGHT_PROCESS" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
fi

if [ -n "$DISCUSSION" ]; then
    echo "Discussion: " >> "$TEMP_FILE"
    echo "$DISCUSSION" >> "$TEMP_FILE"
fi

# Display the commit message
log_message "Generated commit message:"
log_message "---------------------------------------------"
cat "$TEMP_FILE" | sed 's/^/    /'
log_message "---------------------------------------------"

# Commit with the generated message
git commit -F "$TEMP_FILE"
log_message "Commit created successfully!"

# Clean up
rm -f "$TEMP_FILE"
exit 0 