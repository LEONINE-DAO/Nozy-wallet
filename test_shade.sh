#!/bin/bash
# Test script for Shade Protocol integration

echo "=========================================="
echo "Shade Protocol Integration Tests"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -n "Testing: $test_name... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test 1: Check if code compiles
echo "1. Compilation Tests"
echo "-------------------"
run_test "Code compilation" "cargo check --lib"
run_test "Binary compilation" "cargo build --bin nozy"
echo ""

# Test 2: Check if CLI commands are accessible
echo "2. CLI Command Tests"
echo "-------------------"
run_test "Shade command exists" "cargo run --bin nozy -- shade --help"
run_test "List tokens command" "cargo run --bin nozy -- shade list-tokens"
echo ""

# Test 3: Module structure
echo "3. Module Structure Tests"
echo "-------------------"
run_test "Secret module exists" "test -f src/secret/mod.rs"
run_test "Secret keys module exists" "test -f src/secret_keys.rs"
run_test "SNIP-20 module exists" "test -f src/secret/snip20.rs"
run_test "Transaction module exists" "test -f src/secret/transaction.rs"
echo ""

# Test 4: Documentation
echo "4. Documentation Tests"
echo "-------------------"
run_test "Integration docs exist" "test -f SHADE_INTEGRATION.md"
run_test "Notes exist" "test -f SHADE_INTEGRATION_NOTES.md"
run_test "Checklist exists" "test -f GITHUB_PUSH_CHECKLIST.md"
echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review.${NC}"
    exit 1
fi
