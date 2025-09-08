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

# Environment detection - check if running in Claude CLI
if [[ -n "$CLAUDE_CODE_CLI" ]]; then
    # This is a new environment variable we'll add for Claude Code CLI
    echo "claude-3-7-sonnet-20250219"
    exit 0
fi

# Removed early trust of MODEL_NAME to avoid stale/incorrect values; rely on detection logic below

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
        --force-claude-cli)
            # Force detection as Claude CLI
            echo "claude-3-7-sonnet-20250219"
            exit 0
            ;;
        *)
            log_message "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Prefer explicit environment variables if present (override DB heuristics)
for VAR in CURSOR_CURRENT_MODEL AI_CURSOR_MODEL AI_MODEL OPENAI_MODEL CLAUDE_MODEL MODEL_NAME; do
    if [[ -n "${!VAR}" ]]; then
        echo "${!VAR}"
        exit 0
    fi
done

# Function to detect the current model - search for model information
detect_model() {
    local detected_model=""
    local source=""
    
    # Check if we're running in Claude CLI
    if [[ -n "$CLAUDE_CLI" || "$TERM_PROGRAM" == "Claude Code" || -f /.clauderc ]]; then
        # Create a temp file to check for Claude CLI
        temp_file=$(mktemp)
        echo "#!/bin/bash" > "$temp_file"
        echo "echo \$MODEL_NAME" >> "$temp_file"
        chmod +x "$temp_file"
        
        # Try to get model name from environment or set default
        if model_name=$("$temp_file" 2>/dev/null); then
            if [[ -n "$model_name" && "$model_name" != "MODEL_NAME" ]]; then
                detected_model="$model_name"
                source="Claude CLI environment"
                rm "$temp_file"
                echo "$detected_model|$source"
                return
            fi
        fi
        
        rm "$temp_file"
        
        # Hardcoded detection for Claude CLI
        if ps -ef | grep -q "[c]laude"; then
            detected_model="claude-3-7-sonnet-20250219"
            source="Claude CLI process detection"
            echo "$detected_model|$source"
            return
        fi
        
        # Default to correct Claude version
        detected_model="claude-3-7-sonnet-20250219"
        source="Claude CLI detection"
        echo "$detected_model|$source"
        return
    fi
    
    # Check environment variable (useful for testing or when other detection methods fail)
    if [[ -n "$CLAUDE_MODEL" ]]; then
        detected_model="$CLAUDE_MODEL"
        source="environment variable"
        echo "$detected_model|$source"
        return
    fi
    
    # Check for common CLI environment indicators 
    if [[ -n "$ANTHROPIC_API_KEY" || "$AWS_EXECUTION_ENV" == *"anthropic"* ]]; then
        detected_model="claude-3-7-sonnet-20250219"
        source="API environment detection"
        echo "$detected_model|$source"
        return
    fi
    
    # Check SQLite database in Cursor application directory
    if command -v sqlite3 &> /dev/null && [[ -f ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb ]]; then
        # First try to find model in inlineDiffsData (most likely location)
        model_from_db=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM cursorDiskKV WHERE key = 'inlineDiffsData'" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | cut -d'"' -f4)
        
        if [[ -n "$model_from_db" ]]; then
            if [[ "$model_from_db" != *"-thinking" ]]; then
                detected_model="$model_from_db"
                source="inlineDiffsData"
                echo "$detected_model|$source"
                return
            fi
        fi
        
        # Try each composerData entry (there are many)
        model_from_composer=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM cursorDiskKV WHERE key LIKE 'composerData%' LIMIT 10" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | head -1 | cut -d'"' -f4)
            
        if [[ -n "$model_from_composer" ]]; then
            if [[ "$model_from_composer" != *"-thinking" ]]; then
                detected_model="$model_from_composer"
                source="composerData"
                echo "$detected_model|$source"
                return
            fi
        fi
        
        # Try to check if present in any of the data with a broader search
        any_model=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
            "SELECT hex(value) FROM ItemTable" | xxd -r -p | grep -o '"composerModel":"[^"]*"' | head -1 | cut -d'"' -f4)
            
        if [[ -n "$any_model" ]]; then
            # If the first hit is a "-thinking" variant, try to find a non-thinking alternative
            if [[ "$any_model" == *"-thinking" ]]; then
                alt_model=$(sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
                    "SELECT hex(value) FROM ItemTable" | xxd -r -p | grep -o '"composerModel":"[^\"]*"' | cut -d'"' -f4 | grep -v -- -thinking | head -1)
                if [[ -n "$alt_model" ]]; then
                    any_model="$alt_model"
                fi
            fi
            detected_model="$any_model"
            source="general search"
            echo "$detected_model|$source"
            return
        fi
    fi
    
    # Default for Claude CLI if all else fails
    # This ensures we never return N/A in Claude environment
    if [[ "$SHELL" == *"claude"* || -f /.dockerenv && -n "$ANTHROPIC_API_KEY" ]]; then
        detected_model="claude-3-7-sonnet-20250219"
        source="shell environment fallback"
        echo "$detected_model|$source"
        return
    fi
    
    detected_model="N/A"
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