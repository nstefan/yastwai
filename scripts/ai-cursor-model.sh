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
    echo "  --quiet, -q          - Quiet output (model only)"
    echo "  --deep               - Enable deep scan (slower)"
    echo "  --refresh            - Ignore cache and re-detect"
    echo "  --ttl <seconds>      - Cache TTL in seconds (default: 60)"
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

# Removed CLAUDE_CODE_CLI shortcuts and hardcoded model outputs

# Removed early trust of MODEL_NAME to avoid stale/incorrect values; rely on detection logic below

# Parse options
VERBOSE_MODE="false"
DO_DEEP="false"
REFRESH_CACHE="false"
CACHE_TTL="${AI_CURSOR_MODEL_CACHE_TTL:-600}"
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
        --deep)
            DO_DEEP="true"
            shift
            ;;
        --refresh)
            REFRESH_CACHE="true"
            shift
            ;;
        --ttl)
            shift
            CACHE_TTL="$1"
            shift
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

# Cache fast-path (skip detection if recent)
if [[ "$REFRESH_CACHE" != "true" ]]; then
    CACHE_DIR="$HOME/.cache/yastwai"
    CACHE_FILE="$CACHE_DIR/ai-cursor-model"
    if [[ -f "$CACHE_FILE" ]]; then
        NOW_TS=$(date +%s)
        MTIME_TS=$( (stat -f %m "$CACHE_FILE" 2>/dev/null || stat -c %Y "$CACHE_FILE" 2>/dev/null) ) || MTIME_TS=$NOW_TS
        AGE=$(( NOW_TS - MTIME_TS ))
        if [[ "$AGE" -le "$CACHE_TTL" ]]; then
            cat "$CACHE_FILE"
            exit 0
        fi
    fi
fi

# Function to detect the current model - search for model information
detect_model() {
    local detected_model=""
    local source=""
    
    # Removed CLAUDE CLI environment heuristics and hardcoded fallbacks
    
    # Check environment variable (useful for testing or when other detection methods fail)
    if [[ -n "$CLAUDE_MODEL" ]]; then
        detected_model="$CLAUDE_MODEL"
        source="environment variable"
        echo "$detected_model|$source"
        return
    fi
    
    # Removed API environment-based hardcoded model fallback
    
    # Fast path: quickly scan a very small, most-recent subset of composerData for a modelName
    local DB_PATH_FAST="$HOME/Library/Application Support/Cursor/User/globalStorage/state.vscdb"
    if command -v sqlite3 &> /dev/null && [[ -f "$DB_PATH_FAST" ]] && [[ -z "$AI_CURSOR_MODEL_DISABLE_SQLITE" ]]; then
        # Dynamically reduce scan size for large DBs
        local DB_SIZE
        DB_SIZE=$( (stat -f %z "$DB_PATH_FAST" 2>/dev/null || stat -c %s "$DB_PATH_FAST" 2>/dev/null) || echo 0 )
        local FAST_LIMIT=60
        if [[ "$DB_SIZE" -ge 314572800 ]]; then # ~300MB
            FAST_LIMIT=20
        fi

        # Common SQLite safety/perf flags
        local SQLITE_FLAGS=( -readonly )
        local SQL_PREFIX="PRAGMA query_only=ON;"
        local SQL_RECENT="SELECT hex(value) FROM cursorDiskKV WHERE key GLOB 'composerData:*' ORDER BY rowid DESC LIMIT ${FAST_LIMIT};"

        # Try modelName first (Cursor commonly stores it under this key)
        local fast_from_composer
        fast_from_composer=$(LC_ALL=C sqlite3 "${SQLITE_FLAGS[@]}" "$DB_PATH_FAST" "${SQL_PREFIX} ${SQL_RECENT}" | xxd -r -p | grep -m 1 -ao '"modelName":"[^\"]*"' | cut -d '"' -f4 || true)
        if [[ -n "$fast_from_composer" ]]; then
            if [[ "$fast_from_composer" != *"-thinking" ]]; then
                detected_model="$fast_from_composer"; source="db-fast(composerData:modelName)"; echo "$detected_model|$source"; return
            fi
        fi
        # Fallback: composerModel field
        local fast_comp_model
        fast_comp_model=$(LC_ALL=C sqlite3 "${SQLITE_FLAGS[@]}" "$DB_PATH_FAST" "${SQL_PREFIX} ${SQL_RECENT}" | xxd -r -p | grep -m 1 -ao '"composerModel":"[^\"]*"' | cut -d '"' -f4 || true)
        if [[ -n "$fast_comp_model" ]]; then
            if [[ "$fast_comp_model" != *"-thinking" ]]; then
                detected_model="$fast_comp_model"; source="db-fast(composerData:composerModel)"; echo "$detected_model|$source"; return
            fi
        fi
    fi

    # Deep scan (disabled by default)
    if [[ "$DO_DEEP" == "true" ]] && command -v sqlite3 &> /dev/null && [[ -f "$HOME/Library/Application Support/Cursor/User/globalStorage/state.vscdb" ]]; then
        DB_PATH="$HOME/Library/Application Support/Cursor/User/globalStorage/state.vscdb"
        # Aggregate all model-like fields from both cursorDiskKV and ItemTable
        ALL_DATA=$( { sqlite3 "$DB_PATH" "SELECT hex(value) FROM cursorDiskKV" | xxd -r -p; sqlite3 "$DB_PATH" "SELECT hex(value) FROM ItemTable" | xxd -r -p; } 2>/dev/null )
        CANDIDATES=$(echo "$ALL_DATA" | grep -E -ao '"(composerModel|chatModel|completionModel|modelName|selectedModel|model)"\s*:\s*"[^"]*"' | cut -d '"' -f4)

        if [[ -n "$CANDIDATES" ]]; then
            # Prefer the most frequent model name in the DB
            BEST=$(echo "$CANDIDATES" | grep -v '^$' | sort | uniq -c | sort -nr | awk '{print $2}' | head -n 1)
            # Avoid returning "-thinking" variants if possible
            if [[ "$BEST" == *"-thinking" ]]; then
                ALT=$(echo "$CANDIDATES" | grep -v -- -thinking | sort | uniq -c | sort -nr | awk '{print $2}' | head -n 1)
                if [[ -n "$ALT" ]]; then
                    BEST="$ALT"
                fi
            fi
            if [[ -n "$BEST" ]]; then
                detected_model="$BEST"; source="db-aggregated"; echo "$detected_model|$source"; return
            fi
        fi
    fi
    
    # No hardcoded model fallbacks; return N/A if undetectable
    
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

# Write cache best-effort
{ mkdir -p "$HOME/.cache/yastwai" 2>/dev/null && echo "$MODEL" > "$HOME/.cache/yastwai/ai-cursor-model"; } 2>/dev/null || true

exit 0


