#!/bin/bash
# AI Assistant Helper Script for Commit Creation
# This script provides a simple interface for AI assistants to create commits
# Uses two modes: preview (default) and execute
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
    echo "  --mode=[preview|execute]  - 'preview' shows what would be committed, 'execute' performs the commit (default: preview)"
    echo "  --help, -h                - Show this help message"
    echo ""
    echo "WORKFLOW FOR AI AGENTS:"
    echo "1. First call with preview mode to show the user what would be committed"
    echo '   ./scripts/ai-commit.sh --mode=preview "Title" "Description" "Prompt" "Reasoning" "Challenges"'
    echo "2. If user approves, call again with execute mode to actually commit"
    echo '   ./scripts/ai-commit.sh --mode=execute "Title" "Description" "Prompt" "Reasoning" "Challenges"'
    echo ""
    echo "NOTE: Always use quotes around each argument to handle spaces correctly."
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default settings
MODE="preview"

# Parse options
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --mode=*)
            VALUE="${1#*=}"
            if [[ "$VALUE" == "preview" || "$VALUE" == "execute" ]]; then
                MODE="$VALUE"
            else
                log_message "Error: --mode must be 'preview' or 'execute'"
                show_usage
            fi
            shift
            ;;
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

# Get list of staged and unstaged changes
if [ "$MODE" = "preview" ]; then
    log_message "Files that would be committed:"
    git status --porcelain | cat
    
    # Show detailed changes
    log_message "Detailed changes (git diff):"
    git diff --staged | cat
    
    log_message "PREVIEW MODE: No changes committed. To commit these changes, run the same command with --mode=execute"
else
    # We're in execute mode, actually perform the commit
    log_message "Staging all changes..."
    git add .
    
    # Commit with the generated message
    git commit -F "$TEMP_FILE"
    log_message "Commit created successfully!"
fi

# Clean up
rm -f "$TEMP_FILE"
exit 0 