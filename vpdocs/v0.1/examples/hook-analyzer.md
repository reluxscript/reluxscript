# Hook Analyzer Example

Analyze and report React Hook usage patterns in components.

## Code

```reluxscript
plugin HookAnalyzer {
    struct State {
        use_state_count: i32,
        use_effect_count: i32,
        use_memo_count: i32,
        use_callback_count: i32,
        use_ref_count: i32,
        custom_hooks_count: i32,
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if this is a hook call (function name starts with "use")
        if let Callee::Identifier(ref ident) = node.callee {
            if Regex::matches(&ident.name, r"^use[A-Z]") {
                // Count specific built-in hooks
                if ident.name == "useState" {
                    self.state.use_state_count = self.state.use_state_count + 1;
                } else if ident.name == "useEffect" {
                    self.state.use_effect_count = self.state.use_effect_count + 1;
                } else if ident.name == "useMemo" {
                    self.state.use_memo_count = self.state.use_memo_count + 1;
                } else if ident.name == "useCallback" {
                    self.state.use_callback_count = self.state.use_callback_count + 1;
                } else if ident.name == "useRef" {
                    self.state.use_ref_count = self.state.use_ref_count + 1;
                } else {
                    // Custom hook
                    self.state.custom_hooks_count = self.state.custom_hooks_count + 1;
                    node.__customHookName = ident.name.clone();
                }
            }
        }
    }

    fn exit(program: &mut Program, ctx: &Context) {
        println!("Hook Usage Report:");
        println!("  useState: {}", self.state.use_state_count);
        println!("  useEffect: {}", self.state.use_effect_count);
        println!("  useMemo: {}", self.state.use_memo_count);
        println!("  useCallback: {}", self.state.use_callback_count);
        println!("  useRef: {}", self.state.use_ref_count);
        println!("  Custom hooks: {}", self.state.custom_hooks_count);
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

    const memoValue = useMemo(() => count * 2, [count]);
    const callback = useCallback(() => setCount(0), []);
    const ref = useRef(null);
    const data = useCustomData();

    return <div>{count}</div>;
}
```

## Output

```
Hook Usage Report:
  useState: 2
  useEffect: 1
  useMemo: 1
  useCallback: 1
  useRef: 1
  Custom hooks: 1
```

## How It Works

1. For each call expression, check if callee is an identifier
2. Use `Regex::matches()` to check if the name matches hook pattern (`^use[A-Z]`)
3. Count occurrences of each built-in hook type (useState, useEffect, etc.)
4. Mark custom hooks with `__customHookName` property for further analysis
5. In `exit()`, print a summary report of all hook usage statistics

[Back to Examples](/v0.1/examples/)
