#!/bin/bash
# Unit tests for the ai-cursor-model.sh script
# Run with: bash tests/scripts/test_ai_cursor_model.sh

# Colors for test output
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

# Initialize test counts
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
MODEL_SCRIPT="$PROJECT_ROOT/scripts/ai-cursor-model.sh"

# Helper function to run tests
run_test() {
    local name="$1"
    local command="$2"
    local expected_exit_code="${3:-0}"
    
    echo -e "${YELLOW}Running test: $name${NC}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    # Create temp file for output
    local temp_file=$(mktemp)
    
    # Run the command and capture output and exit code
    eval "$command" > "$temp_file" 2>&1
    local actual_exit_code=$?
    
    if [ $actual_exit_code -eq $expected_exit_code ]; then
        echo -e "${GREEN}✓ Test passed: Command exited with expected code $expected_exit_code${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ Test failed: Command exited with code $actual_exit_code, expected $expected_exit_code${NC}"
        echo -e "${RED}Output:${NC}"
        cat "$temp_file"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    # Clean up
    rm -f "$temp_file"
    echo ""
}

# Check if script exists
if [ ! -f "$MODEL_SCRIPT" ]; then
    echo -e "${RED}Error: Script $MODEL_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$MODEL_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $MODEL_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$MODEL_SCRIPT"
fi

# Test 1: Basic script execution
run_test "Basic script execution" "$MODEL_SCRIPT"

# Test 2: Help option
run_test "Help option" "$MODEL_SCRIPT --help" 1

# Test 3: Verbose mode
run_test "Verbose mode" "$MODEL_SCRIPT --verbose"

# Test 4: Quiet mode
run_test "Quiet mode" "$MODEL_SCRIPT --quiet"

# Test 5: Unknown option
run_test "Unknown option" "$MODEL_SCRIPT --unknown-option" 1

# Test 6: Output format check
run_test "Output format check" "model=\$($MODEL_SCRIPT --quiet) && [[ \$model != \"\" ]]"

# Summary
echo -e "${YELLOW}=== Test Summary ===${NC}"
echo -e "Total tests: $TESTS_TOTAL"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
    exit 1
else
    echo -e "Failed: $TESTS_FAILED"
    echo -e "${GREEN}All tests passed successfully!${NC}"
    exit 0
fi 