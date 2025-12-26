#!/bin/bash
# Unit tests for the ai-readme.sh script
# Run with: bash tests/scripts/test_ai_readme.sh

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
README_SCRIPT="$PROJECT_ROOT/scripts/ai-readme.sh"

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

# Setup test environment
setup_test_environment() {
    # Create temporary test directory
    TEST_ENV_DIR=$(mktemp -d)
    
    # Create minimal project structure
    mkdir -p "$TEST_ENV_DIR/scripts"
    mkdir -p "$TEST_ENV_DIR/docs"
    mkdir -p "$TEST_ENV_DIR/src"
    
    # Create mock Cargo.toml
    cat > "$TEST_ENV_DIR/Cargo.toml" << EOF
[package]
name = "test-project"
version = "0.1.0"
authors = ["Test Author <test@example.com>"]
edition = "2021"
rust-version = "1.85.0"
license = "MIT"
description = "A test project for README generation"

[dependencies]
tokio = { version = "1.36.0", features = ["full"] }
anyhow = "1.0.79"
serde = { version = "1.0.193", features = ["derive"] }
serde_json = "1.0.108"
EOF
    
    # Create app_config.rs with TranslationProvider enum
    mkdir -p "$TEST_ENV_DIR/src"
    cat > "$TEST_ENV_DIR/src/app_config.rs" << EOF
pub enum TranslationProvider {
    #[default]
    Ollama,
    OpenAI,
    Anthropic,
}

impl TranslationProvider {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ollama => "Ollama",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
        }
    }
}
EOF
    
    # Create example config
    cat > "$TEST_ENV_DIR/conf.example.json" << EOF
{
  "source_language": "en",
  "target_language": "fr",
  "translation": {
    "provider": "ollama"
  }
}
EOF
    
    # Copy the real README script to the test environment
    cp "$README_SCRIPT" "$TEST_ENV_DIR/scripts/ai-readme.sh"
    chmod +x "$TEST_ENV_DIR/scripts/ai-readme.sh"
    
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
if [ ! -f "$README_SCRIPT" ]; then
    echo -e "${RED}Error: Script $README_SCRIPT does not exist${NC}"
    exit 1
fi

# Check if script is executable
if [ ! -x "$README_SCRIPT" ]; then
    echo -e "${YELLOW}Warning: Script $README_SCRIPT is not executable. Making it executable...${NC}"
    chmod +x "$README_SCRIPT"
fi

# Test 1: Script exists
run_test "Script exists" "[ -f \"$README_SCRIPT\" ]"

# Test 2: Help option
run_test "Help option" "$README_SCRIPT --help" 0

# Test 3: Unknown option
run_test "Unknown option" "$README_SCRIPT --unknown-option" 1

# Test 4: Dry run mode
run_test "Dry run mode" "$README_SCRIPT --dry-run" 0

# Test 5: Quiet mode
run_test "Quiet mode" "$README_SCRIPT --quiet --dry-run" 0

# Test 6: Setup test environment for file generation
setup_test_environment

# Test 7: Generate README in test environment (dry run)
run_test "Generate README in test environment (dry run)" "cd $TEST_ENV_DIR && ./scripts/ai-readme.sh --dry-run"

# Test 8: Generate README in test environment
run_test "Generate README in test environment" "cd $TEST_ENV_DIR && ./scripts/ai-readme.sh"

# Test 9: Verify README was created
run_test "Verify README was created" "[ -f \"$TEST_ENV_DIR/README.md\" ]"

# Test 10: Check README content - use cat to output the first line which should contain the project name
run_test "Check README content" "cat \"$TEST_ENV_DIR/README.md\" | grep -q \"YASTwAI\""

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