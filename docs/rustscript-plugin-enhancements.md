# ReluxScript Enhancements for Complex Plugin Support

This document outlines the language and compiler enhancements needed for ReluxScript to support complex AST transformation plugins like the one in `babel-plugin/generated-swc`.

## Overview

The target plugin is a ~3000 line SWC plugin that:
- Parses React/TSX components
- Extracts hooks (useState, useEffect, useRef, custom hooks, etc.)
- Generates C# output classes
- Produces template JSON for runtime binding
- Handles multiple specialized visitors for different extraction passes

Converting this to ReluxScript requires six key enhancements.

---

## 1. Nested Traverse

### Current Limitation

ReluxScript's `traverse` construct only works at the top level of a visitor function. The Minimact pattern requires nested traversals with different visitors:

```rust
// SWC Pattern - Multiple specialized visitors
fn process_component(&mut self, func: &mut FnDecl) {
    if let Some(body) = &mut func.function.body {
        // First pass: Extract hooks
        let mut hook_extractor = HookExtractor { component: &mut component };
        body.visit_mut_with(&mut hook_extractor);

        // Second pass: Extract local variables
        let mut var_extractor = LocalVariableExtractor { component: &mut component };
        body.visit_mut_with(&mut var_extractor);

        // Third pass: Extract templates
        let mut template_extractor = TemplateExtractor { component: &mut component };
        body.visit_mut_with(&mut template_extractor);
    }
}
```

### Proposed Enhancement

Support nested `traverse` blocks with local state:

```reluxscript
pub fn visit_function_declaration(node: &FunctionDeclaration) {
    let mut component = Component::new(node.id.name.clone());

    if let Some(body) = &node.body {
        // First pass: hooks
        traverse(body) {
            let hook_count = 0;  // Local state for this traversal

            fn visit_variable_declarator(decl: &VariableDeclarator) {
                if is_hook_call(&decl.init) {
                    extract_hook(&decl, &mut component);
                    hook_count += 1;
                }
            }
        }

        // Second pass: templates (separate traversal)
        traverse(body) {
            let mut current_path = vec![];

            fn visit_jsx_element(jsx: &JSXElement) {
                extract_templates(&jsx, &current_path, &mut component);
                current_path.push(0);
                // Recurse into children
                for child in &jsx.children {
                    traverse(child) using self;  // Continue with same visitor
                }
                current_path.pop();
            }
        }
    }
}
```

### Implementation Notes

**Babel Codegen:**
```javascript
// Each traverse block becomes a path.traverse call
path.traverse({
  VariableDeclarator(innerPath) {
    // Hook extraction logic
  }
});

path.traverse({
  JSXElement(innerPath) {
    // Template extraction logic
  }
});
```

**SWC Codegen:**
```rust
// Each traverse block becomes a separate visitor struct
struct HookExtractorVisitor<'a> {
    component: &'a mut Component,
    hook_count: i32,
}

impl VisitMut for HookExtractorVisitor<'_> {
    fn visit_mut_var_declarator(&mut self, decl: &mut VarDeclarator) {
        // Hook extraction logic
    }
}

body.visit_mut_with(&mut HookExtractorVisitor {
    component: &mut component,
    hook_count: 0,
});
```

---

## 2. Mutable References in Visitor Callbacks

### Current Limitation

ReluxScript doesn't fully support `&mut` references being captured and used in nested contexts. Minimact heavily relies on passing mutable component references to extractors.

### Proposed Enhancement

Support explicit mutable captures in traverse blocks:

```reluxscript
pub fn process_component(func: &FunctionDeclaration) {
    let mut component = Component::new(func.id.name.clone());
    let mut template_count = 0;

    // Explicit capture syntax
    traverse(func.body) capturing [&mut component, &mut template_count] {
        fn visit_jsx_element(jsx: &JSXElement) {
            component.templates.insert(/* ... */);
            template_count += 1;
        }
    }

    // component and template_count are still accessible here
    if template_count > 0 {
        generate_template_json(&component);
    }
}
```

### Alternative: Implicit Capture with Ownership Rules

```reluxscript
traverse(func.body) {
    // Compiler infers that component needs &mut capture
    fn visit_jsx_element(jsx: &JSXElement) {
        component.templates.insert(/* ... */);  // component borrowed mutably
    }
}
// component ownership returns here
```

### Implementation Notes

**SWC Codegen** - The hoisted visitor struct includes captured references:
```rust
struct TraverseVisitor<'a> {
    component: &'a mut Component,
    template_count: &'a mut i32,
}
```

**Babel Codegen** - Closures naturally capture:
```javascript
const component = new Component(func.id.name);
let templateCount = 0;

path.traverse({
  JSXElement(innerPath) {
    component.templates.set(/* ... */);
    templateCount++;
  }
});
```

---

## 3. HashMap and HashSet Support

### Current Limitation

ReluxScript has `Vec` but lacks full support for `HashMap` and `HashSet` which are essential for:
- Template storage by path key
- Tracking external imports
- State type mappings
- Deduplication

### Proposed Enhancement

Add first-class HashMap and HashSet support:

```reluxscript
plugin MinimactTransformer {
    struct Component {
        name: Str,
        templates: HashMap<Str, Template>,
        state_types: HashMap<Str, Str>,
        external_imports: HashSet<Str>,
    }

    pub fn visit_jsx_element(jsx: &JSXElement) {
        let path = build_jsx_path(jsx);

        // HashMap operations
        if !component.templates.contains_key(&path) {
            component.templates.insert(path.clone(), Template {
                path: path,
                bindings: vec![],
            });
        }

        // Get with default
        let template = component.templates.get(&path).unwrap_or_default();

        // Iteration
        for (key, value) in &component.templates {
            process_template(key, value);
        }
    }

    pub fn visit_import_declaration(import: &ImportDeclaration) {
        let source = import.source.value.clone();

        // HashSet operations
        if !source.starts_with(".") && !source.starts_with("minimact") {
            component.external_imports.insert(source);
        }

        // Check membership
        if component.external_imports.contains(&"react") {
            // ...
        }
    }
}
```

### Required Operations

**HashMap<K, V>:**
- `new()` - Create empty map
- `insert(key, value)` - Insert or update
- `get(&key)` - Get Option<&V>
- `get_mut(&key)` - Get Option<&mut V>
- `contains_key(&key)` - Check existence
- `remove(&key)` - Remove and return
- `len()` - Get count
- `is_empty()` - Check if empty
- `keys()` - Iterate keys
- `values()` - Iterate values
- `iter()` - Iterate (key, value) pairs

**HashSet<T>:**
- `new()` - Create empty set
- `insert(value)` - Add value
- `contains(&value)` - Check membership
- `remove(&value)` - Remove value
- `len()` - Get count
- `is_empty()` - Check if empty
- `iter()` - Iterate values

### Codegen Mapping

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `HashMap<K,V>` | `new Map()` | `HashMap<K,V>` |
| `HashSet<T>` | `new Set()` | `HashSet<T>` |
| `map.insert(k,v)` | `map.set(k,v)` | `map.insert(k,v)` |
| `map.get(&k)` | `map.get(k)` | `map.get(&k)` |
| `set.insert(v)` | `set.add(v)` | `set.insert(v)` |
| `set.contains(&v)` | `set.has(v)` | `set.contains(&v)` |

---

## 4. String Formatting and Concatenation

### Current Limitation

ReluxScript lacks string interpolation and formatting, which Minimact uses extensively for:
- C# code generation
- Template string building
- Error messages
- JSON paths

### Proposed Enhancement

Add format strings and string concatenation:

```reluxscript
// Format strings (Rust-style)
let class_decl = format!("public class {} : MinimactComponent", component.name);
let property = format!("    public {} {} {{ get; set; }}", prop_type, prop_name);

// Multi-line format
let code = format!(
    "using System;\n\
     using Minimact;\n\n\
     public class {} : MinimactComponent\n\
     {{\n\
         {}\n\
     }}",
    component.name,
    properties.join("\n")
);

// String concatenation (for simple cases)
let path = parent_path + "." + index.to_string();
let message = "Error in component: " + &component.name;

// String builder pattern for complex generation
let mut code = String::new();
code.push_str("using System;\n");
code.push_str(&format!("public class {} {{\n", name));
for prop in &props {
    code.push_str(&format!("    public {} {};\n", prop.prop_type, prop.name));
}
code.push_str("}\n");
```

### Required Operations

**String:**
- `format!(pattern, args...)` - Interpolated string
- `+` operator - Concatenation (with &str)
- `push_str(&str)` - Append to mutable string
- `to_string()` - Convert to String
- `as_str()` - Get &str reference

### Codegen Mapping

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `format!("x={}", x)` | `` `x=${x}` `` | `format!("x={}", x)` |
| `a + &b` | `a + b` | `format!("{}{}", a, b)` |
| `s.push_str(&t)` | `s += t` | `s.push_str(&t)` |

---

## 5. File I/O Operations

### Current Limitation

ReluxScript has no file system access. Minimact needs to:
- Write generated C# files
- Write template JSON files
- Read imported hook metadata

### Proposed Enhancement

Add file I/O operations in a `fs` module:

```reluxscript
use fs;

pub fn generate_outputs(component: &Component, output_dir: &Str) {
    // Write C# file
    let cs_code = generate_csharp_code(component);
    let cs_path = format!("{}/{}.cs", output_dir, component.name);

    match fs::write(&cs_path, &cs_code) {
        Ok(_) => {
            log(&format!("Generated {}", cs_path));
        }
        Err(e) => {
            log_error(&format!("Failed to write {}: {}", cs_path, e));
        }
    }

    // Write templates JSON
    if !component.templates.is_empty() {
        let json = generate_templates_json(component);
        let json_path = format!("{}/{}.templates.json", output_dir, component.name);
        fs::write(&json_path, &json)?;
    }

    // Read file
    let content = fs::read_to_string(&path)?;

    // Check existence
    if fs::exists(&path) {
        // ...
    }

    // Create directory
    fs::create_dir_all(&output_dir)?;
}
```

### Required Operations

**fs module:**
- `write(path, content)` - Write string to file
- `read_to_string(path)` - Read file to string
- `exists(path)` - Check if path exists
- `create_dir_all(path)` - Create directory recursively
- `remove_file(path)` - Delete file

### Codegen Mapping

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `fs::write(p, c)` | `fs.writeFileSync(p, c)` | `std::fs::write(p, c)` |
| `fs::read_to_string(p)` | `fs.readFileSync(p, 'utf8')` | `std::fs::read_to_string(p)` |
| `fs::exists(p)` | `fs.existsSync(p)` | `std::path::Path::new(p).exists()` |
| `fs::create_dir_all(p)` | `fs.mkdirSync(p, {recursive:true})` | `std::fs::create_dir_all(p)` |

---

## 6. JSON Serialization

### Current Limitation

ReluxScript has no JSON support. Minimact generates:
- `.templates.json` - Template bindings
- `.timeline-templates.json` - Animation timelines
- Hook metadata for imports

### Proposed Enhancement

Add JSON serialization with a `json` module:

```reluxscript
use json;

// Derive serialization
#[derive(Serialize, Deserialize)]
struct Template {
    path: Str,
    template: Str,
    bindings: Vec<Str>,
}

#[derive(Serialize)]
struct TemplateOutput {
    component_name: Str,
    templates: HashMap<Str, Template>,
    loop_templates: Vec<LoopTemplate>,
}

pub fn generate_templates_json(component: &Component) -> Str {
    let output = TemplateOutput {
        component_name: component.name.clone(),
        templates: component.templates.clone(),
        loop_templates: component.loop_templates.clone(),
    };

    // Serialize to JSON string
    return json::to_string_pretty(&output).unwrap();
}

pub fn load_hook_metadata(path: &Str) -> Option<HookMetadata> {
    let content = fs::read_to_string(path)?;

    // Deserialize from JSON
    match json::from_str::<HookMetadata>(&content) {
        Ok(metadata) => Some(metadata),
        Err(_) => None,
    }
}

// Manual JSON building (for dynamic structures)
pub fn build_template_json(component: &Component) -> Str {
    let mut obj = json::object();

    obj.insert("componentName", json::string(&component.name));

    let mut templates = json::object();
    for (key, template) in &component.templates {
        let mut t = json::object();
        t.insert("path", json::string(&template.path));
        t.insert("template", json::string(&template.template));
        t.insert("bindings", json::array(&template.bindings));
        templates.insert(key, t);
    }
    obj.insert("templates", templates);

    return json::stringify_pretty(&obj);
}
```

### Required Operations

**json module:**
- `to_string(&value)` - Serialize to compact JSON
- `to_string_pretty(&value)` - Serialize with formatting
- `from_str<T>(&str)` - Deserialize from JSON
- `object()` - Create empty object
- `array()` - Create empty array
- `string(&str)` - Create string value
- `number(n)` - Create number value
- `boolean(b)` - Create boolean value
- `null()` - Create null value

### Codegen Mapping

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `json::to_string(&v)` | `JSON.stringify(v)` | `serde_json::to_string(&v)` |
| `json::to_string_pretty(&v)` | `JSON.stringify(v,null,2)` | `serde_json::to_string_pretty(&v)` |
| `json::from_str(&s)` | `JSON.parse(s)` | `serde_json::from_str(&s)` |
| `#[derive(Serialize)]` | (automatic) | `#[derive(Serialize)]` |

---

## Implementation Priority

### Phase 1: Core Language (Required for basic Minimact)
1. **HashMap/HashSet** - Essential for template storage
2. **String formatting** - Essential for code generation
3. **Mutable references** - Essential for visitor patterns

### Phase 2: Advanced Features (Full Minimact support)
4. **Nested traverse** - Multi-pass AST analysis
5. **File I/O** - Output generation
6. **JSON serialization** - Template/metadata files

### Phase 3: Ergonomics
- Error handling with `?` operator
- Pattern matching enhancements
- Module system improvements

---

## Example: Minimact Hook Extractor in ReluxScript

With these enhancements, a real Minimact visitor would look like:

```reluxscript
use fs;
use json;

plugin MinimactTransformer {
    #[derive(Serialize)]
    struct Component {
        name: Str,
        props: Vec<Prop>,
        use_state: Vec<UseStateInfo>,
        templates: HashMap<Str, Template>,
        external_imports: HashSet<Str>,
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let name = node.id.name.clone();

        if !is_component_name(&name) {
            return;
        }

        let mut component = Component::new(name);

        // Extract props
        if node.params.len() > 0 {
            extract_props(&node.params[0], &mut component);
        }

        // Multi-pass extraction
        if let Some(body) = &node.body {
            // Pass 1: Hooks
            traverse(body) capturing [&mut component] {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, CallExpression) {
                            let call = init.clone();
                            if matches!(call.callee, Identifier) {
                                let callee_name = call.callee.name.clone();
                                match callee_name.as_str() {
                                    "useState" => extract_use_state(&call, &decl.id, &mut component),
                                    "useEffect" => extract_use_effect(&call, &mut component),
                                    "useRef" => extract_use_ref(&call, &decl.id, &mut component),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            // Pass 2: Templates
            traverse(body) capturing [&mut component] {
                let mut path_stack: Vec<usize> = vec![];

                fn visit_jsx_element(jsx: &JSXElement) {
                    let path = path_stack.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(".");

                    // Extract attribute templates
                    for attr in &jsx.opening.attrs {
                        if matches!(attr, JSXAttribute) {
                            extract_attribute_template(&attr, &path, &mut component);
                        }
                    }

                    // Recurse into children
                    for (i, child) in jsx.children.iter().enumerate() {
                        path_stack.push(i);
                        if matches!(child, JSXElement) {
                            // visit recursively
                        }
                        path_stack.pop();
                    }
                }
            }
        }

        // Generate outputs
        generate_outputs(&component);
    }

    fn generate_outputs(component: &Component) {
        // Generate C#
        let cs_code = generate_csharp(component);
        let cs_path = format!("output/{}.cs", component.name);
        fs::write(&cs_path, &cs_code).unwrap();

        // Generate templates JSON
        if !component.templates.is_empty() {
            let json_str = json::to_string_pretty(component).unwrap();
            let json_path = format!("output/{}.templates.json", component.name);
            fs::write(&json_path, &json_str).unwrap();
        }
    }

    fn generate_csharp(component: &Component) -> Str {
        let mut code = String::new();

        code.push_str("using System;\n");
        code.push_str("using Minimact;\n\n");
        code.push_str(&format!("public class {} : MinimactComponent\n{{\n", component.name));

        // Properties from state
        for state in &component.use_state {
            code.push_str(&format!(
                "    public {} {} {{ get; set; }} = {};\n",
                state.state_type,
                state.var_name,
                state.initial_value
            ));
        }

        code.push_str("}\n");

        return code;
    }
}
```

This represents the target state for ReluxScript to fully support real-world AST transformation plugins like Minimact.
