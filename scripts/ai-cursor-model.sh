#!/bin/bash
# AI Assistant Helper Script for Cursor Model Detection
# This script detects the current AI model being used in Cursor
# Follows the naming pattern of ai-*.sh for consistency
# By default, outputs only the model name with no additional logging

set -e  # Exit on error

# Function to show usage with clear examples
show_usage() {
    echo "Usage: ./scripts/ai-cursor-model.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h           - Show this help message"
    echo "  --verbose, -v        - Show detailed logging information"
    echo ""
    echo "WORKFLOW FOR AI AGENTS:"
    echo "1. Use this script to detect the current AI model"
    echo "   MODEL=\"$(./scripts/ai-cursor-model.sh)\""
    echo ""
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    if [[ "$VERBOSE_MODE" == "true" ]]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    fi
}

# Parse options
VERBOSE_MODE="false"
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_usage
            ;;
        --verbose|-v)
            VERBOSE_MODE="true"
            shift
            ;;
        --quiet|-q)
            # Keep quiet flag for backward compatibility
            shift
            ;;
        *)
            log_message "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Function to detect the current model - search for model information
detect_model() {
    local detected_model=""
    local source=""
    
    # Check SQLite database in Cursor application directory
    if command -v sqlite3 &> /dev/null && [[ -f ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb ]]; then
        # First try to find model in inlineDiffsData (most likely location)
        model_from_db=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM cursorDiskKV WHERE key = 'inlineDiffsData'" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | cut -d'"' -f4)
        
        if [[ -n "$model_from_db" ]]; then
            detected_model="$model_from_db"
            source="inlineDiffsData"
            echo "$detected_model|$source"
            return
        fi
        
        # Try each composerData entry (there are many)
        model_from_composer=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM cursorDiskKV WHERE key LIKE 'composerData%' LIMIT 10" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | head -1 | cut -d'"' -f4)
            
        if [[ -n "$model_from_composer" ]]; then
            detected_model="$model_from_composer"
            source="composerData"
            echo "$detected_model|$source"
            return
        fi
        
        # Try to check if present in any of the data with a broader search
        any_model=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM ItemTable" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | head -1 | cut -d'"' -f4)
            
        if [[ -n "$any_model" ]]; then
            detected_model="$any_model"
            source="general search"
            echo "$detected_model|$source"
            return
        fi
    fi
    
    # Use "Claude 3.7 Sonnet" as a fallback (Cursor's default model)
    detected_model="Claude 3.7 Sonnet"
    source="default fallback"
    echo "$detected_model|$source"
}

# Main execution
RESULT=$(detect_model)
MODEL=$(echo "$RESULT" | cut -d'|' -f1)
SOURCE=$(echo "$RESULT" | cut -d'|' -f2)

# Output the model name
if [[ "$VERBOSE_MODE" == "true" ]]; then
    log_message "Detected model from $SOURCE: $MODEL"
    echo "$MODEL"
else
    echo "$MODEL"
fi

exit 0 