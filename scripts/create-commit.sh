#!/bin/bash
# Script to help create commit messages following the yastai.mdc guidelines
# Non-interactive version for automated use by bots

# Function to show usage
show_usage() {
    echo "Usage: ./create-commit.sh \"Commit title\" \"Prompt\" \"Description\" \"Discussion\" [--no-stage]"
    echo ""
    echo "Parameters:"
    echo "  Commit title  - The title of the commit (required)"
    echo "  Prompt        - The prompt that generated the changes (required)"
    echo "  Description   - Detailed description of changes (required)"
    echo "  Discussion    - Challenges or difficulties faced (required)"
    echo "  --no-stage    - Optional flag to skip automatic git add ."
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

# Check for optional flags
for arg in "$@"; do
    if [ "$arg" = "--no-stage" ]; then
        STAGE_ALL=false
    fi
done

# Create a temporary file for the commit message
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

# Stage changes if flag is set
if [ "$STAGE_ALL" = true ]; then
    git add .
fi

# Commit with the generated message
git commit -F "$TEMP_FILE"

# Clean up
rm "$TEMP_FILE"
echo "Commit created successfully!" 