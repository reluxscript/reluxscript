# Compiling to SWC

Learn how to compile your ReluxScript plugins to SWC (Rust) plugins.

## Basic Compilation

Compile a `.lux` file to SWC:

```bash
relux build my-plugin.lux --target swc
```

This generates `dist/lib.rs` - a Rust module that implements the SWC `VisitMut` trait.

## Output Format

The generated SWC plugin follows the standard SWC visitor pattern:

```rust
pub struct MyPlugin;

impl VisitMut for MyPlugin {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        // Your transformation logic
    }
}
```

## Using with SWC

The generated Rust code can be:

1. **Compiled to a native library** for use in Rust projects
2. **Compiled to WASM** for use in JavaScript tooling

### As a Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
swc_common = "0.33"
swc_ecma_ast = "0.110"
swc_ecma_visit = "0.96"
```

Then use the visitor:

```rust
use swc_ecma_visit::VisitMutWith;
let mut visitor = MyPlugin;
program.visit_mut_with(&mut visitor);
```

### As WASM

Compile to WebAssembly (requires additional setup):

```bash
cargo build --target wasm32-wasi --release
```

Use in SWC config:

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        ["./plugin.wasm", {}]
      ]
    }
  }
}
```

## Custom Output Directory

Specify a different output directory:

```bash
relux build my-plugin.lux --target swc --output build
```

This generates `build/lib.rs` instead of `dist/lib.rs`.

## Compile Both Targets

Compile to both Babel and SWC simultaneously:

```bash
relux build my-plugin.lux --target both
```

Or omit `--target` (defaults to both):

```bash
relux build my-plugin.lux
```

This generates:
- `dist/index.js` (Babel)
- `dist/lib.rs` (SWC)

## Next Steps

- [Compile to Babel](/v0.1/guide/compiling-babel)
- [View Examples](/v0.1/examples/)
