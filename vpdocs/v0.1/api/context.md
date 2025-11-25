# Context API

The `Context` object provides utilities and information during AST traversal.

## Overview

Every visitor method receives a `Context` parameter:

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // Use ctx here
}
```

## Properties

### ctx.filename

The current source filename.

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    println!("Processing: {}", ctx.filename);
}
```

**Type:** `Str`

## Scope Methods

::: warning Performance Note
Scope operations are cheap in Babel but expensive in SWC (requires pre-pass analysis). Use sparingly or track bindings manually.
:::

### ctx.scope.has_binding(name)

Check if a name is bound in the current scope.

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    if ctx.scope.has_binding(&node.name) {
        // This identifier is declared in scope
    } else {
        // This is a free variable (global or undefined)
    }
}
```

**Parameters:**
- `name: &Str` - The identifier name to check

**Returns:** `bool`

### ctx.scope.get_binding(name)

Get detailed binding information.

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    if let Some(binding) = ctx.scope.get_binding(&node.name) {
        println!("Binding kind: {}", binding.kind);
        // "var", "let", "const", "param", etc.
    }
}
```

**Parameters:**
- `name: &Str` - The identifier name

**Returns:** `Option<Binding>`

## Utility Methods

### ctx.generate_uid(hint)

Generate a unique identifier name that doesn't conflict with existing names.

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    let temp_name = ctx.generate_uid("temp");
    // Returns "_temp", "_temp2", etc.

    // Use the unique name
    let temp_var = Identifier::new(&temp_name);
}
```

**Parameters:**
- `hint: &Str` - Suggested base name

**Returns:** `Str` - A unique identifier name

## Examples

### Check if Global Reference

```reluxscript
fn is_global_reference(node: &Identifier, ctx: &Context) -> bool {
    !ctx.scope.has_binding(&node.name)
}
```

### Generate Temporary Variable

```reluxscript
fn create_temp_variable(ctx: &Context, init: Expression) -> VariableDeclaration {
    let name = ctx.generate_uid("tmp");

    VariableDeclaration {
        kind: "const",
        declarations: vec![
            VariableDeclarator {
                id: Pattern::Identifier(Identifier::new(&name)),
                init: Some(init),
            }
        ],
    }
}
```

### Track Scope Manually (Better Performance)

Instead of using `ctx.scope` (expensive in SWC), track bindings yourself:

```reluxscript
plugin MyPlugin {
    struct State {
        bindings: HashSet<Str>,
    }

    fn visit_variable_declarator(node: &mut VariableDeclarator, ctx: &Context) {
        if let Pattern::Identifier(id) = &node.id {
            self.state.bindings.insert(id.name.clone());
        }
        node.visit_children(self);
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Use our tracked bindings instead of ctx.scope
        if self.state.bindings.contains(&node.name) {
            // Is declared in our scope
        }
    }
}
```

## Best Practices

### 1. Avoid ctx.scope in Hot Paths

```reluxscript
// ❌ Bad: Expensive scope lookup in every identifier
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    if ctx.scope.has_binding(&node.name) {
        // ...
    }
}

// ✅ Good: Track bindings manually
struct State {
    declared: HashSet<Str>,
}
```

### 2. Use generate_uid for Unique Names

```reluxscript
// ✅ Good: Guaranteed unique
let name = ctx.generate_uid("temp");

// ❌ Bad: May conflict
let name = "_temp";
```

### 3. Cache Scope Lookups

```reluxscript
// ✅ Good: Look up once
let is_bound = ctx.scope.has_binding(&node.name);
if is_bound {
    // Use multiple times
}

// ❌ Bad: Multiple lookups
if ctx.scope.has_binding(&node.name) {
    // ...
}
if ctx.scope.has_binding(&node.name) {
    // ... (looked up again)
}
```

## See Also

- [Visitor Methods](/v0.1/api/visitor-methods)
- [Node Constructors](/v0.1/api/node-constructors)
- [Core Concepts](/v0.1/guide/concepts)
