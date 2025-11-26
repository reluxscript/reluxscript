# Decorator Semantic Completeness Checklist

For each test case, verify the decorated AST provides ALL semantic information needed for dumb SWC codegen.

## Plugin Tests

### 1. console_remover
- [ ] `node.callee` - knows it's `Box<Callee>`, has unwrap strategy
- [ ] `Callee::MemberExpression` pattern - maps to `Callee::Member` (not `Callee::Expr`)
- [ ] `*member.object` - knows it's `Box<Expr>`, has `.as_ref()` or `&*` metadata
- [ ] `*member.property` - knows it's `MemberProp::Ident` (NOT `Expr::Ident`)
- [ ] `obj.name` comparison - knows needs `&*obj.sym` in SWC
- [ ] `prop.name` comparison - knows needs `&*prop.sym` in SWC
- [ ] All types resolved (no "Unknown")

### 2. rename_identifiers
- [ ] Identifier pattern matching - correct SWC type
- [ ] `identifier.name` reads - knows uses `.sym` in SWC
- [ ] `identifier.name` writes - knows needs `.into()` conversion
- [ ] All types resolved (no "Unknown")

### 3. add_annotations
- [ ] Function/variable declaration patterns - correct SWC variants
- [ ] Node field access - correct SWC field names
- [ ] Type annotation manipulation - correct SWC AST structure
- [ ] All types resolved (no "Unknown")

### 4. constant_folding
- [ ] Binary expression patterns - correct SWC types
- [ ] Literal value access - correct conversions
- [ ] Expression replacement - correct constructors
- [ ] All types resolved (no "Unknown")

### 5. inject_imports
- [ ] Program node access - correct SWC type
- [ ] Import declaration creation - correct SWC constructors
- [ ] Array manipulation (.push, .insert) - correct methods
- [ ] All types resolved (no "Unknown")

### 6. instrument_coverage
- [ ] Statement patterns - correct SWC variants
- [ ] Position/span access - correct field names
- [ ] Code injection - correct AST construction
- [ ] All types resolved (no "Unknown")

### 7. rewrite_imports
- [ ] Import specifier patterns - correct SWC variants
- [ ] String manipulation - correct conversions
- [ ] Module path access - correct field names
- [ ] All types resolved (no "Unknown")

### 8. transform_jsx
- [ ] JSX element patterns - correct SWC types
- [ ] Attribute access - correct field names
- [ ] Element transformation - correct constructors
- [ ] All types resolved (no "Unknown")

## Writer Tests

### 9. writer_component_metadata
- [ ] Component detection patterns - correct SWC types
- [ ] Props extraction - correct field access
- [ ] State tracking - correct type information
- [ ] All types resolved (no "Unknown")

### 10. writer_type_definitions
- [ ] Type annotation reading - correct SWC types
- [ ] Type construction - correct conversions
- [ ] String building - correct field access
- [ ] All types resolved (no "Unknown")

### 11. writer_documentation
- [ ] Comment extraction - correct SWC types
- [ ] JSDoc parsing - correct field access
- [ ] Documentation structure - correct type info
- [ ] All types resolved (no "Unknown")

### 12. writer_rust_bindings
- [ ] Type mapping - correct SWC type information
- [ ] Function signature extraction - correct field access
- [ ] Code generation - correct type conversions
- [ ] All types resolved (no "Unknown")

### 13. writer_static_analysis
- [ ] Pattern matching for analysis - correct SWC types
- [ ] Scope tracking - correct type information
- [ ] Metric calculation - correct field access
- [ ] All types resolved (no "Unknown")

### 14. writer_schema_extraction
- [ ] Object/interface patterns - correct SWC types
- [ ] Property extraction - correct field access
- [ ] Type inference - correct type information
- [ ] All types resolved (no "Unknown")

### 15. writer_test_fixtures
- [ ] Type inspection - correct SWC types
- [ ] Value generation - correct type information
- [ ] Mock data structure - correct field access
- [ ] All types resolved (no "Unknown")

### 16. writer_build_config
- [ ] Dependency tracking - correct SWC types
- [ ] Export detection - correct field access
- [ ] Config generation - correct type information
- [ ] All types resolved (no "Unknown")

## Verification Commands

For each test, run:
```bash
cd source
cargo run -- build tests/integration/{test_name}/plugin.lux --target swc --dump-decorated-ast 2>&1 | grep -E "Unknown|swc_type|swc_pattern|swc_field_name"
```

## Success Criteria

✅ **PASS**: All checkboxes checked, zero "Unknown" types, all metadata present
❌ **FAIL**: Any Unknown types, missing metadata, or incorrect SWC mappings

## Notes

The decorator must provide enough context that SWC codegen can be a simple translator:
- Pattern matching: Just emit `metadata.swc_pattern`
- Field access: Just emit `metadata.swc_field_name + metadata.accessor`
- Identifiers: Just emit `metadata.use_sym ? "&*ident.sym" : "ident.name"`
- No type inference in codegen - decorator does it all!
