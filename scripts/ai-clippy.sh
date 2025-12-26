#!/bin/bash
# AI Assistant Helper Script for Running Clippy
# This script helps AI assistants run Rust linting checks in a non-interactive way
# Follows the naming pattern of ai-*.sh for consistency

set -e  # Exit on error

# Function to show usage
show_usage() {
    echo "Usage: ./scripts/ai-clippy.sh [options]"
    echo ""
    echo "Options:"
    echo "  --check-only         - Only run checks without attempting to fix issues"
    echo "  --fix                - Run Clippy with auto-fix capability"
    echo "  --verbose            - Display more detailed output"
    echo "  --help               - Display this help message"
    exit 1
}

# Helper function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Default values
CHECK_ONLY=false
FIX=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --fix)
            FIX=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            log_message "Unknown option: $1"
            show_usage
            ;;
    esac
done

# If neither flag is specified, default to running both check and fix
if [ "$CHECK_ONLY" = false ] && [ "$FIX" = false ]; then
    CHECK_ONLY=true
    log_message "No specific mode selected, defaulting to check-only mode"
fi

# Run standard Clippy check
if [ "$CHECK_ONLY" = true ]; then
    log_message "Running Clippy checks..."
    
    # Common lint exceptions for the project
    LINTS="-A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls"
    
    if [ "$VERBOSE" = true ]; then
        log_message "Using lint exceptions: $LINTS"
    fi
    
    # Run Clippy with warnings as errors and redirect through cat to avoid interactive pager
    cargo clippy -- -D warnings $LINTS 2>&1 | cat
    
    clippy_exit_code=${PIPESTATUS[0]}
    if [ $clippy_exit_code -ne 0 ]; then
        log_message "Clippy check failed with exit code $clippy_exit_code"
        exit $clippy_exit_code
    else
        log_message "Clippy check passed successfully."
    fi
fi

# Run auto-fix if requested
if [ "$FIX" = true ]; then
    log_message "Running Clippy auto-fix..."
    
    # Run cargo fix with options that work for non-interactive environments
    cargo fix --allow-dirty --allow-staged 2>&1 | cat
    
    fix_exit_code=${PIPESTATUS[0]}
    if [ $fix_exit_code -ne 0 ]; then
        log_message "Clippy auto-fix failed with exit code $fix_exit_code"
        exit $fix_exit_code
    else
        log_message "Clippy auto-fix completed successfully."
    fi
    
    # Also run cargo clippy --fix for more specific fixes
    log_message "Running additional clippy fixes..."
    cargo clippy --fix --allow-dirty --allow-staged 2>&1 | cat
    
    clippy_fix_exit_code=${PIPESTATUS[0]}
    if [ $clippy_fix_exit_code -ne 0 ]; then
        log_message "Additional clippy fixes completed with warnings, code: $clippy_fix_exit_code"
    else
        log_message "Additional clippy fixes completed successfully."
    fi
fi

log_message "Clippy process completed."
exit 0 