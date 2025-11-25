# Examples

Learn ReluxScript by example. Each example demonstrates a common AST transformation pattern.

## Basic Examples

### Remove Console Logs

Remove all `console.log()` calls from your code.

```reluxscript
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            *node = Statement::empty();
        }
    }
}
```

**Input:**
```javascript
console.log("debug");
doWork();
console.log("more debug");
```

**Output:**
```javascript
doWork();
```

[Learn more →](/v0.1/examples/remove-console)

---

### Transform Arrow Functions

Convert arrow functions to regular functions.

```reluxscript
plugin ArrowToFunction {
    fn visit_arrow_function(node: &mut ArrowFunctionExpression, ctx: &Context) {
        *node = FunctionExpression {
            id: None,
            params: node.params.clone(),
            body: node.body.clone(),
            async: node.async,
            generator: false,
        };
    }
}
```

**Input:**
```javascript
const add = (a, b) => a + b;
```

**Output:**
```javascript
const add = function(a, b) { return a + b; };
```

[Learn more →](/v0.1/examples/arrow-functions)

---

## React Examples

### JSX Key Checker

Warn about missing `key` props in JSX arrays.

```reluxscript
plugin JSXKeyChecker {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Check if this JSX element is in an array
        if is_in_array_context(ctx) {
            // Check for key attribute
            let has_key = node.opening_element.attributes.iter()
                .any(|attr| matches!(attr.name, "key"));

            if !has_key {
                ctx.warn("JSX element in array should have a key prop");
            }
        }
    }
}
```

[Learn more →](/v0.1/examples/jsx-keys)

---

### Hook Usage Analyzer

Analyze and report React Hook usage patterns.

```reluxscript
plugin HookAnalyzer {
    struct State {
        hooks: Vec<HookInfo>,
        current_component: Option<Str>,
    }

    struct HookInfo {
        name: Str,
        hook_type: Str,
        component: Str,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = node.id.name.clone();
        if is_component_name(&name) {
            self.state.current_component = Some(name);
        }
        node.visit_children(self);
        self.state.current_component = None;
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Some(component) = &self.state.current_component {
            if let Some(name) = get_callee_name(&node.callee) {
                if name.starts_with("use") {
                    self.state.hooks.push(HookInfo {
                        name: name.clone(),
                        hook_type: categorize_hook(&name),
                        component: component.clone(),
                    });
                }
            }
        }
        node.visit_children(self);
    }
}

fn is_component_name(name: &Str) -> bool {
    let first = name.chars().next();
    first.map(|c| c.is_uppercase()).unwrap_or(false)
}

fn get_callee_name(callee: &Expression) -> Option<Str> {
    match callee {
        Expression::Identifier(id) => Some(id.name.clone()),
        _ => None,
    }
}

fn categorize_hook(name: &Str) -> Str {
    match name.as_str() {
        "useState" | "useReducer" => "state",
        "useEffect" | "useLayoutEffect" => "effect",
        "useRef" => "ref",
        "useMemo" | "useCallback" => "memoization",
        _ => "custom",
    }
}
```

[Learn more →](/v0.1/examples/hook-analyzer)

---

## Advanced Examples

### Nested Visitor Pattern

Use nested visitors for complex transformations.

```reluxscript
plugin ComplexTransform {
    fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
        // Process function body with a nested visitor
        for stmt in &mut func.body.stmts {
            if stmt.is_if_statement() {
                traverse(stmt) {
                    let mut return_count = 0;

                    fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
                        self.return_count += 1;
                    }
                }
            }
        }
    }
}
```

---

### State Tracking

Track state across multiple visitor methods.

```reluxscript
plugin StateTracker {
    struct State {
        in_async_function: bool,
        async_operations: Vec<Str>,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let was_async = self.state.in_async_function;
        self.state.in_async_function = node.async;

        node.visit_children(self);

        self.state.in_async_function = was_async;
    }

    fn visit_await_expression(node: &mut AwaitExpression, ctx: &Context) {
        if self.state.in_async_function {
            let operation = get_operation_name(&node.argument);
            self.state.async_operations.push(operation);
        }
    }
}
```

---

## Real-World Examples

### Import Organizer

Organize and sort import statements.

```reluxscript
plugin ImportOrganizer {
    struct State {
        imports: Vec<ImportDeclaration>,
    }

    fn visit_import_declaration(node: &mut ImportDeclaration, ctx: &Context) {
        self.state.imports.push(node.clone());
        *node = Statement::empty();
    }

    fn exit(program: &mut Program, state: &PluginState) {
        // Sort imports
        self.state.imports.sort_by(|a, b| {
            a.source.value.cmp(&b.source.value)
        });

        // Insert at top of program
        for import in self.state.imports.drain(..).rev() {
            program.body.insert(0, Statement::ImportDeclaration(import));
        }
    }
}
```

---

### Dead Code Elimination

Remove unused variables and functions.

```reluxscript
plugin DeadCodeEliminator {
    struct State {
        declared: HashSet<Str>,
        used: HashSet<Str>,
    }

    fn visit_variable_declarator(node: &mut VariableDeclarator, ctx: &Context) {
        if let Pattern::Identifier(id) = &node.id {
            self.state.declared.insert(id.name.clone());
        }
        node.visit_children(self);
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        self.state.used.insert(node.name.clone());
    }

    fn exit(program: &mut Program, state: &PluginState) {
        // Find unused declarations
        let unused: HashSet<_> = self.state.declared
            .difference(&self.state.used)
            .cloned()
            .collect();

        // Remove them (implementation details omitted)
        remove_unused_declarations(program, &unused);
    }
}
```

---

## Pattern Library

Common patterns you can reuse:

### Check if identifier is a specific name
```reluxscript
if matches!(node.callee, Identifier { name: "console" }) {
    // ...
}
```

### Get string value from literal
```reluxscript
fn get_string_value(node: &Expression) -> Option<Str> {
    match node {
        Expression::StringLiteral(lit) => Some(lit.value.clone()),
        _ => None,
    }
}
```

### Clone and modify node
```reluxscript
let mut new_node = node.clone();
new_node.name = "modified";
*node = new_node;
```

### Iterate over children
```reluxscript
for stmt in &mut node.body.statements {
    // Process each statement
}
```

---

## Contributing Examples

Have a useful ReluxScript example? Contribute it to the repository!

1. Create a new `.lux` file in `examples/`
2. Add test cases
3. Document the use case
4. Submit a pull request

---

## Next Steps

- [Read the API Reference](/v0.1/api/visitor-methods)
- [Learn Core Concepts](/v0.1/guide/concepts)
- [View the Language Specification](/v0.1/language/specification)
