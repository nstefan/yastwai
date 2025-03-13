#!/bin/bash
# Unit tests for the ai-commit.sh script
# Run with: bash tests/scripts/test_ai_commit.sh

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
COMMIT_SCRIPT="$PROJECT_ROOT/scripts/ai-commit.sh"

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

# Mock git commands for testing
mock_git_commands() {
    # Create temporary directory for mock
    TEST_TEMP_DIR=$(mktemp -d)
    
    # Create mock git script
    cat > "$TEST_TEMP_DIR/git" << EOF
#!/bin/bash
echo "Mock git called with: \$@"
exit 0
EOF
    
    # Make it executable
    chmod +x "$TEST_TEMP_DIR/git"
    
    # Add to path
    export PATH="$TEST_TEMP_DIR:$PATH"
    
    echo "Mock git environment set up in $TEST_TEMP_DIR"
}

# Clean up mock environment
cleanup_mock() {
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
        echo "Mock environment cleaned up"
    fi
}

# Check if script exists
if [ ! -f "$COMMIT_SCRIPT" ]; then
    echo -e "${RED}Error: Script $COMMIT_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$COMMIT_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $COMMIT_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$COMMIT_SCRIPT"
fi

# Test 1: Help option
run_test "Help option" "$COMMIT_SCRIPT --help" 1

# Set up mock git for the following tests
mock_git_commands

# Test 2: No arguments
run_test "No arguments" "$COMMIT_SCRIPT" 1

# Test 3: Missing model parameter
run_test "Missing model parameter" "$COMMIT_SCRIPT \"Test title\" \"Test description\" \"Test prompt\"" 1

# Test 4: Valid command with minimal parameters
run_test "Valid command with minimal parameters" "$COMMIT_SCRIPT --model \"test-model\" \"Test title\" \"Test description\" \"Test prompt\"" 0

# Test 5: Valid command with all parameters
run_test "Valid command with all parameters" "$COMMIT_SCRIPT --model \"test-model\" \"Test title\" \"Test description\" \"Test prompt\" \"Test thought process\" \"Test discussion\"" 0

# Test 6: Invalid model parameter
run_test "Invalid model parameter" "$COMMIT_SCRIPT --model" 1

# Clean up mock environment
cleanup_mock

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