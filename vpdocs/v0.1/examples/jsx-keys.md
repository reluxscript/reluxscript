# JSX Key Checker Example

Warn about missing `key` props in JSX elements inside arrays.

## Code

```reluxscript
plugin JSXKeyChecker {
    struct State {
        in_array: bool,
    }

    fn visit_array_expression(node: &mut ArrayExpression, ctx: &Context) {
        let was_in_array = self.state.in_array;
        self.state.in_array = true;

        node.visit_children(self);

        self.state.in_array = was_in_array;
    }

    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        if self.state.in_array {
            let has_key = node.opening_element.attributes.iter()
                .any(|attr| matches!(attr, JSXAttribute { name: "key" }));

            if !has_key {
                eprintln!("Warning: JSX element in array should have a key prop");
            }
        }

        node.visit_children(self);
    }
}
```

## Input

```javascript
const items = [
    <Item />,
    <Item />,
];
```

## Output

```
Warning: JSX element in array should have a key prop
Warning: JSX element in array should have a key prop
```

## How It Works

1. Track whether we're inside an array using state
2. When entering an array, set `in_array` to true
3. When visiting JSX elements, check if we're in an array
4. If in array and no `key` attribute, emit warning
5. Restore previous state when leaving array

[Back to Examples](/v0.1/examples/)
