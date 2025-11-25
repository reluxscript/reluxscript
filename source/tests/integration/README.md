# ReluxScript Integration Tests

This directory contains integration tests that verify Babel and SWC codegen produce identical outputs.

## Overview

The integration test framework:
1. Compiles `.lux` plugin files to both Babel (JS) and SWC (Rust)
2. Runs both plugins on test JavaScript code
3. Compares the outputs to ensure they match
4. Validates against expected output

## Directory Structure

```
integration/
├── package.json           # Node.js dependencies for Babel testing
├── setup.bat             # Install Node.js dependencies
├── README.md             # This file
└── console_remover/      # Example test case
    ├── test.toml         # Test manifest
    ├── plugin.lux        # ReluxScript plugin source
    ├── input.js          # JavaScript to transform
    └── expected.js       # Expected output
```

## Setup

### Prerequisites

- Node.js (v18 or later)
- npm
- Rust toolchain
- `@babel/core` and related packages (installed automatically)

### Installation

Run the setup script to install Node.js dependencies:

```batch
cd source\tests\integration
setup.bat
```

Or manually:

```batch
npm install
```

## Creating a Test Case

### 1. Create a test directory

```batch
mkdir source\tests\integration\my_test
```

### 2. Create `test.toml` manifest

```toml
[test]
name = "my-test"
description = "Description of what this test does"

[plugin]
source = "plugin.lux"

[input]
source = "input.js"

[expected]
source = "expected.js"
```

### 3. Create the plugin (`plugin.lux`)

Write your ReluxScript plugin:

```lux
plugin MyPlugin {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        if node.name == "foo" {
            *node = Identifier {
                name: "bar",
            };
        }
    }
}
```

### 4. Create test input (`input.js`)

JavaScript code to transform:

```javascript
const foo = 42;
console.log(foo);
```

### 5. Create expected output (`expected.js`)

What the transformed code should look like:

```javascript
const bar = 42;
console.log(bar);
```

## Running Tests

### Run all integration tests

```bash
cd source
cargo test --test integration_test
```

### Run a specific test

```bash
cargo test --test integration_test run_all_integration_tests
```

### With verbose output

```bash
cargo test --test integration_test -- --nocapture
```

## How It Works

### Test Execution Flow

```
1. Test Discovery
   └─> Scan integration/ for directories with test.toml

2. For Each Test:
   ├─> Compile plugin.lux to Babel (JS)
   ├─> Compile plugin.lux to SWC (Rust)
   ├─> Execute Babel plugin on input.js
   ├─> Execute SWC plugin on input.js
   ├─> Compare Babel output vs SWC output
   └─> Compare outputs vs expected.js

3. Report Results
   └─> ✓ Pass / ✗ Fail with diff
```

### Babel Execution

The test runner:
1. Generates a Node.js script that loads the Babel plugin
2. Uses `@babel/core` to transform the input
3. Captures and returns the output

### SWC Execution (TODO)

SWC execution is not yet implemented. Currently only Babel output is validated against expected output.

## Example Test Case: console_remover

Located in `console_remover/`:

- **Purpose**: Removes `console.log()` statements
- **Input**: JavaScript with console.log calls
- **Expected**: Same code without console.log statements
- **Validates**: If-let pattern matching, context methods, member expression handling

## Debugging Failed Tests

When a test fails, you'll see:

```
✗ Babel output does not match expected

Expected:
<expected code>

Got:
<actual output>
```

### Common Issues

1. **Whitespace differences**: The test normalizes whitespace, but extreme formatting differences may fail
2. **Compilation errors**: Check that your `.lux` syntax is correct
3. **Missing Node modules**: Run `setup.bat` to install dependencies
4. **Path issues**: Ensure all paths in `test.toml` are correct

### Manual Testing

You can manually compile and test:

```bash
# Compile to Babel
cd source
cargo run -- compile --target babel tests/integration/console_remover/plugin.lux -o /tmp/babel_out

# Inspect the generated plugin
cat /tmp/babel_out/index.js
```

## Continuous Integration

These tests run automatically:
- When you run `cargo test`
- In the CI/CD pipeline (GitHub Actions)
- Via the Windows Test Watcher service (monitors file changes)

## Adding More Test Cases

Good test cases to add:
- **Identifier renaming**: Simple transformations
- **Node removal**: Removing specific AST nodes
- **Node insertion**: Adding new nodes
- **Complex pattern matching**: Nested if-let patterns
- **State management**: Plugins with internal state
- **Error cases**: Invalid transformations

## Future Enhancements

- [ ] SWC plugin execution
- [ ] AST diff visualization
- [ ] Performance benchmarks (Babel vs SWC)
- [ ] Snapshot testing
- [ ] Test generation from examples
- [ ] Visual diff in test output

## Troubleshooting

### "toml not found" error

Add to `Cargo.toml`:
```toml
[dev-dependencies]
toml = "0.8"
```

### "Node command not found"

Ensure Node.js is installed and in your PATH:
```batch
node --version
npm --version
```

### Tests pass locally but fail in CI

Check:
- Node.js version differences
- File path separators (Windows vs Unix)
- Absolute vs relative paths in test manifests

## License

Same as ReluxScript project (MIT)
