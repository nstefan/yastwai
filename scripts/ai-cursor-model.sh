#!/bin/bash
# AI Assistant Helper Script for Cursor Model Detection
# Simple, reliable model detection without unnecessary complexity

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-cursor-model.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h           - Show this help message"
    echo "  --verbose, -v        - Show detailed logging information"
    echo "  --quiet, -q          - Quiet output (model only)"
    echo ""
    echo "WORKFLOW FOR AI AGENTS:"
    echo "1. Use this script to detect the current AI model"
    echo "   MODEL=\"\$(./scripts/ai-cursor-model.sh)\""
    echo ""
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    if [[ "$VERBOSE_MODE" == "true" ]]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >&2
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
            # Keep quiet flag for backward compatibility (default behavior)
            shift
            ;;
        *)
            log_message "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Check for explicit environment variables first
for VAR in CURSOR_CURRENT_MODEL AI_CURSOR_MODEL AI_MODEL MODEL_NAME; do
    if [[ -n "${!VAR}" ]]; then
        log_message "Found model from environment variable $VAR: ${!VAR}"
        echo "${!VAR}"
        exit 0
    fi
done

# Function to detect model from Cursor database
detect_cursor_model() {
    local db_path="$HOME/Library/Application Support/Cursor/User/globalStorage/state.vscdb"
    
    # Check if SQLite and database exist
    if ! command -v sqlite3 &> /dev/null; then
        log_message "SQLite not available"
        return 1
    fi
    
    if [[ ! -f "$db_path" ]]; then
        log_message "Cursor database not found at: $db_path"
        return 1
    fi
    
    log_message "Querying Cursor database for model information..."
    
    # Query for recent composer data and extract model name
    local model_name
    model_name=$(sqlite3 -readonly "$db_path" \
        "PRAGMA query_only=ON; SELECT hex(value) FROM cursorDiskKV WHERE key GLOB 'composerData:*' ORDER BY rowid DESC LIMIT 10;" \
        2>/dev/null | xxd -r -p 2>/dev/null | grep -o '"modelName":"[^"]*"' | head -1 | cut -d '"' -f4 || true)
    
    if [[ -n "$model_name" ]]; then
        log_message "Found model from Cursor database: $model_name"
        echo "$model_name"
        return 0
    fi
    
    # Fallback: try composerModel field
    model_name=$(sqlite3 -readonly "$db_path" \
        "PRAGMA query_only=ON; SELECT hex(value) FROM cursorDiskKV WHERE key GLOB 'composerData:*' ORDER BY rowid DESC LIMIT 10;" \
        2>/dev/null | xxd -r -p 2>/dev/null | grep -o '"composerModel":"[^"]*"' | head -1 | cut -d '"' -f4 || true)
    
    if [[ -n "$model_name" ]]; then
        log_message "Found model from Cursor database (composerModel): $model_name"
        echo "$model_name"
        return 0
    fi
    
    log_message "No model found in Cursor database"
    return 1
}

# Main detection logic
if detect_cursor_model; then
    # Model found and printed by detect_cursor_model
    exit 0
else
    # No model detected
    log_message "Could not detect current AI model"
    echo "N/A"
    exit 0
fi