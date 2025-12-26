#!/bin/bash
# Unit tests for the ai-branch.sh script
# Run with: bash tests/scripts/test_ai_branch.sh

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
BRANCH_SCRIPT="$PROJECT_ROOT/scripts/ai-branch.sh"

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
if [[ "\$*" == *"branch --show-current"* ]]; then
    echo "mock-branch"
elif [[ "\$*" == *"status --porcelain"* ]]; then
    # If force flag is set, return empty (no changes)
    if [[ \$MOCK_FORCE == "true" ]]; then
        echo ""
    else
        echo "M  file.txt"
    fi
elif [[ "\$*" == *"checkout -b"* ]]; then
    echo "Switched to a new branch '\$(echo \$* | grep -o "checkout -b [^ ]*" | cut -d ' ' -f 3)'"
    exit 0
elif [[ "\$*" == *"checkout"* ]]; then
    echo "Switched to branch '\$(echo \$* | grep -o "checkout [^ ]*" | cut -d ' ' -f 2)'"
    exit 0
elif [[ "\$*" == *"pull"* ]]; then
    echo "Already up to date."
    exit 0
else
    echo "Mock git called with: \$@"
    exit 0
fi
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
if [ ! -f "$BRANCH_SCRIPT" ]; then
    echo -e "${RED}Error: Script $BRANCH_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$BRANCH_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $BRANCH_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$BRANCH_SCRIPT"
fi

# Set up mock git for all tests
mock_git_commands

# Test 1: Help option
run_test "Help option" "$BRANCH_SCRIPT --help" 1

# Test 2: Check only mode
run_test "Check only mode" "$BRANCH_SCRIPT --check-only"

# Test 3: Missing branch name
run_test "Missing branch name in new-branch option" "$BRANCH_SCRIPT --new-branch" 1

# Test 4: Invalid is-related value
run_test "Invalid is-related value" "$BRANCH_SCRIPT --is-related maybe" 1

# Test 5: Check if new branch works
run_test "Create new branch" "$BRANCH_SCRIPT --new-branch test-branch"

# Test 6: Work related to current branch
run_test "Work related to current branch" "$BRANCH_SCRIPT --is-related true --new-branch test-branch"

# Test 7: Work not related to current branch (should fail with uncommitted changes)
run_test "Work not related to current branch with uncommitted changes" "$BRANCH_SCRIPT --is-related false --new-branch test-branch" 1

# Test 8: Work not related to current branch with force
export MOCK_FORCE="true"
run_test "Work not related to current branch with force" "$BRANCH_SCRIPT --is-related false --new-branch test-branch --force"
unset MOCK_FORCE

# Test 9: Unknown option
run_test "Unknown option" "$BRANCH_SCRIPT --unknown-option" 1

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