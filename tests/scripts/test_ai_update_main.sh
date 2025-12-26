#!/bin/bash
# Unit tests for the ai-update-main.sh script
# Run with: bash tests/scripts/test_ai_update_main.sh
#
# IMPORTANT SAFETY NOTE:
# This test script uses a mock git command to intercept all git operations.
# It should NEVER execute actual git commands against the real repository.
# If you modify this test, ensure the mocking approach remains intact to prevent
# accidental repository modifications.
#
# INTERACTIVE COMMAND WARNING:
# Always pipe git commands that might trigger a pager (log, diff, show, etc.)
# through 'cat' to prevent hanging on interactive prompts, e.g.: git log | cat
# This is critical for automated scripts and tests to run without interruption.

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
UPDATE_SCRIPT="$PROJECT_ROOT/scripts/ai-update-main.sh"

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
    echo "\$MOCK_CURRENT_BRANCH"
elif [[ "\$*" == *"status --porcelain"* ]]; then
    # If MOCK_CLEAN is set, return empty (no changes)
    if [[ \$MOCK_CLEAN == "true" ]]; then
        echo ""
    else
        echo "M  file.txt"
    fi
elif [[ "\$*" == *"rev-parse main"* ]]; then
    echo "local-main-rev"
elif [[ "\$*" == *"rev-parse origin/main"* ]]; then
    if [[ \$MOCK_MAIN_UPDATED == "true" ]]; then
        echo "local-main-rev" # Same as local
    else
        echo "remote-main-rev" # Different from local
    fi
elif [[ "\$*" == *"checkout"* ]]; then
    echo "Switched to branch '\$(echo \$* | grep -o "checkout [^ ]*" | cut -d ' ' -f 2)'"
    exit 0
elif [[ "\$*" == *"fetch"* ]]; then
    echo "Fetching origin"
    exit 0
elif [[ "\$*" == *"pull --rebase"* ]]; then
    if [[ \$MOCK_PULL_FAIL == "true" ]]; then
        echo "Failed to pull with rebase"
        exit 1
    else
        echo "Successfully pulled with rebase"
        exit 0
    fi
elif [[ "\$*" == *"rebase"* ]]; then
    if [[ \$MOCK_REBASE_FAIL == "true" ]]; then
        echo "Failed to rebase current branch"
        exit 1
    else
        echo "Successfully rebased"
        exit 0
    fi
elif [[ "\$*" == *"log"* ]]; then
    echo "abc1234 Commit message 1"
    echo "def5678 Commit message 2"
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
    
    # Make extra sure we're using the system git when we exit
    hash -r 2>/dev/null || true
}

# Set up trap to ensure cleanup even if the script is interrupted
trap cleanup_mock EXIT SIGINT SIGTERM

# Check if script exists
if [ ! -f "$UPDATE_SCRIPT" ]; then
    echo -e "${RED}Error: Script $UPDATE_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$UPDATE_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $UPDATE_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$UPDATE_SCRIPT"
fi

# Set up mock git for all tests
mock_git_commands

# Verify that our mock git is being used
WHICH_GIT=$(which git)
if [[ "$WHICH_GIT" != "$TEST_TEMP_DIR/git" ]]; then
    echo -e "${RED}Error: Mock git is not being used! This is unsafe.${NC}"
    echo -e "${RED}Using git from: $WHICH_GIT${NC}"
    echo -e "${RED}Expected to use: $TEST_TEMP_DIR/git${NC}"
    cleanup_mock
    exit 1
fi
echo -e "${GREEN}✓ Verified: Using mock git command from $WHICH_GIT${NC}"

# Set default mock environment
export MOCK_CURRENT_BRANCH="feature-branch"
export MOCK_CLEAN="true"
export MOCK_MAIN_UPDATED="false"

# Test 1: Help option
run_test "Help option" "$UPDATE_SCRIPT --help" 1

# Test 2: Check only mode - updates available
run_test "Check only mode - updates available" "$UPDATE_SCRIPT --check-only"

# Test 3: Check only mode - no updates available
export MOCK_MAIN_UPDATED="true"
run_test "Check only mode - no updates available" "$UPDATE_SCRIPT --check-only"
export MOCK_MAIN_UPDATED="false"

# Test 4: Update main with uncommitted changes
export MOCK_CLEAN="false"
run_test "Update main with uncommitted changes" "$UPDATE_SCRIPT" 1
export MOCK_CLEAN="true"

# Test 5: Update main successfully
run_test "Update main successfully" "$UPDATE_SCRIPT"

# Test 6: Update main with pull failure
export MOCK_PULL_FAIL="true"
run_test "Update main with pull failure" "$UPDATE_SCRIPT" 1
unset MOCK_PULL_FAIL

# Test 7: Rebase current branch option
run_test "Rebase current branch option" "$UPDATE_SCRIPT --rebase-current"

# Test 8: Rebase current branch with rebase failure
export MOCK_REBASE_FAIL="true"
run_test "Rebase current branch with rebase failure" "$UPDATE_SCRIPT --rebase-current" 1
unset MOCK_REBASE_FAIL

# Test 9: On main branch
export MOCK_CURRENT_BRANCH="main"
run_test "On main branch" "$UPDATE_SCRIPT"
export MOCK_CURRENT_BRANCH="feature-branch"

# Test 10: Unknown option
run_test "Unknown option" "$UPDATE_SCRIPT --unknown-option" 1

# Clean up mock environment
cleanup_mock
unset MOCK_CURRENT_BRANCH
unset MOCK_CLEAN
unset MOCK_MAIN_UPDATED

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