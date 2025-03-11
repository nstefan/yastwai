#!/bin/bash
# AI Assistant Helper Script for Commit Creation
# This script provides a simple interface for AI assistants to create commits
# Follows the naming pattern of ai-*.sh for consistency

set -e  # Exit on error

# Function to show usage with clear examples
show_usage() {
    echo "Usage: ./scripts/ai-commit.sh [OPTIONS] TITLE DESCRIPTION PROMPT [THOUGHT_PROCESS] [DISCUSSION]"
    echo ""
    echo "Arguments:"
    echo "  TITLE            - Commit title (required)"
    echo "  DESCRIPTION      - Short description of changes (required)"
    echo "  PROMPT           - Original prompt that generated changes (required)"
    echo "  THOUGHT_PROCESS  - Reasoning process (optional)"
    echo "  DISCUSSION       - Challenges faced (optional)"
    echo ""
    echo "Options:"
    echo "  --help, -h                - Show this help message"
    echo ""
    echo "WORKFLOW FOR AI AGENTS:"
    echo "1. AI presents a formatted preview directly to the user (without using this script)"
    echo "2. If user approves, call this script to execute the commit"
    echo '   ./scripts/ai-commit.sh "Title" "Description" "Prompt" "Reasoning" "Challenges"'
    echo ""
    echo "NOTE: Always use quotes around each argument to handle spaces correctly."
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Parse options
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_usage
            ;;
        *)
            POSITIONAL_ARGS+=("$1")
            shift
            ;;
    esac
done

# Restore positional arguments
set -- "${POSITIONAL_ARGS[@]}"

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

# Stage and commit changes
log_message "Staging all changes..."
git add .

# Commit with the generated message
git commit -F "$TEMP_FILE"
log_message "Commit created successfully!"

# Clean up
rm -f "$TEMP_FILE"
exit 0 