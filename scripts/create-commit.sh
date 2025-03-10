#!/bin/bash
# Script to help create commit messages following the yastwai.mdc guidelines
# Uses multi-line format by default to improve readability and comply with guidelines
# Non-interactive version for automated use by bots

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/create-commit.sh \"Commit title\" \"Prompt\" \"Description\" \"Discussion\" [--no-stage] [--single-line]"
    echo ""
    echo "Parameters:"
    echo "  Commit title  - The title of the commit (required)"
    echo "  Prompt        - The prompt that generated the changes (required)"
    echo "  Description   - Detailed description of changes (required)"
    echo "  Discussion    - Challenges or difficulties faced (required)"
    echo "  --no-stage    - Optional flag to skip automatic git add ."
    echo "  --single-line - Use single-line commit message format (default is multi-line)"
    echo "  --help        - Display this help message"
    echo ""
    echo "Note: For multi-line text in any parameter, use escaped newlines (\\n)"
    exit 1
}

# Check for help flag
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_usage
fi

# Check if the minimum required parameters were provided
if [ $# -lt 4 ]; then
    show_usage
fi

# Process the parameters (enable escaped newlines)
COMMIT_TITLE=$(echo -e "$1")
PROMPT=$(echo -e "$2")
DESCRIPTION=$(echo -e "$3")
DISCUSSION=$(echo -e "$4")
STAGE_ALL=true
MULTI_LINE=true

# Check for optional flags
for arg in "$@"; do
    if [ "$arg" = "--no-stage" ]; then
        STAGE_ALL=false
    fi
    if [ "$arg" = "--single-line" ]; then
        MULTI_LINE=false
    fi
done

# Stage changes if flag is set
if [ "$STAGE_ALL" = true ]; then
    git add .
fi

if [ "$MULTI_LINE" = true ]; then
    # Create a temporary file for the multi-line commit message
    TEMP_FILE=$(mktemp)

    # Build the commit message
    echo "$COMMIT_TITLE" > "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    echo "Prompt: $PROMPT" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    echo "Description: " >> "$TEMP_FILE"
    echo "$DESCRIPTION" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    echo "Discussion: " >> "$TEMP_FILE"
    echo "$DISCUSSION" >> "$TEMP_FILE"

    # Commit with the generated message
    git commit -F "$TEMP_FILE"

    # Clean up
    rm "$TEMP_FILE"
else
    # Create a single-line commit message
    COMMIT_MSG="$COMMIT_TITLE - Prompt: $PROMPT - Description: $DESCRIPTION - Discussion: $DISCUSSION"
    
    # Commit with single-line message
    git commit -m "$COMMIT_MSG"
fi

echo "Commit created successfully!" 