# C# Syntax Support Test Files

This directory contains minimal `.lux` files for TDD (Test-Driven Development) of C# syntax features.

## Test Files

| File | Feature | Priority | Status |
|------|---------|----------|--------|
| `01_property_shorthand.lux` | Property initialization shorthand `{ name }` | P1 | Not Started |
| `02_arrow_functions.lux` | Arrow function syntax `x => expr` | P1 | Not Started |
| `03_is_pattern.lux` | `is` pattern matching | P1 | Not Started |
| `04_string_interpolation.lux` | String interpolation `$"..."` | P1 | Not Started |
| `05_null_coalescing.lux` | Null-coalescing `??` | P2 | Not Started |
| `06_null_conditional.lux` | Null-conditional `?.` | P2 | Not Started |
| `07_null_coalescing_assign.lux` | Null-coalescing assignment `??=` | P2 | Not Started |
| `08_linq_methods.lux` | LINQ method aliases | P3 | Not Started |
| `09_property_patterns.lux` | Property patterns `{ prop: value }` | P4 | Not Started |
| `10_type_aliases.lux` | C# type name aliases | P5 | Not Started |

## TDD Workflow

For each feature:

1. **Red**: Run the test file, confirm it fails with a parse/compile error
   ```bash
   cargo run --bin relux -- build examples/csharp_syntax/01_property_shorthand.lux --target swc
   ```

2. **Green**: Implement the feature until the test passes

3. **Refactor**: Clean up the implementation

## Running Tests

```bash
# Run a single test
cargo run --bin relux -- build examples/csharp_syntax/01_property_shorthand.lux --target swc

# Run all tests (once implemented)
for f in examples/csharp_syntax/*.lux; do
    echo "Testing $f..."
    cargo run --bin relux -- build "$f" --target swc
done
```

## Expected Errors (Before Implementation)

Each test file should fail with specific errors before the feature is implemented:

- **01**: Should parse successfully (shorthand may already work or fail on missing `:`)
- **02**: `unexpected token '=>'` or similar
- **03**: `unexpected keyword 'is'` or similar
- **04**: `unexpected token '$'` before string
- **05**: `unexpected token '??'`
- **06**: `unexpected token '?.'`
- **07**: `unexpected token '??='`
- **08**: `unknown method 'Select'` (semantic error, not parse error)
- **09**: `unexpected '{' in pattern`
- **10**: `unknown type 'string'` (semantic error)

## Implementation Order

Recommended implementation order based on:
- Complexity (simpler first)
- User impact (most commonly needed first)
- Dependencies (foundational features first)

### Phase 1 (Parser-only, no semantic changes)
1. `01_property_shorthand.lux` - Likely already works or trivial fix
2. `02_arrow_functions.lux` - May already be partially supported
3. `03_is_pattern.lux` - Desugar to `if let`
4. `04_string_interpolation.lux` - Desugar to `format!`

### Phase 2 (Requires semantic type info)
5. `05_null_coalescing.lux` - Needs Option type awareness
6. `06_null_conditional.lux` - Most complex null feature
7. `07_null_coalescing_assign.lux` - Builds on `??`

### Phase 3 (Method aliasing)
8. `08_linq_methods.lux` - Rewriter/codegen changes

### Phase 4 (Advanced patterns)
9. `09_property_patterns.lux` - Complex pattern desugaring
10. `10_type_aliases.lux` - Type name mapping
