# Hook Analyzer Example

Analyze and report React Hook usage patterns in components.

## Code

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

    fn exit(program: &mut Program, state: &PluginState) {
        println!("Hook Usage Report:");
        for hook in &self.state.hooks {
            println!("  {} uses {} ({})", hook.component, hook.name, hook.hook_type);
        }
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

## Input

```javascript
function Counter() {
    const [count, setCount] = useState(0);
    const [name, setName] = useState("Counter");

    useEffect(() => {
        console.log("Count changed:", count);
    }, [count]);

    return <div>{count}</div>;
}
```

## Output

```
Hook Usage Report:
  Counter uses useState (state)
  Counter uses useState (state)
  Counter uses useEffect (effect)
```

## How It Works

1. Track the current component name when entering function declarations
2. Check if the function name starts with uppercase (component convention)
3. When visiting call expressions, check if they start with "use" (hook convention)
4. Collect hook information with component context
5. On exit, generate a report of all hook usage

[Back to Examples](/v0.1/examples/)
