#!/bin/bash
# AI Assistant Helper Script for Cursor Model Detection
# This script detects the current AI model being used in Cursor
# Follows the naming pattern of ai-*.sh for consistency

set -e  # Exit on error

# Function to show usage with clear examples
show_usage() {
    echo "Usage: ./scripts/ai-cursor-model.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h           - Show this help message"
    echo "  --quiet, -q          - Only output the model name (no additional logging)"
    echo ""
    echo "WORKFLOW FOR AI AGENTS:"
    echo "1. Use this script to detect the current AI model"
    echo "   MODEL=\"$(./scripts/ai-cursor-model.sh --quiet)\""
    echo ""
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    if [[ "$QUIET_MODE" != "true" ]]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    fi
}

# Parse options
QUIET_MODE="false"
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_usage
            ;;
        --quiet|-q)
            QUIET_MODE="true"
            shift
            ;;
        *)
            log_message "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Function to detect the current model - only using SQLite database method
detect_model() {
    local detected_model=""
    
    # Check SQLite database in Cursor application directory
    if command -v sqlite3 &> /dev/null && [[ -f ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb ]]; then
        # Look for composerModel in the aiSettings object
        model_from_db=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM ItemTable WHERE key LIKE '%aiSettings%'" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | cut -d'"' -f4)
        
        if [[ -n "$model_from_db" ]]; then
            detected_model="$model_from_db"
            log_message "Detected model from database: $detected_model"
            echo "$detected_model"
            return
        fi
    fi
    
    # Use "N/A" if no model could be detected
    detected_model="N/A"
    log_message "Could not detect model, returning: $detected_model"
    echo "$detected_model"
}

# Main execution
MODEL=$(detect_model)

# Output the model name
if [[ "$QUIET_MODE" == "true" ]]; then
    echo "$MODEL"
else
    log_message "AI Model detected: $MODEL"
    echo "$MODEL"
fi

exit 0 