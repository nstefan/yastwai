#!/bin/bash
# Unit tests for the ai-clippy.sh script
# Run with: bash tests/scripts/test_ai_clippy.sh

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
CLIPPY_SCRIPT="$PROJECT_ROOT/scripts/ai-clippy.sh"

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

# Mock cargo commands for testing
mock_cargo_commands() {
    # Create temporary directory for mock
    TEST_TEMP_DIR=$(mktemp -d)
    
    # Create mock cargo script
    cat > "$TEST_TEMP_DIR/cargo" << EOF
#!/bin/bash
echo "Mock cargo called with: \$@"
if [[ "\$*" == *"--unknown-option"* ]]; then
  exit 1
else
  exit 0
fi
EOF
    
    # Make it executable
    chmod +x "$TEST_TEMP_DIR/cargo"
    
    # Add to path
    export PATH="$TEST_TEMP_DIR:$PATH"
    
    echo "Mock cargo environment set up in $TEST_TEMP_DIR"
}

# Clean up mock environment
cleanup_mock() {
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
        echo "Mock environment cleaned up"
    fi
}

# Check if script exists
if [ ! -f "$CLIPPY_SCRIPT" ]; then
    echo -e "${RED}Error: Script $CLIPPY_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$CLIPPY_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $CLIPPY_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$CLIPPY_SCRIPT"
fi

# Test 1: Help option
run_test "Help option" "$CLIPPY_SCRIPT --help" 1

# Set up mock cargo for the following tests
mock_cargo_commands

# Test 2: Default behavior (check only)
run_test "Default behavior (check only)" "$CLIPPY_SCRIPT"

# Test 3: Check only mode
run_test "Check only mode" "$CLIPPY_SCRIPT --check-only"

# Test 4: Fix mode
run_test "Fix mode" "$CLIPPY_SCRIPT --fix"

# Test 5: Verbose mode
run_test "Verbose mode" "$CLIPPY_SCRIPT --verbose"

# Test 6: Combined modes
run_test "Combined modes" "$CLIPPY_SCRIPT --check-only --fix --verbose"

# Test 7: Unknown option
run_test "Unknown option" "$CLIPPY_SCRIPT --unknown-option" 1

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