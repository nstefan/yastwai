#!/bin/bash
# AI Assistant Helper Script for Commit Creation
# This script helps AI assistants create structured commit messages
# without having to deal with multiline input issues
# Follows the naming pattern of ai-*-helper.sh for consistency

set -e  # Exit on error

# Function to show usage with clear examples
show_usage() {
    echo "Usage: ./scripts/ai-commit-helper.sh [options]"
    echo ""
    echo "Options:"
    echo "  --title TITLE           - Commit title (required)"
    echo "  --description TEXT      - Short description of changes (required)"
    echo "  --prompt TEXT           - Original prompt that generated changes (required)"
    echo "  --thought-process TEXT  - Reasoning process (comma-separated for multiple lines)"
    echo "  --discussion TEXT       - Challenges faced (comma-separated for multiple lines)"
    echo "  --no-stage              - Skip automatic git add ."
    echo "  --single-line           - Use single-line format instead of multi-line"
    echo "  --dry-run               - Show the commit message without committing"
    echo "  --help                  - Display this help message"
    echo ""
    echo "EXAMPLE FOR AI AGENTS:"
    echo "./scripts/ai-commit-helper.sh \\"
    echo "  --title \"Update documentation structure\" \\"
    echo "  --description \"Reorganized documentation files\" \\"
    echo "  --prompt \"commit recent changes\" \\"
    echo "  --thought-process \"First identified changes,Second made a new branch,Third committed the changes\" \\"
    echo "  --discussion \"Had to create a new branch to avoid committing to main\""
    echo ""
    echo "NOTE: For multiline inputs, use commas to separate lines (no spaces around commas)."
    echo "Each comma will be converted to a newline in the commit message."
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
COMMIT_TITLE=""
DESCRIPTION=""
PROMPT=""
THOUGHT_PROCESS=""
DISCUSSION=""
STAGE_ALL=true
MULTI_LINE=true
DRY_RUN=false

# Check if no arguments provided
if [ $# -eq 0 ]; then
    log_message "Error: No arguments provided"
    show_usage
fi

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --title)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --title requires a value"
                show_usage
            fi
            COMMIT_TITLE="$2"
            shift 2
            ;;
        --description)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --description requires a value"
                show_usage
            fi
            DESCRIPTION="$2"
            shift 2
            ;;
        --prompt)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --prompt requires a value"
                show_usage
            fi
            PROMPT="$2"
            shift 2
            ;;
        --thought-process)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --thought-process requires a value"
                show_usage
            fi
            THOUGHT_PROCESS="$2"
            shift 2
            ;;
        --discussion)
            if [[ -z "$2" || "$2" == --* ]]; then
                log_message "Error: --discussion requires a value"
                show_usage
            fi
            DISCUSSION="$2"
            shift 2
            ;;
        --no-stage)
            STAGE_ALL=false
            shift
            ;;
        --single-line)
            MULTI_LINE=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            log_message "Error: Unknown option: $1"
            show_usage
            ;;
    esac
done

# Check required parameters with clear error messages
if [ -z "$COMMIT_TITLE" ]; then
    log_message "Error: Commit title is required (--title)"
    show_usage
fi

if [ -z "$DESCRIPTION" ]; then
    log_message "Error: Description is required (--description)"
    show_usage
fi

if [ -z "$PROMPT" ]; then
    log_message "Error: Prompt is required (--prompt)"
    show_usage
fi

# Function to convert comma-separated list to multi-line text
comma_to_lines() {
    if [[ "$1" == *","* ]]; then
        IFS=',' read -ra LINES <<< "$1"
        result=""
        for line in "${LINES[@]}"; do
            result="${result}${line}\n"
        done
        echo -e "$result"
    else
        echo "$1"
    fi
}

# Process thought process and discussion to handle comma-separated items
PROCESSED_THOUGHT=$(comma_to_lines "$THOUGHT_PROCESS")
PROCESSED_DISCUSSION=$(comma_to_lines "$DISCUSSION")

# Stage changes if flag is set
if [ "$STAGE_ALL" = true ] && [ "$DRY_RUN" = false ]; then
    log_message "Staging all changes..."
    git add .
fi

# Create a temporary file for the multi-line commit message
TEMP_FILE=$(mktemp)

if [ "$MULTI_LINE" = true ]; then
    # Build the commit message
    echo "$COMMIT_TITLE" > "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    echo "Short description: $DESCRIPTION" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    echo "Prompt: $PROMPT" >> "$TEMP_FILE"
    echo "" >> "$TEMP_FILE"
    
    if [ -n "$THOUGHT_PROCESS" ]; then
        echo "Chain of thoughts: " >> "$TEMP_FILE"
        echo "$PROCESSED_THOUGHT" >> "$TEMP_FILE"
        echo "" >> "$TEMP_FILE"
    fi
    
    if [ -n "$DISCUSSION" ]; then
        echo "Discussion: " >> "$TEMP_FILE"
        echo "$PROCESSED_DISCUSSION" >> "$TEMP_FILE"
    fi

    # Display the commit message
    log_message "Generated commit message:"
    log_message "---------------------------------------------"
    cat "$TEMP_FILE" | sed 's/^/    /'
    log_message "---------------------------------------------"

    # Commit with the generated message if not dry run
    if [ "$DRY_RUN" = false ]; then
        git commit -F "$TEMP_FILE"
        log_message "Commit created successfully!"
    else
        log_message "Dry run: No commit was created."
    fi
else
    # Create a single-line commit message
    COMMIT_MSG="$COMMIT_TITLE - Short description: $DESCRIPTION - Prompt: $PROMPT"
    
    if [ -n "$THOUGHT_PROCESS" ]; then
        COMMIT_MSG="$COMMIT_MSG - Chain of thoughts: $THOUGHT_PROCESS"
    fi
    
    if [ -n "$DISCUSSION" ]; then
        COMMIT_MSG="$COMMIT_MSG - Discussion: $DISCUSSION"
    fi
    
    # Display the commit message
    log_message "Generated single-line commit message:"
    log_message "---------------------------------------------"
    echo "    $COMMIT_MSG"
    log_message "---------------------------------------------"

    # Commit with single-line message if not dry run
    if [ "$DRY_RUN" = false ]; then
        git commit -m "$COMMIT_MSG"
        log_message "Commit created successfully!"
    else
        log_message "Dry run: No commit was created."
    fi
fi

# Clean up
rm -f "$TEMP_FILE"
exit 0 