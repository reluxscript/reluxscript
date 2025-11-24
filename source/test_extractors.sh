#!/bin/bash
# Test all extractor files

passed=0
failed=0
parse_errors=0

echo "Testing extractor files..."
echo ""

for file in tests/codegen/extractors/**/*.rsc; do
    result=$(cargo run --quiet -- check "$file" 2>&1 | grep -E "Check passed|Parse error|Check failed")

    if echo "$result" | grep -q "Check passed"; then
        ((passed++))
        echo "✓ $file"
    elif echo "$result" | grep -q "Parse error"; then
        ((parse_errors++))
        echo "✗ Parse: $file"
    elif echo "$result" | grep -q "Check failed"; then
        ((failed++))
        echo "✗ Semantic: $file"
    fi
done

echo ""
echo "Results:"
echo "  Passed: $passed"
echo "  Parse errors: $parse_errors"
echo "  Semantic errors: $failed"
echo "  Total: $((passed + parse_errors + failed))"
