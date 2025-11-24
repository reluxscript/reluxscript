#!/bin/bash

passed=0
failed=0
total=0

echo "Testing parser on all .rsc files..."
echo "=================================="
echo ""

while IFS= read -r file; do
    total=$((total + 1))
    echo -n "[$total] Testing: $file ... "

    if output=$(cargo run --no-default-features --bin reluxscript -- parse "$file" 2>&1); then
        if echo "$output" | grep -q "Parse error"; then
            echo "FAILED"
            echo "$output" | grep "Parse error" | head -1
            failed=$((failed + 1))
        else
            echo "PASSED"
            passed=$((passed + 1))
        fi
    else
        if echo "$output" | grep -q "Parse error"; then
            echo "FAILED"
            echo "$output" | grep "Parse error" | head -1
            failed=$((failed + 1))
        else
            echo "ERROR (non-parse error)"
            echo "$output" | tail -3
            failed=$((failed + 1))
        fi
    fi
done < /tmp/rsc_files.txt

echo ""
echo "=================================="
echo "Results: $passed passed, $failed failed out of $total total"
echo "Success rate: $(echo "scale=1; $passed * 100 / $total" | bc)%"
