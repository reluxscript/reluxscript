# Babel Plugin Minimact → ReluxScript Migration Plan

**Status**: Planning
**Target**: Migrate babel-plugin-minimact from JavaScript to ReluxScript
**Benefit**: Single source that compiles to both Babel (npm) and SWC (native/WASM)

---

## 1. Overview

### Current State
The babel-plugin-minimact is a JavaScript Babel plugin (~3,500 lines across 39 files) that:
- Transpiles TSX components to C# MinimactComponent classes
- Extracts templates, hooks, event handlers
- Generates JSON files for hot reload
- Produces structural change detection

### Target State
A ReluxScript `writer` plugin that:
- Compiles to both Babel and SWC targets
- Maintains feature parity with current JS implementation
- Enables native performance for Cactus Browser (no Node.js dependency)

---

## 2. Architecture Analysis

### 2.1 Plugin Type: `writer` (Not `plugin`)

babel-plugin-minimact is a **transpiler**, not a transformer. It:
- Reads TSX AST (doesn't mutate it)
- Generates C# code
- Outputs JSON metadata files

This maps to ReluxScript's `writer` API (Section 16 of spec):

```reluxscript
writer MinimactTranspiler {
    builder: CodeBuilder,

    fn visit_function_declaration(node: &FunctionDeclaration, ctx: &Context) {
        // Read-only visitor - generates C# output
    }

    fn finish(self) -> Str {
        self.builder.to_string()
    }
}
```

### 2.2 Module Structure Mapping

| Current JS Module | ReluxScript Equivalent |
|-------------------|----------------------|
| `index.cjs` | Main writer declaration |
| `processComponent.cjs` | `visit_function_declaration` |
| `extractors/*.cjs` | Helper functions |
| `generators/*.cjs` | CodeBuilder methods |
| `analyzers/*.cjs` | Helper functions |
| `types/*.cjs` | Struct definitions + helpers |
| `utils/*.cjs` | Standard library extensions |

---

## 3. Feature Migration Checklist

### 3.1 Core Features (Phase 1)

- [ ] **Component Detection**
  - Detect `FunctionDeclaration` with PascalCase name
  - Detect JSX return statement
  - Extract function parameters (props)

- [ ] **Hook Extraction**
  - `useState` → `[State]` field
  - `useEffect` → `OnInitialize`/`OnUpdate`
  - `useRef` → `[Ref]` field
  - 20+ custom hooks (useClientState, useServerTask, etc.)

- [ ] **JSX to VNode**
  - Generate `VElement`, `VText`, `VNull`
  - Handle attributes → props dictionary
  - Handle event handlers → method references

- [ ] **C# Code Generation**
  - Generate class declaration
  - Generate `Render()` method
  - Generate event handler methods

### 3.2 Advanced Features (Phase 2)

- [ ] **Template Extraction**
  - Text templates with bindings
  - Attribute templates
  - Loop templates (`.map()` patterns)
  - Conditional templates

- [ ] **Structural Change Detection**
  - Compare with previous `.tsx.keys`
  - Detect insertions/deletions
  - Detect hook changes

- [ ] **TypeScript Support**
  - Interface → C# class conversion
  - Type inference from generics
  - Prop type extraction

### 3.3 Output Files (Phase 3)

- [ ] **`.cs` file** - C# component class
- [ ] **`.templates.json`** - Template metadata
- [ ] **`.hooks.json`** - Hook signature
- [ ] **`.structural-changes.json`** - Change detection
- [ ] **`.tsx.keys`** - Keyed JSX for hot reload

---

## 4. ReluxScript Gaps & Enhancements Needed

### 4.1 Critical Gaps

#### A. Multiple Output Files
ReluxScript `writer` returns a single `Str`. babel-plugin-minimact generates 5+ files.

**Solution**: Extend ReluxScript with multi-file output:

```reluxscript
writer MinimactTranspiler {
    outputs: HashMap<Str, CodeBuilder>,

    fn finish(self) -> HashMap<Str, Str> {
        let mut result = HashMap::new();
        for (key, builder) in self.outputs {
            result.insert(key, builder.to_string());
        }
        result
    }
}
```

Or use return type annotation:

```reluxscript
#[output_files]
writer MinimactTranspiler {
    fn finish(self) -> MultiFileOutput {
        MultiFileOutput {
            files: vec![
                ("Component.cs", self.csharp.to_string()),
                ("Component.templates.json", self.templates.to_json()),
            ]
        }
    }
}
```

#### B. File System Access
Current plugin reads/writes files directly. ReluxScript has `use fs;` but:
- SWC plugins run in WASM sandbox
- File access needs platform-specific handling

**Solution**: File I/O should be handled by the host (CLI/IDE), not the plugin:

```reluxscript
// ReluxScript plugin returns structured output
// Host CLI writes files based on output

// Alternative: Platform escape hatch
#[cfg(target = "babel")]
fn write_file(path: &Str, content: &Str) {
    fs::write(path, content).unwrap();
}

#[cfg(target = "swc")]
fn write_file(path: &Str, content: &Str) {
    // SWC plugins return output to host
    self.pending_writes.push((path.clone(), content.clone()));
}
```

#### C. JSON Serialization
Heavy use of JSON for output. ReluxScript has `use json;` but needs:
- Nested object serialization
- Pretty printing
- Custom struct serialization

**Solution**: Already in spec (Section 3.4):

```reluxscript
#[derive(Serialize)]
struct TemplateMap {
    templates: HashMap<Str, Template>,
    loopTemplates: Vec<LoopTemplate>,
}

fn output_templates() {
    let map = TemplateMap { ... };
    let json_str = json::to_string_pretty(&map).unwrap();
}
```

#### D. Regular Expressions
Template extraction uses regex for pattern matching.

**Solution**: Not supported in ReluxScript (Section 18.1). Must use string methods:

```reluxscript
// Instead of: /key=(?:"([^"]+)"|'([^']+)'|\{([^}]+)\})/g
fn extract_keys(source: &Str) -> Vec<Str> {
    let mut keys = vec![];
    let mut pos = 0;

    while pos < source.len() {
        if let Some(key_pos) = source.find_from("key=", pos) {
            // Manual parsing logic
        }
    }

    keys
}
```

### 4.2 Medium Gaps

#### E. Complex String Operations
Need `split`, `replace`, `slice`, etc.

**Current**: ReluxScript has basic string methods. May need extensions:
- `s.split(",")` → `Vec<Str>`
- `s.replace("old", "new")` → `Str`
- `s.slice(start, end)` → `Str`

#### F. AST Node Field Access
Need consistent field names across Babel/SWC.

**Solution**: Already handled by ReluxScript's U-AST mapping (Section 7).
Must add JSX-specific fields:
- `JSXElement.openingElement.name` → tag name
- `JSXAttribute.name.name` → attribute name
- `JSXExpressionContainer.expression` → inner expression

#### G. Parent/Scope Traversal
babel-plugin-minimact uses `path.parent`, `path.getFunctionParent()`.

**Solution**: Manual tracking via visitor state:

```reluxscript
writer MinimactTranspiler {
    current_component: Option<Str>,
    parent_stack: Vec<NodeType>,

    fn visit_function_declaration(node: &FunctionDeclaration, ctx: &Context) {
        self.current_component = Some(node.id.name.clone());
        self.parent_stack.push(NodeType::FunctionDeclaration);

        node.visit_children(self);

        self.parent_stack.pop();
        self.current_component = None;
    }
}
```

### 4.3 Minor Gaps

#### H. Type Annotations
Access to TypeScript type annotations for C# type mapping.

**Solution**: Already in spec (Section 7.2). Ensure complete mapping:
- `TSStringKeyword` → `string`
- `TSNumberKeyword` → `double` or `int`
- `TSBooleanKeyword` → `bool`
- `TSTypeReference` → custom type name
- `TSArrayType` → `List<T>`

#### I. Generator/Async Functions
`useServerTask` extracts async functions.

**Solution**: ReluxScript doesn't support async, but can detect `async` flag:

```reluxscript
fn visit_function_declaration(node: &FunctionDeclaration, ctx: &Context) {
    if node.async {
        // Generate C# async Task method
    }
}
```

---

## 5. Implementation Strategy

### Phase 1: Core Transpiler (Week 1-2)

1. **Set up ReluxScript project structure**
   ```
   minimact-transpiler/
   ├── src/
   │   └── main.rsc          # Main writer
   ├── lib/
   │   ├── extractors.rsc    # Hook/prop extraction
   │   ├── generators.rsc    # C# code generation
   │   └── types.rsc         # Type conversion
   └── tests/
   ```

2. **Implement basic component detection**
   - `visit_function_declaration`
   - Check for PascalCase name
   - Check for JSX return

3. **Implement useState/useEffect extraction**
   - Pattern match `const [x, setX] = useState(...)`
   - Extract initial values and types

4. **Implement basic JSX → VNode**
   - `VElement` with tag, props, children
   - `VText` for text nodes
   - Event handler references

5. **Generate C# class skeleton**
   - `public class X : MinimactComponent`
   - `[State]` fields
   - `Render()` method

### Phase 2: Complete Feature Parity (Week 3-4)

1. **All hook types** (20+ hooks)
2. **Template extraction** (text, attribute, loop, conditional)
3. **TypeScript interface support**
4. **Custom hook handling**
5. **Lifted state pattern**

### Phase 3: Multi-File Output & Hot Reload (Week 5)

1. **Implement multi-file output system**
2. **Generate `.templates.json`**
3. **Generate `.hooks.json`**
4. **Generate `.structural-changes.json`**
5. **Generate `.tsx.keys`**

### Phase 4: Testing & Validation (Week 6)

1. **Port existing test cases**
2. **Compare output with JS plugin**
3. **Performance benchmarking**
4. **Edge case handling**

---

## 6. Code Examples

### 6.1 Main Writer Structure

```reluxscript
/// Minimact TSX to C# Transpiler
writer MinimactTranspiler {
    // Output builders
    csharp: CodeBuilder,
    templates: HashMap<Str, Template>,
    hooks: Vec<HookSignature>,

    // Traversal state
    current_component: Option<ComponentInfo>,
    parent_stack: Vec<NodeType>,

    fn init() -> Self {
        Self {
            csharp: CodeBuilder::new(),
            templates: HashMap::new(),
            hooks: vec![],
            current_component: None,
            parent_stack: vec![],
        }
    }

    fn visit_function_declaration(node: &FunctionDeclaration, ctx: &Context) {
        let name = node.id.name.clone();

        // Check if this is a component (PascalCase)
        if !is_pascal_case(&name) {
            return;
        }

        // Initialize component info
        let mut component = ComponentInfo::new(name.clone());

        // Extract props from parameters
        extract_props(node.params, &mut component);

        // Traverse body to extract hooks
        traverse(node.body) {
            fn visit_call_expression(call: &CallExpression, ctx: &Context) {
                extract_hook(call, &mut component);
            }

            fn visit_return_statement(ret: &ReturnStatement, ctx: &Context) {
                if let Some(jsx) = get_jsx_element(ret.argument) {
                    component.render_body = Some(jsx);
                }
            }
        }

        // Generate C# code
        self.generate_csharp_class(&component);

        // Extract templates for hot reload
        self.extract_templates(&component);
    }

    fn finish(self) -> TranspilerOutput {
        TranspilerOutput {
            csharp: self.csharp.to_string(),
            templates: json::to_string_pretty(&self.templates).unwrap(),
            hooks: json::to_string_pretty(&self.hooks).unwrap(),
        }
    }
}
```

### 6.2 Hook Extraction

```reluxscript
fn extract_hook(call: &CallExpression, component: &mut ComponentInfo) {
    if let Some(name) = get_identifier_name(&call.callee) {
        match name.as_str() {
            "useState" => extract_use_state(call, component),
            "useEffect" => extract_use_effect(call, component),
            "useRef" => extract_use_ref(call, component),
            "useClientState" => extract_use_state(call, component), // Same pattern
            "useServerTask" => extract_use_server_task(call, component),
            _ => {
                // Check for custom hooks
                if name.starts_with("use") {
                    extract_custom_hook_call(call, component, &name);
                }
            }
        }
    }
}

fn extract_use_state(call: &CallExpression, component: &mut ComponentInfo) {
    // Get parent variable declarator: const [x, setX] = useState(...)
    let parent = get_variable_declarator(call);

    if let Some(array_pattern) = get_array_pattern(&parent.id) {
        let state_var = array_pattern.elements[0].clone();
        let setter_var = array_pattern.elements.get(1).cloned();
        let initial_value = call.arguments.get(0);

        // Check for explicit type parameter: useState<string>(...)
        let explicit_type = get_type_parameter(call);
        let inferred_type = explicit_type.unwrap_or_else(|| infer_type(initial_value));

        component.use_state.push(StateInfo {
            name: state_var,
            setter: setter_var,
            initial_value: generate_csharp_expr(initial_value),
            state_type: inferred_type,
        });
    }
}
```

### 6.3 C# Generation

```reluxscript
fn generate_csharp_class(component: &ComponentInfo) {
    let builder = &mut self.csharp;

    builder.append("using Minimact.Core;\n");
    builder.append("using Minimact.VDom;\n\n");

    builder.append(&format!("public class {} : MinimactComponent\n", component.name));
    builder.append("{\n");
    builder.indent();

    // Generate state fields
    for state in &component.use_state {
        builder.append(&format!(
            "[State] private {} {} = {};\n",
            state.state_type,
            state.name,
            state.initial_value
        ));
    }

    builder.newline();

    // Generate event handlers
    for handler in &component.event_handlers {
        generate_event_handler(builder, handler);
    }

    builder.newline();

    // Generate Render method
    builder.append("protected override VNode Render()\n");
    builder.append("{\n");
    builder.indent();

    if let Some(jsx) = &component.render_body {
        let vnode_code = generate_vnode(jsx, component);
        builder.append(&format!("return {};\n", vnode_code));
    } else {
        builder.append("return new VNull();\n");
    }

    builder.dedent();
    builder.append("}\n");

    builder.dedent();
    builder.append("}\n");
}
```

---

## 7. Required ReluxScript Enhancements

### 7.1 Must Have

1. **Multi-file output for writers**
   - Return `HashMap<Str, Str>` or `MultiFileOutput` struct
   - Host handles file writing

2. **Complete JSX AST node mapping**
   - `JSXElement`, `JSXAttribute`, `JSXText`, `JSXExpressionContainer`
   - All field accessors

3. **String slice/substring**
   - `s.slice(start, end)` → `Str`
   - Or `s.chars().skip(start).take(end-start).collect()`

4. **TypeScript AST node mapping**
   - `TSInterfaceDeclaration`, `TSPropertySignature`
   - `TSTypeAnnotation`, `TSTypeReference`

### 7.2 Nice to Have

1. **Improved string manipulation**
   - `split`, `replace`, `trim_start`, `trim_end`

2. **Regex support** (limited)
   - Basic pattern matching for key extraction

3. **Debug logging**
   - `console.log` equivalent for development

---

## 8. Migration Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Core Setup | Project structure, basic component detection |
| 2 | Core Transpiler | useState/useEffect, basic JSX, C# skeleton |
| 3 | Full Hooks | All 20+ hook types |
| 4 | Templates | Template extraction, TypeScript support |
| 5 | Multi-File | JSON outputs, structural changes |
| 6 | Testing | Validation, benchmarks, documentation |

---

## 9. Success Criteria

1. **Feature parity**: All babel-plugin-minimact features work
2. **Output compatibility**: Generated C# matches current output
3. **Dual target**: Compiles to both Babel and SWC
4. **Performance**: SWC target is faster than Babel target
5. **Test coverage**: All existing tests pass

---

## 10. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex regex patterns | High | Use string methods + manual parsing |
| File system access in SWC | Medium | Multi-file output returned to host |
| Parent traversal patterns | Medium | Manual state tracking |
| Missing AST nodes | High | Extend ReluxScript mappings |
| Edge cases in type inference | Low | Fallback to `dynamic` type |

---

## 11. Next Steps

1. **Review this plan** with stakeholders
2. **Prioritize ReluxScript enhancements** needed
3. **Set up minimact-transpiler.rsc** project
4. **Port simplest component test** as proof of concept
5. **Iterate** through features

---

*Generated by Claude Code - Migration planning document for babel-plugin-minimact to ReluxScript*
