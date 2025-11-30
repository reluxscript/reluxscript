#!/bin/bash
# Test script to verify Babel and SWC plugins produce identical output

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <plugin.lux> <input.js> [expected_output.js]"
    echo "Example: $0 console_remover.lux test_input.js expected_output.js"
    exit 1
fi

PLUGIN_FILE="$1"
INPUT_FILE="$2"
EXPECTED_OUTPUT="${3:-}"

PLUGIN_NAME=$(basename "$PLUGIN_FILE" .lux)
DIST_DIR="dist"
BABEL_PLUGIN="$DIST_DIR/index.js"
SWC_PLUGIN="$DIST_DIR/lib.rs"
BABEL_OUTPUT="babel_output.js"
SWC_OUTPUT="swc_output.js"

echo -e "${YELLOW}=== Testing ReluxScript Cross-Compilation ===${NC}"
echo "Plugin: $PLUGIN_FILE"
echo "Input: $INPUT_FILE"
echo ""

# Step 1: Compile the .lux file
echo -e "${YELLOW}[1/5] Compiling $PLUGIN_FILE...${NC}"
../target/release/relux build "$PLUGIN_FILE"

if [ ! -f "$BABEL_PLUGIN" ]; then
    echo -e "${RED}Error: Babel plugin not generated${NC}"
    exit 1
fi

if [ ! -f "$SWC_PLUGIN" ]; then
    echo -e "${RED}Error: SWC plugin not generated${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Compilation successful${NC}"
echo ""

# Step 2: Run Babel transform
echo -e "${YELLOW}[2/5] Running Babel transform...${NC}"
node -e "
const babel = require('@babel/core');
const fs = require('fs');
const plugin = require('./$BABEL_PLUGIN');

const input = fs.readFileSync('$INPUT_FILE', 'utf8');
const result = babel.transformSync(input, {
    plugins: [plugin],
    filename: '$INPUT_FILE',
    configFile: false,
    babelrc: false
});

fs.writeFileSync('$BABEL_OUTPUT', result.code);
console.log('Babel output written to $BABEL_OUTPUT');
"

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Babel transform failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Babel transform complete${NC}"
echo ""

# Step 3: Run SWC transform
echo -e "${YELLOW}[3/5] Running SWC transform...${NC}"

# First, create a temporary Rust project to run the SWC plugin
SWC_TEST_DIR="swc_test_runner"
mkdir -p "$SWC_TEST_DIR/src"

cat > "$SWC_TEST_DIR/Cargo.toml" << 'EOF'
[package]
name = "swc-test-runner"
version = "0.1.0"
edition = "2021"

[dependencies]
swc_common = "17.0.1"
swc_ecma_ast = "18.0.0"
swc_ecma_visit = "18.0.1"
swc_ecma_parser = "27.0.3"
swc_ecma_codegen = "20.0.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.145"
anyhow = "1.0"
EOF

# Copy the SWC plugin into the test runner
cp "$SWC_PLUGIN" "$SWC_TEST_DIR/plugin.rs"

# Create the test runner
cat > "$SWC_TEST_DIR/src/main.rs" << 'EOF'
use swc_common::{sync::Lrc, SourceMap, FileName, DUMMY_SP};
use swc_ecma_parser::{Parser, Syntax, EsConfig, StringInput};
use swc_ecma_codegen::{Emitter, text_writer::JsWriter, Config as CodegenConfig};
use swc_ecma_visit::VisitMutWith;
use std::fs;

mod plugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_file = std::env::args().nth(1).expect("Input file required");
    let output_file = std::env::args().nth(2).expect("Output file required");

    // Read input
    let source = fs::read_to_string(&input_file)?;

    // Parse
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Custom(input_file.clone()), source);

    let syntax = Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    });

    let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);
    let mut program = parser.parse_program()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Transform
    let mut visitor = plugin::PLUGIN_STRUCT_NAME::default();
    program.visit_mut_with(&mut visitor);

    // Generate output
    let mut buf = vec![];
    {
        let mut emitter = Emitter {
            cfg: CodegenConfig::default(),
            cm: cm.clone(),
            comments: None,
            wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)),
        };
        program.emit_with(&mut emitter)?;
    }

    let output = String::from_utf8(buf)?;
    fs::write(output_file, output)?;

    Ok(())
}
EOF

# Replace PLUGIN_STRUCT_NAME with actual plugin name from the generated code
PLUGIN_STRUCT=$(grep "pub struct" "$SWC_PLUGIN" | head -1 | awk '{print $3}')
sed -i "s/PLUGIN_STRUCT_NAME/$PLUGIN_STRUCT/g" "$SWC_TEST_DIR/src/main.rs"

# Build and run the SWC test runner
cd "$SWC_TEST_DIR"
cargo build --release --quiet 2>&1 | grep -i error || true
cd ..

if [ ! -f "$SWC_TEST_DIR/target/release/swc-test-runner" ] && [ ! -f "$SWC_TEST_DIR/target/release/swc-test-runner.exe" ]; then
    echo -e "${RED}Error: SWC test runner build failed${NC}"
    exit 1
fi

# Run the SWC transform
if [ -f "$SWC_TEST_DIR/target/release/swc-test-runner.exe" ]; then
    "$SWC_TEST_DIR/target/release/swc-test-runner.exe" "$INPUT_FILE" "$SWC_OUTPUT"
else
    "$SWC_TEST_DIR/target/release/swc-test-runner" "$INPUT_FILE" "$SWC_OUTPUT"
fi

echo -e "${GREEN}✓ SWC transform complete${NC}"
echo ""

# Step 4: Compare outputs
echo -e "${YELLOW}[4/5] Comparing outputs...${NC}"

if ! diff -u "$BABEL_OUTPUT" "$SWC_OUTPUT" > /dev/null 2>&1; then
    echo -e "${RED}✗ Outputs differ!${NC}"
    echo ""
    echo "Differences:"
    diff -u "$BABEL_OUTPUT" "$SWC_OUTPUT" || true
    exit 1
fi

echo -e "${GREEN}✓ Outputs are identical!${NC}"
echo ""

# Step 5: Compare with expected output if provided
if [ -n "$EXPECTED_OUTPUT" ] && [ -f "$EXPECTED_OUTPUT" ]; then
    echo -e "${YELLOW}[5/5] Comparing with expected output...${NC}"

    if ! diff -u "$EXPECTED_OUTPUT" "$BABEL_OUTPUT" > /dev/null 2>&1; then
        echo -e "${RED}✗ Output doesn't match expected!${NC}"
        echo ""
        echo "Differences:"
        diff -u "$EXPECTED_OUTPUT" "$BABEL_OUTPUT" || true
        exit 1
    fi

    echo -e "${GREEN}✓ Output matches expected!${NC}"
else
    echo -e "${YELLOW}[5/5] No expected output provided, skipping validation${NC}"
fi

echo ""
echo -e "${GREEN}=== All tests passed! ===${NC}"
echo ""
echo "Output files:"
echo "  Babel:    $BABEL_OUTPUT"
echo "  SWC:      $SWC_OUTPUT"

# Cleanup
rm -rf "$SWC_TEST_DIR"
