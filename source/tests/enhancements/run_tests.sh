#!/bin/bash
# Run all enhancement tests to verify they fail (TDD red phase)
# Each test should fail until the corresponding feature is implemented

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUSTSCRIPT_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
TEST_DIR="$SCRIPT_DIR"

echo "================================"
echo "ReluxScript Enhancement Tests"
echo "================================"
echo ""
echo "Running TDD red phase - all tests should FAIL"
echo ""

# Track results
PASSED=0
FAILED=0

run_test() {
    local test_file="$1"
    local test_name="$(basename "$test_file" .rsc)"

    echo "Testing: $test_name"
    echo "----------------------------------------"

    if cargo run --manifest-path "$RUSTSCRIPT_DIR/Cargo.toml" -- check "$test_file" 2>&1; then
        echo "  UNEXPECTED PASS - Feature may already be implemented!"
        ((PASSED++))
    else
        echo "  EXPECTED FAIL - Feature not yet implemented"
        ((FAILED++))
    fi
    echo ""
}

# Run each test
run_test "$TEST_DIR/nested_traverse.rsc"
run_test "$TEST_DIR/mutable_refs.rsc"
run_test "$TEST_DIR/hashmap_hashset.rsc"
run_test "$TEST_DIR/string_formatting.rsc"
run_test "$TEST_DIR/file_io.rsc"
run_test "$TEST_DIR/json_serialization.rsc"

echo "================================"
echo "Summary"
echo "================================"
echo "Expected failures (features needed): $FAILED"
echo "Unexpected passes: $PASSED"
echo ""

if [ $FAILED -eq 6 ]; then
    echo "All tests failed as expected - TDD red phase complete!"
    echo "Now implement features to make tests pass."
else
    echo "Some tests passed unexpectedly - check implementation status."
fi
