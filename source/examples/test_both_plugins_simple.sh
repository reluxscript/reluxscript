#!/bin/bash
# Simplified test script - just tests Babel output for now
# SWC testing requires more complex setup

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ $# -lt 2 ]; then
    echo "Usage: $0 <plugin.lux> <input.js> [expected_output.js]"
    exit 1
fi

PLUGIN_FILE="$1"
INPUT_FILE="$2"
EXPECTED_OUTPUT="${3:-}"

BABEL_PLUGIN="dist/index.js"
BABEL_OUTPUT="babel_output.js"

echo -e "${YELLOW}=== Testing ReluxScript Babel Plugin ===${NC}"
echo "Plugin: $PLUGIN_FILE"
echo "Input: $INPUT_FILE"
echo ""

# Step 1: Compile
echo -e "${YELLOW}[1/3] Compiling $PLUGIN_FILE...${NC}"
../target/release/relux build "$PLUGIN_FILE" 2>&1 | grep -E "(Generated|✓|Build complete)" || true

if [ ! -f "$BABEL_PLUGIN" ]; then
    echo -e "${RED}Error: Babel plugin not generated${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Compilation successful${NC}"
echo ""

# Step 2: Run Babel
echo -e "${YELLOW}[2/3] Running Babel transform...${NC}"
node << 'NODEJS'
const babel = require('@babel/core');
const fs = require('fs');
const plugin = require('./dist/index.js');

const input = fs.readFileSync(process.argv[1], 'utf8');
const result = babel.transformSync(input, {
    plugins: [plugin],
    filename: process.argv[1],
    configFile: false,
    babelrc: false
});

fs.writeFileSync(process.argv[2], result.code);
NODEJS "$INPUT_FILE" "$BABEL_OUTPUT"

echo -e "${GREEN}✓ Babel transform complete${NC}"
echo ""

# Step 3: Compare with expected if provided
if [ -n "$EXPECTED_OUTPUT" ] && [ -f "$EXPECTED_OUTPUT" ]; then
    echo -e "${YELLOW}[3/3] Comparing with expected output...${NC}"

    if diff -w "$EXPECTED_OUTPUT" "$BABEL_OUTPUT" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Output matches expected!${NC}"
    else
        echo -e "${RED}✗ Output doesn't match expected${NC}"
        echo ""
        echo "Expected:"
        cat "$EXPECTED_OUTPUT"
        echo ""
        echo "Got:"
        cat "$BABEL_OUTPUT"
        echo ""
        echo "Diff:"
        diff -u "$EXPECTED_OUTPUT" "$BABEL_OUTPUT" || true
        exit 1
    fi
else
    echo -e "${YELLOW}[3/3] No expected output provided${NC}"
fi

echo ""
echo -e "${GREEN}=== Test passed! ===${NC}"
echo "Output: $BABEL_OUTPUT"
