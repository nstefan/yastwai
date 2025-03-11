#!/bin/bash
# AI Assistant Helper Script for Commit Creation
# This script provides a simple interface for AI assistants to create commits
# Uses positional arguments for ease of use
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
    echo "  --require-user-approval [true|false]  - Whether to require user approval before committing (default: true)"
    echo "  require_user_approval=[true|false]    - Same as above, alternate syntax for tool integration"
    echo "  --help, -h                          - Show this help message"
    echo ""
    echo "EXAMPLE FOR AI AGENTS:"
    echo './scripts/ai-commit.sh "Update documentation" "Reorganized docs" "commit changes" "Analyzed structure and made changes" "Created new branch to avoid main"'
    echo ""
    echo "NOTE: Always use quotes around each argument to handle spaces correctly."
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default settings
REQUIRE_USER_APPROVAL=true

# Parse options
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --require-user-approval)
            if [[ "$2" == "false" ]]; then
                REQUIRE_USER_APPROVAL=false
            elif [[ "$2" == "true" || -z "$2" ]]; then
                REQUIRE_USER_APPROVAL=true
            else
                log_message "Error: --require-user-approval must be 'true' or 'false'"
                show_usage
            fi
            shift 2
            ;;
        require_user_approval=*)
            VALUE="${1#*=}"
            if [[ "$VALUE" == "false" ]]; then
                REQUIRE_USER_APPROVAL=false
            elif [[ "$VALUE" == "true" || -z "$VALUE" ]]; then
                REQUIRE_USER_APPROVAL=true
            else
                log_message "Error: require_user_approval must be 'true' or 'false'"
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

# Commit with the generated message based on user approval setting
if [ "$REQUIRE_USER_APPROVAL" = true ]; then
    # Ask for user confirmation
    read -p "Do you want to commit these changes? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git commit -F "$TEMP_FILE"
        log_message "Commit created successfully!"
    else
        log_message "Commit aborted by user."
        # Clean up
        rm -f "$TEMP_FILE"
        exit 0
    fi
else
    # Automatic commit without user approval
    git commit -F "$TEMP_FILE"
    log_message "Commit created automatically (user approval not required)!"
fi

# Clean up
rm -f "$TEMP_FILE"
exit 0 