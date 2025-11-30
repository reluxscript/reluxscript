#!/bin/bash
# Test script to verify Babel and SWC writer plugins produce identical output

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ $# -lt 2 ]; then
    echo "Usage: $0 <writer.lux> <input.js>"
    exit 1
fi

PLUGIN_FILE="$1"
INPUT_FILE="$2"

BABEL_PLUGIN="dist/index.js"
SWC_PLUGIN="dist/lib.rs"
BABEL_OUTPUT="babel_writer_output.txt"
SWC_OUTPUT="swc_writer_output.txt"

echo -e "${YELLOW}=== Testing Babel & SWC Writer Cross-Compilation ===${NC}"
echo "Writer: $PLUGIN_FILE"
echo "Input: $INPUT_FILE"
echo ""

# Step 1: Compile
echo -e "${YELLOW}[1/4] Compiling $PLUGIN_FILE...${NC}"
../target/release/relux build "$PLUGIN_FILE" 2>&1 | grep -E "(Generated|✓|Build complete|Error)" || true

if [ ! -f "$BABEL_PLUGIN" ] || [ ! -f "$SWC_PLUGIN" ]; then
    echo -e "${RED}Error: Plugins not generated${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Compilation successful${NC}"
echo ""

# Step 2: Run Babel writer
echo -e "${YELLOW}[2/4] Running Babel writer...${NC}"
node -e "
const babel = require('@babel/core');
const fs = require('fs');

const input = fs.readFileSync('$INPUT_FILE', 'utf8');

// Capture the finish() output by wrapping the plugin
let writerOutput = '';
const pluginWrapper = function(api) {
    const plugin = require('./$BABEL_PLUGIN')(api);
    const originalExit = plugin.visitor.Program.exit;

    plugin.visitor.Program = {
        exit(path, state) {
            const result = originalExit.call(this, path, state);
            writerOutput = result;
        }
    };

    return plugin;
};

babel.transformSync(input, {
    plugins: [pluginWrapper],
    filename: '$INPUT_FILE',
    configFile: false,
    babelrc: false
});

fs.writeFileSync('$BABEL_OUTPUT', writerOutput);
"

echo -e "${GREEN}✓ Babel writer complete${NC}"
echo ""

# Step 3: Build SWC test runner
echo -e "${YELLOW}[3/4] Building SWC writer test runner...${NC}"
SWC_DIR="swc_writer_test"
rm -rf "$SWC_DIR"
mkdir -p "$SWC_DIR/src"

cat > "$SWC_DIR/Cargo.toml" << 'EOF'
[package]
name = "swc-writer-test"
version = "0.1.0"
edition = "2021"

[dependencies]
swc_common = "17.0.1"
swc_ecma_ast = "18.0.0"
swc_ecma_visit = "18.0.1"
swc_ecma_parser = "27.0.3"
swc_ecma_codegen = "20.0.0"
regex = "1.11.1"
EOF

# Get writer struct name
WRITER_STRUCT=$(grep "^pub struct" "$SWC_PLUGIN" | head -1 | awk '{print $3}')

# Create main.rs - strip the imports from plugin since we'll add our own
PLUGIN_CODE=$(cat "$SWC_PLUGIN" | grep -v "^use " | grep -v "^// Generated" | grep -v "^// Do not edit")

cat > "$SWC_DIR/src/main.rs" <<EOF
use swc_common::{sync::Lrc, SourceMap, FileName};
use swc_ecma_parser::{Parser, Syntax, StringInput};
use swc_ecma_visit::{Visit, VisitWith};
use swc_ecma_ast::*;
use std::fs;

$PLUGIN_CODE

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input_file = &args[1];
    let output_file = &args[2];

    let source = fs::read_to_string(input_file)?;
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(Lrc::new(FileName::Custom(input_file.clone())), source);

    let syntax = Syntax::Es(Default::default());
    let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);

    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut writer = $WRITER_STRUCT::new();
    program.visit_with(&mut writer);

    let output = writer.finish();
    fs::write(output_file, output)?;
    Ok(())
}
EOF

cd "$SWC_DIR"
cargo build --release --quiet 2>&1
cd ..

if [ ! -f "$SWC_DIR/target/release/swc-writer-test.exe" ] && [ ! -f "$SWC_DIR/target/release/swc-writer-test" ]; then
    echo -e "${RED}Error: SWC writer build failed${NC}"
    cat "$SWC_DIR/src/main.rs"
    exit 1
fi

# Step 4: Run SWC writer
echo -e "${YELLOW}[4/4] Running SWC writer...${NC}"
if [ -f "$SWC_DIR/target/release/swc-writer-test.exe" ]; then
    "$SWC_DIR/target/release/swc-writer-test.exe" "$INPUT_FILE" "$SWC_OUTPUT"
else
    "$SWC_DIR/target/release/swc-writer-test" "$INPUT_FILE" "$SWC_OUTPUT"
fi

echo -e "${GREEN}✓ SWC writer complete${NC}"
echo ""

# Compare outputs
echo -e "${YELLOW}Comparing writer outputs...${NC}"

if diff "$BABEL_OUTPUT" "$SWC_OUTPUT" > /dev/null 2>&1; then
    echo -e "${GREEN}✓✓✓ SUCCESS! Writer outputs are identical! ✓✓✓${NC}"
else
    echo -e "${RED}✗ Writer outputs differ${NC}"
    echo ""
    echo "Babel writer output:"
    cat "$BABEL_OUTPUT"
    echo ""
    echo "SWC writer output:"
    cat "$SWC_OUTPUT"
    echo ""
    diff -u "$BABEL_OUTPUT" "$SWC_OUTPUT" || true
    exit 1
fi

echo ""
echo "Babel: $BABEL_OUTPUT"
echo "SWC:   $SWC_OUTPUT"

# Cleanup
rm -rf "$SWC_DIR"
