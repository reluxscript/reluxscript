# Visitor Pattern

The visitor pattern is the core concept in ReluxScript for traversing and transforming ASTs.

## Overview

The visitor pattern works by:
1. Traversing the AST tree
2. Calling visitor methods for matching node types
3. Allowing transformations at each node

## Basic Visitor

```reluxscript
plugin MyPlugin {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Called for every identifier in the code
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Called for every function call
    }
}
```

## Traversal Order

By default, traversal is **depth-first, pre-order**:

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // 1. This runs first

    // 2. Then children are visited automatically
    node.visit_children(self);

    // 3. Post-processing can go here
}
```

## Manual Traversal Control

### Skip Children

Don't visit child nodes:

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // Don't call visit_children() - children won't be visited
    // Useful for skipping certain subtrees
}
```

### Explicit Child Traversal

Manually control which children to visit:

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    for stmt in &mut node.body {
        if should_visit(stmt) {
            stmt.visit_with(self);
        }
    }
}
```

## Nested Visitors

Create scoped visitors for subtrees:

```reluxscript
fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
    // Use a nested visitor for the function body
    traverse(&mut func.body) {
        let mut return_count = 0;

        fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
            self.return_count += 1;
        }
    }

    println!("Function has {} returns", return_count);
}
```

## State Tracking

Track state across visitor methods:

```reluxscript
plugin StateTracker {
    struct State {
        depth: i32,
        in_async: bool,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        self.state.depth += 1;

        let was_async = self.state.in_async;
        self.state.in_async = node.async;

        node.visit_children(self);

        self.state.in_async = was_async;
        self.state.depth -= 1;
    }
}
```

## Multiple Visitors

Combine multiple transformations:

```reluxscript
plugin CompositePlugin {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // First transformation
        self.remove_console_logs(node);

        // Second transformation
        self.inline_simple_calls(node);

        node.visit_children(self);
    }

    fn remove_console_logs(&mut self, node: &mut CallExpression) {
        if matches!(node.callee, "console.log") {
            *node = Statement::empty();
        }
    }

    fn inline_simple_calls(&mut self, node: &mut CallExpression) {
        // ...
    }
}
```

## Using External Visitors

Delegate to another visitor:

```reluxscript
plugin Main {
    fn visit_function(node: &mut Function, ctx: &Context) {
        if node.is_async {
            // Use a specialized visitor for async functions
            traverse(node) using AsyncTransformer;
        }
    }
}

plugin AsyncTransformer {
    fn visit_await_expression(node: &mut AwaitExpression, ctx: &Context) {
        // Handle await expressions
    }
}
```

## Visitor Entry Points

### Regular Visitors

Most visitor methods:

```reluxscript
fn visit_<node_type>(node: &mut NodeType, ctx: &Context) {
    // Transform node
}
```

### Program Hooks

Special lifecycle methods:

```reluxscript
// Called before traversal begins
fn pre(file: &File) {
    // Initialize
}

// Called after traversal completes
fn exit(program: &mut Program, state: &PluginState) {
    // Finalize
}
```

## Best Practices

### 1. Keep Visitors Focused

```reluxscript
// ✅ Good: One concern per visitor
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if self.should_remove(node) {
        *node = Statement::empty();
    }
}

// ❌ Bad: Too much logic
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // 100 lines of complex logic
}
```

### 2. Use Helper Methods

```reluxscript
plugin MyPlugin {
    fn is_console_method(&self, callee: &Expression) -> bool {
        matches!(callee, MemberExpression {
            object: Identifier { name: "console" }
        })
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if self.is_console_method(&node.callee) {
            self.handle_console_call(node);
        }
    }
}
```

### 3. Track State Explicitly

```reluxscript
// ✅ Good: Explicit state
struct State {
    current_component: Option<Str>,
    hooks_found: Vec<HookInfo>,
}

// ❌ Bad: Implicit tracking (won't work)
// Don't rely on parent node context
```

### 4. Clean Up State

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // Save state
    let prev_component = self.state.current_component.clone();

    // Set new state
    self.state.current_component = Some(node.id.name.clone());

    // Visit children
    node.visit_children(self);

    // Restore state
    self.state.current_component = prev_component;
}
```

## Common Patterns

### Count Occurrences

```reluxscript
struct State {
    call_count: i32,
}

fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    self.state.call_count += 1;
    node.visit_children(self);
}
```

### Collect Information

```reluxscript
struct State {
    identifiers: Vec<Str>,
}

fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    self.state.identifiers.push(node.name.clone());
}
```

### Conditional Transformation

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if self.should_transform(node) {
        *node = self.create_replacement(node);
    } else {
        node.visit_children(self);
    }
}
```

See [Visitor Methods API](/v0.1/api/visitor-methods) for all available methods.
