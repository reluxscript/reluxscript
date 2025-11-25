# Custom AST Properties Implementation

## Overview

This document describes the design and implementation of **Custom AST Properties** in ReluxScript - a unified abstraction that allows developers to attach arbitrary metadata to AST nodes while maintaining compatibility with both Babel (JavaScript) and SWC (Rust) code generation targets.

## Motivation

### The Problem

Babel plugins commonly mutate AST nodes by adding custom properties for tracking state across multiple traversal passes:

```javascript
// Babel plugin - common pattern
node.__hexPath = generateHexCode();
node.__processed = true;
node.__metadata = { ... };

// Later in another visitor
if (node.__hexPath) {
  // Use the hex code
}
```

This pattern is **impossible in SWC** because:
- SWC uses strongly-typed Rust structs for AST nodes
- You cannot add arbitrary fields to existing structs
- The only alternative is maintaining side-channel HashMaps manually

### The Solution

ReluxScript provides a **unified API** that looks like direct property mutation, but compiles differently for each target:

```reluxscript
// ReluxScript - works for both Babel and SWC
fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
    node.__hexPath = generate_hex_code();
    node.__processed = true;
}

fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // Read custom property
    if let Some(hex) = node.__hexPath {
        // Use it
    }
}
```

**Babel output:**
```javascript
function visit_jsx_element(node, ctx) {
    node.__hexPath = generateHexCode();
    node.__processed = true;
}
```

**SWC output:**
```rust
fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
    let node_id = self.get_node_id(node);
    self.state.set_custom_prop(node_id, "__hexPath", generateHexCode());
    self.state.set_custom_prop(node_id, "__processed", true);
}
```

## Feature Specification

### 1. Syntax

#### 1.1 Custom Property Naming Convention

Custom properties are identified by a **double underscore prefix** (`__`):

```reluxscript
node.__customProp = value;     // Custom property
node.name = value;             // Standard AST field
```

**Rules:**
- Must start with `__` (double underscore)
- Followed by valid identifier characters
- Case-sensitive
- Cannot conflict with standard AST field names

**Examples:**
```reluxscript
node.__hexPath = "0x1234";           // ✓ Valid
node.__processed = true;              // ✓ Valid
node.__metadata = metadata_obj;       // ✓ Valid
node._singleUnderscore = value;       // ✗ Not a custom property
node.standardField = value;           // ✗ Standard AST field
```

#### 1.2 Assignment

```reluxscript
// Direct assignment
node.__customProp = value;

// Assignment from expression
node.__hexPath = self.state.hex_gen.next();

// Assignment in conditionals
if condition {
    node.__marker = true;
}
```

#### 1.3 Reading

```reluxscript
// Option<T> pattern - property may not exist
if let Some(hex) = node.__hexPath {
    // Use hex
}

// Direct access with default
let hex = node.__hexPath.unwrap_or("default");

// Check existence
if node.__processed.is_some() {
    // Already processed
}
```

#### 1.4 Supported Value Types

Custom properties support these ReluxScript types:

- **Primitives**: `bool`, `i32`, `i64`, `f64`, `Str`
- **Containers**: `Vec<T>`, `HashMap<K, V>`, `Option<T>`
- **User-defined structs**: Any struct declared in the plugin
- **Enums**: Any enum declared in the plugin

```reluxscript
struct Metadata {
    hex: Str,
    depth: i32,
}

node.__metadata = Metadata { hex: "0x1234", depth: 3 };
node.__flags = vec!["processed", "validated"];
node.__map = HashMap::from([("key", "value")]);
```

### 2. Type System

#### 2.1 Type Inference

The type of a custom property is **inferred from first assignment**:

```reluxscript
// Type inferred as Str
node.__hexPath = "0x1234";

// Later assignments must match
node.__hexPath = "0x5678";  // ✓ OK
node.__hexPath = 42;         // ✗ Type error: expected Str, got i32
```

#### 2.2 Type Annotations

Explicit type annotations are supported:

```reluxscript
node.__hexPath: Str = generate_hex();
node.__depth: i32 = 0;
node.__metadata: Metadata = Metadata::new();
```

#### 2.3 Return Type

Reading a custom property always returns `Option<T>`:

```reluxscript
// node.__hexPath has type Option<Str>
let hex: Option<Str> = node.__hexPath;

// Pattern matching
match node.__hexPath {
    Some(hex) => println!("Hex: {}", hex),
    None => println!("No hex path"),
}
```

### 3. Scoping and Lifetime

#### 3.1 Node Identity

Custom properties are tied to **specific AST node instances**:

```reluxscript
fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
    node.__hexPath = "0x1234";
}

fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // Different node - won't have __hexPath
    if let Some(hex) = node.__hexPath {
        // This won't execute
    }
}
```

#### 3.2 Lifetime Across Passes

Custom properties persist **for the duration of the plugin execution**:

```reluxscript
// First pass
fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
    node.__hexPath = generate_hex();
}

// Second pass (same traversal or later)
fn visit_jsx_opening_element(node: &mut JSXOpeningElement, ctx: &Context) {
    // Can access parent's custom property if we have reference
    if let Some(parent) = ctx.parent_jsx_element() {
        if let Some(hex) = parent.__hexPath {
            // Use parent's hex path
        }
    }
}
```

#### 3.3 Deletion

Custom properties can be explicitly removed:

```reluxscript
// Set to None to remove
node.__hexPath = None;

// Or use conditional assignment
if should_remove {
    node.__processed = None;
}
```

## Implementation Design

### 4. Parser Changes

#### 4.1 AST Node Extensions

Add a new AST node type for custom property assignments:

```rust
// ast.rs
pub enum Stmt {
    // ... existing variants
    CustomPropAssignment(CustomPropAssignment),
}

pub struct CustomPropAssignment {
    pub node: Box<Expr>,      // The AST node being assigned to
    pub property: String,      // The custom property name (e.g., "__hexPath")
    pub value: Box<Expr>,      // The value being assigned
}

pub enum Expr {
    // ... existing variants
    CustomPropAccess(CustomPropAccess),
}

pub struct CustomPropAccess {
    pub node: Box<Expr>,      // The AST node being read from
    pub property: String,      // The custom property name
}
```

#### 4.2 Parsing Logic

In the parser (`parser.rs`), detect custom property patterns:

```rust
// Simplified pseudocode
fn parse_assignment(&mut self) -> Result<Stmt> {
    let left = self.parse_member_expr()?;

    // Check if this is a custom property assignment
    if let Expr::MemberExpression(member) = &left {
        if member.property.starts_with("__") {
            // This is a custom property
            self.expect(Token::Assign)?;
            let value = self.parse_expr()?;

            return Ok(Stmt::CustomPropAssignment(CustomPropAssignment {
                node: member.object.clone(),
                property: member.property.clone(),
                value,
            }));
        }
    }

    // Regular assignment
    // ...
}

fn parse_member_expr(&mut self) -> Result<Expr> {
    let object = self.parse_primary()?;

    if self.match_token(Token::Dot) {
        let property = self.parse_identifier()?;

        // Check if custom property access
        if property.starts_with("__") {
            return Ok(Expr::CustomPropAccess(CustomPropAccess {
                node: Box::new(object),
                property,
            }));
        }

        // Regular member expression
        return Ok(Expr::MemberExpression(MemberExpression {
            object: Box::new(object),
            property,
        }));
    }

    Ok(object)
}
```

### 5. Semantic Analysis

#### 5.1 Custom Property Registry

Track all custom properties used in the plugin:

```rust
// semantic/mod.rs
pub struct CustomPropertyRegistry {
    // Map: (node_type, property_name) -> inferred_type
    properties: HashMap<(String, String), Type>,
}

impl CustomPropertyRegistry {
    pub fn register(&mut self, node_type: &str, prop_name: &str, ty: Type) {
        let key = (node_type.to_string(), prop_name.to_string());

        if let Some(existing_type) = self.properties.get(&key) {
            // Verify type consistency
            if existing_type != &ty {
                // Type error: inconsistent types for same property
            }
        } else {
            self.properties.insert(key, ty);
        }
    }

    pub fn get_type(&self, node_type: &str, prop_name: &str) -> Option<&Type> {
        let key = (node_type.to_string(), prop_name.to_string());
        self.properties.get(&key)
    }
}
```

#### 5.2 Type Checking

Validate custom property usage during semantic analysis:

```rust
// semantic/type_checker.rs
fn check_custom_prop_assignment(&mut self, assign: &CustomPropAssignment) -> Result<()> {
    // 1. Ensure the target is an AST node type
    let node_type = self.infer_expr_type(&assign.node)?;
    if !self.is_ast_node_type(&node_type) {
        return Err(Error::NotAnASTNode(node_type));
    }

    // 2. Ensure property name starts with __
    if !assign.property.starts_with("__") {
        return Err(Error::InvalidCustomProperty(assign.property.clone()));
    }

    // 3. Infer value type
    let value_type = self.infer_expr_type(&assign.value)?;

    // 4. Register or verify type consistency
    self.custom_props.register(
        &node_type.to_string(),
        &assign.property,
        value_type,
    );

    Ok(())
}

fn check_custom_prop_access(&mut self, access: &CustomPropAccess) -> Result<Type> {
    // 1. Get node type
    let node_type = self.infer_expr_type(&access.node)?;

    // 2. Look up registered type
    if let Some(ty) = self.custom_props.get_type(&node_type.to_string(), &access.property) {
        // Return Option<T>
        Ok(Type::Option(Box::new(ty.clone())))
    } else {
        // Property never assigned - type is Option<Unknown>
        Ok(Type::Option(Box::new(Type::Unknown)))
    }
}
```

#### 5.3 Validation Rules

The semantic analyzer enforces:

1. **Custom property naming**: Must start with `__`
2. **Target validation**: Can only assign to AST node types
3. **Type consistency**: Same property must have consistent type across all uses
4. **Return type**: Reading custom properties always returns `Option<T>`

### 6. Code Generation - Babel

#### 6.1 Assignment Generation

Custom property assignments compile to direct property mutations:

```rust
// codegen/babel.rs
impl BabelCodegen {
    fn gen_custom_prop_assignment(&mut self, assign: &CustomPropAssignment) {
        // Generate: node.__hexPath = value
        self.gen_expr(&assign.node);
        self.emit(".");
        self.emit(&assign.property);
        self.emit(" = ");
        self.gen_expr(&assign.value);
    }
}
```

**Example:**

ReluxScript:
```reluxscript
node.__hexPath = "0x1234";
```

Babel output:
```javascript
node.__hexPath = "0x1234";
```

#### 6.2 Access Generation

Custom property reads compile to direct property access:

```rust
fn gen_custom_prop_access(&mut self, access: &CustomPropAccess) {
    // Generate: node.__hexPath
    self.gen_expr(&access.node);
    self.emit(".");
    self.emit(&access.property);
}
```

**Example:**

ReluxScript:
```reluxscript
if let Some(hex) = node.__hexPath {
    // use hex
}
```

Babel output:
```javascript
if (node.__hexPath !== undefined && node.__hexPath !== null) {
    const hex = node.__hexPath;
    // use hex
}
```

#### 6.3 None Assignment

Setting to `None` compiles to `delete`:

```rust
fn gen_custom_prop_delete(&mut self, assign: &CustomPropAssignment) {
    // node.__hexPath = None -> delete node.__hexPath
    self.emit("delete ");
    self.gen_expr(&assign.node);
    self.emit(".");
    self.emit(&assign.property);
}
```

### 7. Code Generation - SWC

#### 7.1 State Infrastructure

Auto-inject custom property storage into the State struct:

```rust
// codegen/swc.rs
impl SwcCodegen {
    fn gen_state_struct(&mut self, state: &StateDecl) {
        self.emit("#[derive(Default)]\n");
        self.emit("pub struct State {\n");
        self.indent();

        // User-defined fields
        for field in &state.fields {
            self.gen_field(field);
        }

        // Auto-injected custom property storage
        self.emit("// Auto-generated: Custom AST property storage\n");
        self.emit("__custom_props: std::collections::HashMap<usize, std::collections::HashMap<String, CustomPropValue>>,\n");

        self.dedent();
        self.emit("}\n\n");

        // Generate CustomPropValue enum
        self.gen_custom_prop_value_enum();

        // Generate helper methods
        self.gen_custom_prop_helpers();
    }
}
```

#### 7.2 CustomPropValue Enum

Generate a value enum to hold different types:

```rust
fn gen_custom_prop_value_enum(&mut self) {
    self.emit("#[derive(Clone, Debug)]\n");
    self.emit("enum CustomPropValue {\n");
    self.indent();
    self.emit("Bool(bool),\n");
    self.emit("I32(i32),\n");
    self.emit("I64(i64),\n");
    self.emit("F64(f64),\n");
    self.emit("Str(String),\n");
    self.emit("Vec(Vec<CustomPropValue>),\n");
    self.emit("Map(std::collections::HashMap<String, CustomPropValue>),\n");

    // Add user-defined types as needed
    for custom_type in &self.custom_prop_types {
        self.emit(&format!("{}({}),\n", custom_type.name, custom_type.name));
    }

    self.dedent();
    self.emit("}\n\n");
}
```

#### 7.3 Helper Methods

Generate state helper methods:

```rust
fn gen_custom_prop_helpers(&mut self) {
    self.emit("impl State {\n");
    self.indent();

    // get_node_id: Generate unique ID for AST nodes
    self.emit("fn get_node_id<T>(&self, node: &T) -> usize {\n");
    self.indent();
    self.emit("// Use node memory address as ID\n");
    self.emit("node as *const T as usize\n");
    self.dedent();
    self.emit("}\n\n");

    // set_custom_prop: Set a custom property
    self.emit("fn set_custom_prop<T>(&mut self, node: &T, prop: &str, value: CustomPropValue) {\n");
    self.indent();
    self.emit("let node_id = self.get_node_id(node);\n");
    self.emit("self.__custom_props\n");
    self.indent();
    self.emit(".entry(node_id)\n");
    self.emit(".or_insert_with(std::collections::HashMap::new)\n");
    self.emit(".insert(prop.to_string(), value);\n");
    self.dedent();
    self.dedent();
    self.emit("}\n\n");

    // get_custom_prop: Get a custom property
    self.emit("fn get_custom_prop<T>(&self, node: &T, prop: &str) -> Option<&CustomPropValue> {\n");
    self.indent();
    self.emit("let node_id = self.get_node_id(node);\n");
    self.emit("self.__custom_props\n");
    self.indent();
    self.emit(".get(&node_id)\n");
    self.emit(".and_then(|m| m.get(prop))\n");
    self.dedent();
    self.dedent();
    self.emit("}\n\n");

    // delete_custom_prop: Remove a custom property
    self.emit("fn delete_custom_prop<T>(&mut self, node: &T, prop: &str) {\n");
    self.indent();
    self.emit("let node_id = self.get_node_id(node);\n");
    self.emit("if let Some(props) = self.__custom_props.get_mut(&node_id) {\n");
    self.indent();
    self.emit("props.remove(prop);\n");
    self.dedent();
    self.emit("}\n");
    self.dedent();
    self.emit("}\n");

    self.dedent();
    self.emit("}\n\n");
}
```

#### 7.4 Assignment Generation

Custom property assignments compile to helper method calls:

```rust
fn gen_custom_prop_assignment(&mut self, assign: &CustomPropAssignment) {
    let value_type = self.infer_expr_type(&assign.value);

    // Check for None assignment (deletion)
    if self.is_none_literal(&assign.value) {
        // Generate: self.state.delete_custom_prop(node, "__hexPath")
        self.emit("self.state.delete_custom_prop(");
        self.gen_expr(&assign.node);
        self.emit(", \"");
        self.emit(&assign.property);
        self.emit("\")");
        return;
    }

    // Generate: self.state.set_custom_prop(node, "__hexPath", CustomPropValue::Str(value))
    self.emit("self.state.set_custom_prop(");
    self.gen_expr(&assign.node);
    self.emit(", \"");
    self.emit(&assign.property);
    self.emit("\", ");

    // Wrap value in CustomPropValue variant
    self.gen_custom_prop_value_wrapper(&value_type);
    self.emit("(");
    self.gen_expr(&assign.value);
    self.emit("))");
}

fn gen_custom_prop_value_wrapper(&mut self, ty: &Type) {
    match ty {
        Type::Bool => self.emit("CustomPropValue::Bool"),
        Type::I32 => self.emit("CustomPropValue::I32"),
        Type::I64 => self.emit("CustomPropValue::I64"),
        Type::F64 => self.emit("CustomPropValue::F64"),
        Type::Str => self.emit("CustomPropValue::Str"),
        Type::Vec(_) => self.emit("CustomPropValue::Vec"),
        Type::HashMap(_, _) => self.emit("CustomPropValue::Map"),
        Type::Named(name) => self.emit(&format!("CustomPropValue::{}", name)),
        _ => panic!("Unsupported custom property type: {:?}", ty),
    }
}
```

**Example:**

ReluxScript:
```reluxscript
node.__hexPath = "0x1234";
```

SWC output:
```rust
self.state.set_custom_prop(
    node,
    "__hexPath",
    CustomPropValue::Str("0x1234".to_string())
);
```

#### 7.5 Access Generation

Custom property reads compile to getter calls with unwrapping:

```rust
fn gen_custom_prop_access(&mut self, access: &CustomPropAccess) {
    // Generate: self.state.get_custom_prop(node, "__hexPath")
    //           .and_then(|v| if let CustomPropValue::Str(s) = v { Some(s.clone()) } else { None })

    self.emit("self.state.get_custom_prop(");
    self.gen_expr(&access.node);
    self.emit(", \"");
    self.emit(&access.property);
    self.emit("\").and_then(|v| ");

    // Generate unwrapper based on type
    let prop_type = self.custom_props.get_type(
        &self.node_type_to_string(&access.node),
        &access.property
    );

    if let Some(ty) = prop_type {
        self.gen_custom_prop_unwrapper(ty);
    } else {
        // Unknown type - return None
        self.emit("None");
    }

    self.emit(")");
}

fn gen_custom_prop_unwrapper(&mut self, ty: &Type) {
    match ty {
        Type::Str => {
            self.emit("if let CustomPropValue::Str(s) = v { Some(s.clone()) } else { None }");
        }
        Type::Bool => {
            self.emit("if let CustomPropValue::Bool(b) = v { Some(*b) } else { None }");
        }
        Type::I32 => {
            self.emit("if let CustomPropValue::I32(i) = v { Some(*i) } else { None }");
        }
        // ... other types
        Type::Named(name) => {
            self.emit(&format!(
                "if let CustomPropValue::{}(val) = v {{ Some(val.clone()) }} else {{ None }}",
                name
            ));
        }
        _ => panic!("Unsupported custom property type: {:?}", ty),
    }
}
```

**Example:**

ReluxScript:
```reluxscript
if let Some(hex) = node.__hexPath {
    // use hex
}
```

SWC output:
```rust
if let Some(hex) = self.state.get_custom_prop(node, "__hexPath")
    .and_then(|v| if let CustomPropValue::Str(s) = v { Some(s.clone()) } else { None })
{
    // use hex
}
```

### 8. Node Identity Strategy

#### 8.1 Babel - Object Identity

In JavaScript, AST nodes are objects with inherent identity:

```javascript
// Same object reference = same identity
node.__hexPath = "0x1234";
// ... traversal continues ...
if (node.__hexPath) {  // Same node, property persists
    // ...
}
```

No special tracking needed - JavaScript's reference semantics handle this.

#### 8.2 SWC - Memory Address Hashing

In Rust, we use the node's **memory address** as a unique identifier:

```rust
fn get_node_id<T>(&self, node: &T) -> usize {
    node as *const T as usize
}
```

**Rationale:**
- Memory addresses are unique within a single program execution
- Fast to compute (no hashing overhead)
- Works for any AST node type

**Limitations:**
- Only valid during a single traversal/execution
- Custom properties don't persist across separate plugin runs
- This matches Babel's behavior (properties don't persist to disk)

**Alternative approaches** (if needed in the future):
1. **Span-based ID**: Use `node.span.lo` + `node.span.hi` as ID
2. **Content hash**: Hash node structure for deterministic IDs
3. **Explicit ID assignment**: Add `__id` field during first traversal

For minimact use case, memory address is sufficient.

### 9. Memory Management

#### 9.1 Babel - Automatic Garbage Collection

No special handling needed. JavaScript's GC cleans up custom properties when nodes are no longer referenced.

#### 9.2 SWC - HashMap Cleanup

The `__custom_props` HashMap grows as custom properties are added. Consider cleanup strategies:

**Option 1: No cleanup** (simplest)
- HashMap lives for duration of plugin execution
- Memory usage proportional to number of AST nodes with custom props
- Acceptable for most use cases

**Option 2: Manual cleanup**
```rust
impl State {
    pub fn clear_custom_props(&mut self) {
        self.__custom_props.clear();
    }

    pub fn clear_node_props<T>(&mut self, node: &T) {
        let node_id = self.get_node_id(node);
        self.__custom_props.remove(&node_id);
    }
}
```

**Option 3: Automatic cleanup in exit hook**
```reluxscript
fn exit(program: &mut Program, ctx: &Context) {
    // Cleanup happens automatically when State is dropped
}
```

For minimact, **Option 1** (no cleanup) is recommended - the HashMap is dropped when the plugin completes.

### 10. Error Handling

#### 10.1 Compile-Time Errors

The compiler should report clear errors:

**Type mismatch:**
```reluxscript
node.__hexPath = "0x1234";  // Type: Str
node.__hexPath = 42;         // ERROR
```

Error message:
```
error: type mismatch for custom property '__hexPath'
  --> plugin.lux:12:5
   |
10 |     node.__hexPath = "0x1234";
   |                       -------- first assigned as type 'Str'
11 |
12 |     node.__hexPath = 42;
   |                      ^^ expected 'Str', found 'i32'
```

**Invalid target:**
```reluxscript
let x = 42;
x.__custom = true;  // ERROR: not an AST node
```

Error message:
```
error: custom properties can only be assigned to AST nodes
  --> plugin.lux:5:5
   |
5 |     x.__custom = true;
   |     ^ 'x' has type 'i32', not an AST node type
```

**Invalid property name:**
```reluxscript
node._singleUnderscore = value;  // ERROR: must start with __
```

Error message:
```
error: custom properties must start with '__' (double underscore)
  --> plugin.lux:3:10
   |
3 |     node._singleUnderscore = value;
   |          ^^^^^^^^^^^^^^^^^ invalid custom property name
   |
help: custom property names must start with '__', like '__singleUnderscore'
```

#### 10.2 Runtime Behavior

**Accessing undefined property:**
```reluxscript
// Property never set - returns None
if let Some(hex) = node.__hexPath {
    // Won't execute
} else {
    // Executes - property is None
}
```

**Type mismatch at runtime (SWC only):**

If somehow a type mismatch occurs in the HashMap (e.g., from verbatim blocks or unsafe code), the getter returns `None`:

```rust
// Stored as Bool
self.state.set_custom_prop(node, "__flag", CustomPropValue::Bool(true));

// Try to read as Str - returns None (not a panic)
if let Some(s) = self.state.get_custom_prop(node, "__flag")
    .and_then(|v| if let CustomPropValue::Str(s) = v { Some(s.clone()) } else { None })
{
    // Won't execute - type mismatch returns None
}
```

### 11. Performance Considerations

#### 11.1 Babel Performance

**Impact:** Minimal to none
- Direct property access is native JavaScript operation
- Same performance as existing Babel plugins
- No overhead compared to manual `node.__prop = value`

#### 11.2 SWC Performance

**HashMap overhead:**
- Hash computation: ~10-20ns per operation
- Lookup: O(1) average case
- Memory: ~24 bytes per entry + key/value sizes

**Optimization strategies:**

1. **Use FxHashMap** (faster than default HashMap):
```rust
use rustc_hash::FxHashMap;

struct State {
    __custom_props: FxHashMap<usize, FxHashMap<String, CustomPropValue>>,
}
```

2. **Pre-allocate with capacity** (if node count is known):
```rust
fn init() -> State {
    State {
        __custom_props: FxHashMap::with_capacity_and_hasher(
            1024,  // Expected number of nodes
            Default::default()
        ),
    }
}
```

3. **Inline property access** (compiler optimization):
```rust
#[inline(always)]
fn get_custom_prop<T>(&self, node: &T, prop: &str) -> Option<&CustomPropValue> {
    // ... implementation
}
```

**Benchmark targets:**
- Custom property set: < 100ns
- Custom property get: < 50ns
- Memory overhead: < 1KB per 100 nodes with props

### 12. Testing Strategy

#### 12.1 Unit Tests

Test individual components:

**Parser tests:**
```rust
#[test]
fn test_parse_custom_prop_assignment() {
    let src = "node.__hexPath = \"0x1234\";";
    let ast = parse(src).unwrap();

    assert!(matches!(
        ast.body[0],
        Stmt::CustomPropAssignment(ref assign) if assign.property == "__hexPath"
    ));
}

#[test]
fn test_parse_custom_prop_access() {
    let src = "let hex = node.__hexPath;";
    let ast = parse(src).unwrap();
    // ...
}
```

**Semantic tests:**
```rust
#[test]
fn test_type_consistency_check() {
    let src = r#"
        node.__hexPath = "0x1234";
        node.__hexPath = 42;  // Should error
    "#;

    let result = analyze(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("type mismatch"));
}
```

**Codegen tests:**
```rust
#[test]
fn test_babel_codegen_custom_prop() {
    let src = "node.__hexPath = \"0x1234\";";
    let output = compile_to_babel(src).unwrap();

    assert_eq!(output, "node.__hexPath = \"0x1234\";");
}

#[test]
fn test_swc_codegen_custom_prop() {
    let src = "node.__hexPath = \"0x1234\";";
    let output = compile_to_swc(src).unwrap();

    assert!(output.contains("self.state.set_custom_prop"));
    assert!(output.contains("CustomPropValue::Str"));
}
```

#### 12.2 Integration Tests

Test complete plugin scenarios:

```rust
#[test]
fn test_hex_path_plugin_babel() {
    let plugin_src = r#"
        plugin HexPathPlugin {
            struct State {
                counter: i32,
            }

            fn init() -> State {
                State { counter: 0 }
            }

            fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
                self.state.counter += 1;
                node.__hexPath = format!("0x{:x}", self.state.counter);
            }

            fn visit_identifier(node: &mut Identifier, ctx: &Context) {
                if let Some(parent) = ctx.parent_jsx_element() {
                    if let Some(hex) = parent.__hexPath {
                        println!("Identifier in element with hex: {}", hex);
                    }
                }
            }
        }
    "#;

    let babel_plugin = compile_to_babel(plugin_src).unwrap();

    // Test the generated Babel plugin
    let input_jsx = "<div><span>Hello</span></div>";
    let output = run_babel_plugin(&babel_plugin, input_jsx);

    // Verify custom properties were added
    assert!(output.ast.program.body[0].__hexPath == "0x1");
}

#[test]
fn test_hex_path_plugin_swc() {
    // Similar test for SWC target
}
```

#### 12.3 End-to-End Tests

Test real-world usage with minimact:

```rust
#[test]
fn test_minimact_conversion() {
    // Use actual minimact plugin code
    let minimact_src = read_file("minimact_full_refactored_v2.lux");

    // Compile to both targets
    let babel_output = compile_to_babel(&minimact_src).unwrap();
    let swc_output = compile_to_swc(&minimact_src).unwrap();

    // Both should compile without errors
    assert!(babel_output.len() > 0);
    assert!(swc_output.len() > 0);

    // Test on sample React component
    let react_input = read_file("test_component.tsx");

    // Run both plugins and compare outputs
    let babel_result = run_babel_plugin(&babel_output, &react_input);
    let swc_result = run_swc_plugin(&swc_output, &react_input);

    // Both should produce same C# output
    assert_eq!(babel_result.csharp, swc_result.csharp);
}
```

### 13. Migration Guide

#### 13.1 For Existing minimact Plugin

The minimact plugin currently uses direct AST mutation in the JavaScript source. After this feature is implemented:

**Before (JavaScript):**
```javascript
node.__hexPath = hexPathGenerator.next();

if (node.__hexPath) {
    // use it
}
```

**After (ReluxScript):**
```reluxscript
node.__hexPath = hex_path_generator.next();

if let Some(hex) = node.__hexPath {
    // use hex
}
```

**Changes required:**
1. Property access must use Option pattern (`if let Some(...)`)
2. No other changes needed - assignments work the same way

#### 13.2 For New Plugins

When writing new ReluxScript plugins:

**Use custom properties for:**
- Marking nodes as processed
- Storing generated identifiers (hex paths, unique IDs)
- Caching computed values across passes
- Tracking relationships between nodes

**Don't use custom properties for:**
- Plugin-wide state (use `State` struct instead)
- Data that needs to persist to disk (use `ctx.state` + `exit()` hook)
- Data shared between different plugins (use standard AST fields)

### 14. Future Enhancements

#### 14.1 Property Namespacing

Add support for namespaced properties to avoid conflicts:

```reluxscript
node.__minimact::hexPath = "0x1234";
node.__otherPlugin::customData = data;
```

#### 14.2 Property Inheritance

Allow child nodes to inherit custom properties:

```reluxscript
// Set on parent
parentNode.__theme = "dark";

// Auto-inherit in children (opt-in)
if let Some(theme) = childNode.__theme @inherited {
    // Gets "dark" from parent
}
```

#### 14.3 Property Serialization

Support persisting custom properties to JSON for debugging:

```reluxscript
fn exit(program: &mut Program, ctx: &Context) {
    let props = ctx.serialize_custom_props();
    fs::write("custom_props.json", props);
}
```

#### 14.4 Property Type Validation

Stronger type validation with explicit declarations:

```reluxscript
plugin MyPlugin {
    // Declare custom properties upfront
    custom_properties {
        __hexPath: Str,
        __processed: bool,
        __metadata: Metadata,
    }

    // Rest of plugin...
}
```

#### 14.5 Performance Optimizations

**Compact encoding** for common types:

```rust
// Instead of full CustomPropValue enum, use compact encoding
enum CustomPropValueCompact {
    Inline(u64),  // Encode small types (bool, i32, etc.) inline
    Heap(Box<CustomPropValue>),  // Larger types on heap
}
```

**Property pools** for frequently-accessed properties:

```rust
struct State {
    // Fast path for common properties
    hex_paths: FxHashMap<usize, String>,
    processed_flags: FxHashMap<usize, bool>,

    // Fallback for other properties
    __custom_props: FxHashMap<usize, FxHashMap<String, CustomPropValue>>,
}
```

## 15. Examples

### 15.1 Basic Usage

```reluxscript
plugin SimpleMarker {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Mark element as processed
        node.__processed = true;
    }

    fn visit_jsx_opening_element(node: &mut JSXOpeningElement, ctx: &Context) {
        // Check if parent was processed
        if let Some(parent) = ctx.parent_jsx_element() {
            if parent.__processed.is_some() {
                println!("Parent was processed");
            }
        }
    }
}
```

### 15.2 Hex Path Generator (minimact use case)

```reluxscript
plugin MinimactHexPath {
    struct State {
        counter: i32,
    }

    fn init() -> State {
        State { counter: 0 }
    }

    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Generate hex path
        self.state.counter += 1;
        let hex = format!("0x{:04x}", self.state.counter);
        node.__hexPath = hex;

        // Store metadata
        node.__depth = ctx.depth();
        node.__isRoot = ctx.is_root();
    }

    fn visit_jsx_attribute(node: &mut JSXAttribute, ctx: &Context) {
        // Access parent's hex path
        if let Some(parent) = ctx.parent_jsx_element() {
            if let Some(hex) = parent.__hexPath {
                println!("Attribute in element: {}", hex);
            }
        }
    }
}
```

### 15.3 Complex Metadata

```reluxscript
struct ElementMetadata {
    hex_path: Str,
    has_state: bool,
    event_handlers: Vec<Str>,
}

plugin MetadataTracker {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        let metadata = ElementMetadata {
            hex_path: generate_hex(),
            has_state: check_for_state(node),
            event_handlers: extract_handlers(node),
        };

        node.__metadata = metadata;
    }

    fn exit(program: &mut Program, ctx: &Context) {
        // Collect all metadata
        let all_metadata = collect_metadata_from_elements();

        // Write to file
        let json = json::stringify(&all_metadata);
        fs::write("metadata.json", json);
    }
}
```

### 15.4 Multi-Pass Processing

```reluxscript
plugin MultiPassPlugin {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // First pass: mark elements for processing
        if should_process(node) {
            node.__needsTransform = true;
        }
    }

    fn visit_jsx_opening_element(node: &mut JSXOpeningElement, ctx: &Context) {
        // Second pass: transform marked elements
        if let Some(parent) = ctx.parent_jsx_element() {
            if parent.__needsTransform.unwrap_or(false) {
                // Perform transformation
                transform_element(node);

                // Mark as complete
                parent.__transformed = true;
            }
        }
    }
}
```

## 16. Summary

Custom AST Properties provide a **clean, unified abstraction** for attaching metadata to AST nodes in ReluxScript plugins. The feature:

- ✅ **Unified API**: Same code works for both Babel and SWC
- ✅ **Type-safe**: Compile-time type checking for property consistency
- ✅ **Ergonomic**: Looks like natural property access
- ✅ **Performant**: Minimal overhead in both targets
- ✅ **Compatible**: Works with existing ReluxScript features

**Implementation priority:**
1. Parser support (custom property syntax)
2. Semantic analysis (type tracking)
3. Babel codegen (direct passthrough)
4. SWC codegen (HashMap-based storage)
5. Testing and documentation
6. Migration of minimact plugin

This feature unblocks the complete migration of babel-plugin-minimact to ReluxScript and enables other plugins that rely on AST metadata tracking.
