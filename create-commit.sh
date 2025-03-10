#!/bin/bash
# Script to help create commit messages following the yastai.mdc guidelines

# Check if the commit title was provided
if [ $# -lt 1 ]; then
    echo "Usage: ./create-commit.sh \"Commit title\" [\"Optional prompt\"]"
    exit 1
fi

COMMIT_TITLE="$1"
PROMPT="${2:-"No prompt specified"}"

# Create a temporary file for the commit message
TEMP_FILE=$(mktemp)

# Start building the commit message
echo "$COMMIT_TITLE" > "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "Prompt: $PROMPT" >> "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "Description: " >> "$TEMP_FILE"
echo "# Enter a detailed description of your changes here" >> "$TEMP_FILE"
echo "# Lines starting with '#' will be ignored" >> "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
echo "Discussion: " >> "$TEMP_FILE"
echo "# Enter any challenges or difficulties faced during implementation" >> "$TEMP_FILE"

# Open the editor to complete the commit message
if [ -n "$EDITOR" ]; then
    $EDITOR "$TEMP_FILE"
elif which nano > /dev/null 2>&1; then
    nano "$TEMP_FILE"
elif which vim > /dev/null 2>&1; then
    vim "$TEMP_FILE"
else
    echo "No editor found. Please edit $TEMP_FILE manually."
    exit 1
fi

# Remove comment lines
grep -v "^#" "$TEMP_FILE" > "${TEMP_FILE}.clean"

# Check if we should proceed with the commit
echo "Commit message preview:"
echo "----------------------"
cat "${TEMP_FILE}.clean"
echo "----------------------"
echo ""
read -p "Proceed with commit? (y/n): " PROCEED

if [ "$PROCEED" = "y" ] || [ "$PROCEED" = "Y" ]; then
    # Stage changes
    read -p "Stage all changes? (y/n): " STAGE_ALL
    if [ "$STAGE_ALL" = "y" ] || [ "$STAGE_ALL" = "Y" ]; then
        git add .
    else
        echo "Please stage your changes manually before committing."
        read -p "Press Enter when ready to continue..."
    fi

    # Commit with the generated message
    git commit -F "${TEMP_FILE}.clean"
    
    # Clean up
    rm "$TEMP_FILE" "${TEMP_FILE}.clean"
    echo "Commit created successfully!"
else
    echo "Commit aborted. The prepared message is available at ${TEMP_FILE}.clean"
fi 