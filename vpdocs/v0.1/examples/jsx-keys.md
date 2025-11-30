# JSX Key Checker Example

Warn about missing `key` props in JSX elements inside arrays.

## Code

```reluxscript
plugin JSXKeyChecker {
    struct State {
        jsx_without_keys: i32,
    }

    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Check if this JSX element has a key attribute
        let mut has_key = false;

        for attr in &node.opening_element.attributes {
            if let JSXAttribute::JSXAttribute(jsx_attr) = attr {
                if let JSXAttributeName::Identifier(ref ident) = jsx_attr.name {
                    if ident.name == "key" {
                        has_key = true;
                        break;
                    }
                }
            }
        }

        if !has_key {
            // Mark elements without keys using custom property
            node.__missingKey = true;
            self.state.jsx_without_keys = self.state.jsx_without_keys + 1;
        }
    }

    fn exit(program: &mut Program, ctx: &Context) {
        if self.state.jsx_without_keys > 0 {
            println!("Warning: {} JSX elements without key prop", self.state.jsx_without_keys);
        }
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
Warning: 2 JSX elements without key prop
```

## How It Works

1. For each JSX element, iterate through its attributes
2. Use nested `if let` to check if any attribute is named "key"
3. If no key attribute found, mark the element with custom property `__missingKey`
4. Track count of elements without keys in plugin state
5. In `exit()`, report total count of JSX elements missing keys

[Back to Examples](/v0.1/examples/)
