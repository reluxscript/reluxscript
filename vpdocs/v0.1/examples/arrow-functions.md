# Arrow Functions Example

Track and analyze arrow function usage in your code.

## Code

```reluxscript
plugin ArrowFunctionAnalyzer {
    struct State {
        arrow_count: i32,
        async_arrow_count: i32,
    }

    fn visit_arrow_function_expression(node: &mut ArrowFunctionExpression, ctx: &Context) {
        // Track total arrow functions
        self.state.arrow_count = self.state.arrow_count + 1;

        // Track async arrow functions
        if node.async_ {
            self.state.async_arrow_count = self.state.async_arrow_count + 1;
        }

        // Mark arrow functions with custom property for later processing
        node.__isArrowFunction = true;
    }

    fn exit(program: &mut Program, ctx: &Context) {
        // Report statistics
        println!("Found {} arrow functions", self.state.arrow_count);
        println!("  {} are async", self.state.async_arrow_count);
    }
}
```

## Input

```javascript
const add = (a, b) => a + b;
const fetchData = async () => {
    return await fetch('/api/data');
};
const log = (msg) => {
    console.log(msg);
};
```

## Output

```
Found 3 arrow functions
  1 are async
```

## How It Works

1. `visit_arrow_function_expression` is called for every arrow function
2. Increment the `arrow_count` in plugin state
3. Check `node.async_` field to track async arrow functions
4. Use custom AST property `__isArrowFunction` to mark nodes for later analysis
5. In `exit()`, print statistics about arrow function usage

[Back to Examples](/v0.1/examples/)
