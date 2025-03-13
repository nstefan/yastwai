#!/bin/bash
# Unit tests for the ai-pr.sh script
# Run with: bash tests/scripts/test_ai_pr.sh

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
PR_SCRIPT="$PROJECT_ROOT/scripts/ai-pr.sh"

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

# Mock git and python commands for testing
mock_commands() {
    # Create temporary directory for mock
    TEST_TEMP_DIR=$(mktemp -d)
    
    # Create mock git script
    cat > "$TEST_TEMP_DIR/git" << EOF
#!/bin/bash
if [[ "\$*" == *"branch --show-current"* ]]; then
    echo "mock-branch"
elif [[ "\$*" == *"status --porcelain"* ]]; then
    echo ""  # No uncommitted changes
elif [[ "\$*" == *"ls-remote --heads"* ]]; then
    echo "1234567890abcdef1234567890abcdef1234 refs/heads/mock-branch"
elif [[ "\$*" == *"rev-list --count"* ]]; then
    echo "5"  # 5 commits
elif [[ "\$*" == *"config --get remote.origin.url"* ]]; then
    echo "git@github.com:user/repo.git"
else
    echo "Mock git called with: \$@"
    exit 0
fi
EOF
    
    # Create mock gh script
    cat > "$TEST_TEMP_DIR/gh" << EOF
#!/bin/bash
if [[ "\$*" == *"auth status"* ]]; then
    # Mock successful auth status
    exit 0
elif [[ "\$*" == *"pr create"* ]]; then
    echo "https://github.com/user/repo/pull/123"
    exit 0
else
    echo "Mock gh called with: \$@"
    exit 0
fi
EOF
    
    # Make them executable
    chmod +x "$TEST_TEMP_DIR/git"
    chmod +x "$TEST_TEMP_DIR/gh"
    
    # Add to path
    export PATH="$TEST_TEMP_DIR:$PATH"
    
    echo "Mock commands environment set up in $TEST_TEMP_DIR"
}

# Clean up mock environment
cleanup_mock() {
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
        echo "Mock environment cleaned up"
    fi
}

# Check if script exists
if [ ! -f "$PR_SCRIPT" ]; then
    echo -e "${RED}Error: Script $PR_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$PR_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $PR_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$PR_SCRIPT"
fi

# Set up mock environment
mock_commands

# Test 1: Help option
run_test "Help option" "$PR_SCRIPT --help" 1

# Test 2: No arguments
run_test "No arguments" "$PR_SCRIPT" 1

# Test 3: Missing required parameters
run_test "Missing required parameters (title only)" "$PR_SCRIPT --title \"Test PR\"" 1

# Test 4: Missing model parameter
run_test "Missing model parameter" "$PR_SCRIPT --title \"Test PR\" --overview \"Test overview\"" 1

# Test 5: Basic valid command
run_test "Basic valid command" "$PR_SCRIPT --title \"Test PR\" --overview \"Test overview\" --model \"test-model\" --no-browser"

# Test 6: Valid command with all parameters
run_test "Valid command with all parameters" "$PR_SCRIPT --title \"Test PR\" --overview \"Test overview\" --key-changes \"Change 1,Change 2\" --implementation \"Detail 1,Detail 2\" --model \"test-model\" --base \"main\" --draft --no-browser"

# Test 7: Invalid parameter usage
run_test "Invalid parameter usage (missing value)" "$PR_SCRIPT --title" 1

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