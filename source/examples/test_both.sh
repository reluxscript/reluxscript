#!/bin/bash
# Test script to verify Babel and SWC plugins produce identical output

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ $# -lt 2 ]; then
    echo "Usage: $0 <plugin.lux> <input.js>"
    exit 1
fi

PLUGIN_FILE="$1"
INPUT_FILE="$2"

BABEL_PLUGIN="dist/index.js"
SWC_PLUGIN="dist/lib.rs"
BABEL_OUTPUT="babel_output.js"
SWC_OUTPUT="swc_output.js"

echo -e "${YELLOW}=== Testing Babel & SWC Cross-Compilation ===${NC}"
echo "Plugin: $PLUGIN_FILE"
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

# Step 2: Run Babel
echo -e "${YELLOW}[2/4] Running Babel transform...${NC}"
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
"

echo -e "${GREEN}✓ Babel complete${NC}"
echo ""

# Step 3: Build SWC test runner
echo -e "${YELLOW}[3/4] Building SWC test runner...${NC}"
SWC_DIR="swc_test"
rm -rf "$SWC_DIR"
mkdir -p "$SWC_DIR/src"

cat > "$SWC_DIR/Cargo.toml" << 'EOF'
[package]
name = "swc-test"
version = "0.1.0"
edition = "2021"

[dependencies]
swc_common = "17.0.1"
swc_ecma_ast = "18.0.0"
swc_ecma_visit = "18.0.1"
swc_ecma_parser = "27.0.3"
swc_ecma_codegen = "20.0.0"
EOF

# Get plugin struct name
PLUGIN_STRUCT=$(grep "^pub struct" "$SWC_PLUGIN" | head -1 | awk '{print $3}')

# Create main.rs - strip the imports from plugin since we'll add our own
PLUGIN_CODE=$(cat "$SWC_PLUGIN" | grep -v "^use " | grep -v "^// Generated" | grep -v "^// Do not edit")

cat > "$SWC_DIR/src/main.rs" <<EOF
use swc_common::{sync::Lrc, SourceMap, FileName, DUMMY_SP, SyntaxContext};
use swc_ecma_parser::{Parser, Syntax, StringInput};
use swc_ecma_codegen::{Emitter, text_writer::JsWriter, Config as CodegenConfig, Node};
use swc_ecma_visit::{VisitMut, VisitMutWith};
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

    let mut program = parser.parse_program()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut visitor = $PLUGIN_STRUCT {};
    program.visit_mut_with(&mut visitor);

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

    fs::write(output_file, String::from_utf8(buf)?)?;
    Ok(())
}
EOF

cd "$SWC_DIR"
cargo build --release --quiet 2>&1
cd ..

if [ ! -f "$SWC_DIR/target/release/swc-test.exe" ] && [ ! -f "$SWC_DIR/target/release/swc-test" ]; then
    echo -e "${RED}Error: SWC build failed${NC}"
    cat "$SWC_DIR/src/main.rs"
    exit 1
fi

# Step 4: Run SWC
echo -e "${YELLOW}[4/4] Running SWC transform...${NC}"
if [ -f "$SWC_DIR/target/release/swc-test.exe" ]; then
    "$SWC_DIR/target/release/swc-test.exe" "$INPUT_FILE" "$SWC_OUTPUT"
else
    "$SWC_DIR/target/release/swc-test" "$INPUT_FILE" "$SWC_OUTPUT"
fi

echo -e "${GREEN}✓ SWC complete${NC}"
echo ""

# Compare outputs (ignoring whitespace and comments)
echo -e "${YELLOW}Comparing outputs...${NC}"

# Normalize: remove comments and extra whitespace for comparison
normalize() {
    sed 's|//.*$||g' "$1" | sed '/^[[:space:]]*$/d' | sed 's/^[[:space:]]*//g' | sed 's/[[:space:]]*$//g'
}

BABEL_NORM=$(normalize "$BABEL_OUTPUT")
SWC_NORM=$(normalize "$SWC_OUTPUT")

if diff <(echo "$BABEL_NORM") <(echo "$SWC_NORM") > /dev/null 2>&1; then
    echo -e "${GREEN}✓✓✓ SUCCESS! Outputs are semantically identical! ✓✓✓${NC}"
    echo ""
    echo "Note: Formatting may differ slightly (whitespace, comments)"
else
    echo -e "${RED}✗ Outputs differ${NC}"
    echo ""
    echo "Babel output:"
    cat "$BABEL_OUTPUT"
    echo ""
    echo "SWC output:"
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
