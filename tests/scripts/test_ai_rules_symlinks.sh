#!/bin/bash
# Unit tests for the ai-rules-symlinks.sh script
# Run with: bash tests/scripts/test_ai_rules_symlinks.sh

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
RULES_SCRIPT="$PROJECT_ROOT/scripts/ai-rules-symlinks.sh"

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

# Setup for symlink tests
setup_test_environment() {
    # Create temporary test directory
    TEST_ENV_DIR=$(mktemp -d)
    
    # Create source and target directories
    mkdir -p "$TEST_ENV_DIR/docs/agentrules"
    mkdir -p "$TEST_ENV_DIR/.cursor/rules"
    
    # Create test files
    echo "Test rule 1" > "$TEST_ENV_DIR/docs/agentrules/test1_mdc.txt"
    echo "Test rule 2" > "$TEST_ENV_DIR/docs/agentrules/test2_mdc.txt"
    echo "Test rule 3" > "$TEST_ENV_DIR/docs/agentrules/not_a_rule.txt"  # This should be ignored
    
    # Create an existing symlink to test removal
    touch "$TEST_ENV_DIR/docs/agentrules/existing_mdc.txt"
    ln -sf "$TEST_ENV_DIR/docs/agentrules/existing_mdc.txt" "$TEST_ENV_DIR/.cursor/rules/existing.mdc"
    
    echo "Test environment set up in $TEST_ENV_DIR"
    return 0
}

# Clean up test environment
cleanup_test_environment() {
    if [ -n "$TEST_ENV_DIR" ] && [ -d "$TEST_ENV_DIR" ]; then
        rm -rf "$TEST_ENV_DIR"
        echo "Test environment cleaned up"
    fi
}

# Check if script exists
if [ ! -f "$RULES_SCRIPT" ]; then
    echo -e "${RED}Error: Script $RULES_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$RULES_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $RULES_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$RULES_SCRIPT"
fi

# Set up test environment
setup_test_environment

# Test 1: Script exists
run_test "Script exists" "[ -f \"$RULES_SCRIPT\" ]"

# Test 2: Direct script execution (will operate on real repo)
run_test "Basic script execution in current repo structure" "$RULES_SCRIPT"

# Test 3: Check symlink creation (in test environment)
run_test "Symlink creation test" "cd $TEST_ENV_DIR && $RULES_SCRIPT"

# Test 4: Verify symlinks were created correctly
run_test "Verify test1.mdc symlink" "[ -L \"$TEST_ENV_DIR/.cursor/rules/test1.mdc\" ]"
run_test "Verify test2.mdc symlink" "[ -L \"$TEST_ENV_DIR/.cursor/rules/test2.mdc\" ]"

# Test 5: Verify non-matching files were ignored
run_test "Verify non-matching file was ignored" "[ ! -L \"$TEST_ENV_DIR/.cursor/rules/not_a_rule.mdc\" ]"

# Test 6: Running again should replace existing symlinks
run_test "Re-run script to replace symlinks" "cd $TEST_ENV_DIR && $RULES_SCRIPT"

# Clean up test environment
cleanup_test_environment

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