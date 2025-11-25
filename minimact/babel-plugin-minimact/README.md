# @minimact/babel-plugin

Babel plugin that transforms JSX/TSX React components into C# Minimact components for server-side rendering with ASP.NET Core.

## Purpose

Minimact allows developers to write familiar React syntax (JSX/TSX with hooks) that gets compiled to C# classes running on ASP.NET Core. This plugin handles the transformation from JavaScript to C#.

> **Note**: This package is designed to work seamlessly with **Minimact Swig**, the official Electron-based IDE for Minimact development.

## Features

✅ **useState** → C# `[UseState]` attribute with private field
✅ **useEffect** → C# `[UseEffect]` attribute with method
✅ **useRef** → C# `[UseRef]` attribute with ElementRef field
✅ **JSX elements** → C# VNode construction (VElement, VText)
✅ **Event handlers** → C# methods with SignalR binding
✅ **Type inference** → JavaScript types mapped to C# types

## Input Example

```tsx
// Counter.tsx
import { useState, useEffect, useRef } from '@minimact/core';

export function Counter() {
  const [count, setCount] = useState(0);
  const buttonRef = useRef(null);

  useEffect(() => {
    console.log(`Count changed to: ${count}`);
  }, [count]);

  const increment = () => {
    setCount(count + 1);
  };

  return (
    <div className="counter">
      <h1>Counter</h1>
      <p>Count: {count}</p>
      <button ref={buttonRef} onClick={increment}>
        Increment
      </button>
    </div>
  );
}
```

## Output Example

```csharp
// Generated/Components/Counter.cs
using Minimact;
using System;
using System.Collections.Generic;
using System.Linq;

namespace Generated.Components
{
    [MinimactComponent]
    public class Counter : MinimactComponent
    {
        [UseState(0)]
        private int count;

        [UseRef(null)]
        private ElementRef buttonRef;

        [UseEffect("count")]
        private void Effect_0()
        {
            Console.WriteLine($"Count changed to: {count}");
        }

        protected override VNode Render()
        {
            return new VElement("div", new Dictionary<string, string>
            {
                ["className"] = "counter"
            }, new VNode[]
            {
                new VElement("h1", "Counter"),
                new VElement("p", $"Count: {count}"),
                new VElement("button", new Dictionary<string, string>
                {
                    ["ref"] = "buttonRef",
                    ["onClick"] = "Increment"
                }, "Increment")
            });
        }

        private void Increment()
        {
            SetState(nameof(count), count + 1);
        }
    }
}
```

## Usage

### Install

```bash
npm install --save-dev @minimact/babel-plugin @babel/core
```

### Configure Babel

```javascript
// babel.config.js
module.exports = {
  presets: [
    '@babel/preset-typescript',
    '@babel/preset-react'
  ],
  plugins: [
    '@minimact/babel-plugin'
  ]
};
```

### Usage with Minimact Swig

Minimact Swig automatically uses this plugin for transpilation. No manual configuration needed!

When you:
1. Create a new project in Swig
2. Edit TSX files in the Monaco editor
3. Save the file

Swig automatically:
- Invokes this Babel plugin
- Transpiles TSX → C#
- Writes the generated .cs files
- Triggers a rebuild (if auto-build is enabled)

### Build

```bash
# Transform TSX to C#
npx babel src/components --out-dir Generated/Components --extensions .tsx,.jsx
```

### Integration with Minimact Framework

```bash
# Watch mode during development
npx babel src/components --out-dir Generated/Components --extensions .tsx,.jsx --watch
```

The generated C# files are then compiled with your ASP.NET Core project.

## Type Mapping

| JavaScript Type | C# Type |
|----------------|---------|
| `number` | `int` |
| `string` | `string` |
| `boolean` | `bool` |
| `array` | `List<object>` |
| `object` | `Dictionary<string, object>` |
| `function` | `Action` |

## Hook Transformations

### useState

**Input:**
```tsx
const [count, setCount] = useState(0);
const [name, setName] = useState('John');
const [items, setItems] = useState([]);
```

**Output:**
```csharp
[UseState(0)]
private int count;

[UseState("John")]
private string name;

[UseState(new List<object>())]
private List<object> items;
```

### useEffect

**Input:**
```tsx
useEffect(() => {
  console.log('Mounted');
}, []);

useEffect(() => {
  console.log('Count changed');
}, [count]);

useEffect(() => {
  document.title = `Count: ${count}`;
}, [count, name]);
```

**Output:**
```csharp
[UseEffect()]
private void Effect_0()
{
    Console.WriteLine("Mounted");
}

[UseEffect("count")]
private void Effect_1()
{
    Console.WriteLine("Count changed");
}

[UseEffect("count", "name")]
private void Effect_2()
{
    // Note: document.title not available server-side
}
```

### useRef

**Input:**
```tsx
const inputRef = useRef(null);
const buttonRef = useRef(null);
```

**Output:**
```csharp
[UseRef(null)]
private ElementRef inputRef;

[UseRef(null)]
private ElementRef buttonRef;
```

## JSX Transformations

### Elements

**Input:**
```tsx
<div className="container">
  <h1>Title</h1>
  <p>Paragraph</p>
</div>
```

**Output:**
```csharp
new VElement("div", new Dictionary<string, string>
{
    ["className"] = "container"
}, new VNode[]
{
    new VElement("h1", "Title"),
    new VElement("p", "Paragraph")
})
```

### Dynamic Content

**Input:**
```tsx
<p>Count: {count}</p>
<p>{count > 5 ? 'High' : 'Low'}</p>
```

**Output:**
```csharp
new VElement("p", $"Count: {count}")
new VElement("p", $"{(count > 5 ? "High" : "Low")}")
```

### Event Handlers

**Input:**
```tsx
<button onClick={() => setCount(count + 1)}>
  Increment
</button>

<button onClick={handleReset}>
  Reset
</button>
```

**Output:**
```csharp
// In Render():
new VElement("button", new Dictionary<string, string>
{
    ["onClick"] = "HandleClick_0"
}, "Increment")

new VElement("button", new Dictionary<string, string>
{
    ["onClick"] = "HandleReset"
}, "Reset")

// Generated methods:
private void HandleClick_0()
{
    SetState(nameof(count), count + 1);
}

private void HandleReset()
{
    // User-defined method
}
```

## Complex Example

**Input:**
```tsx
import { useState, useEffect } from '@minimact/core';

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

export function TodoList() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [input, setInput] = useState('');

  const addTodo = () => {
    if (!input.trim()) return;

    setTodos([...todos, {
      id: Date.now(),
      text: input,
      done: false
    }]);
    setInput('');
  };

  const toggleTodo = (id: number) => {
    setTodos(todos.map(todo =>
      todo.id === id ? { ...todo, done: !todo.done } : todo
    ));
  };

  return (
    <div className="todo-list">
      <input
        type="text"
        value={input}
        onInput={(e) => setInput(e.target.value)}
      />
      <button onClick={addTodo}>Add</button>

      <ul>
        {todos.map(todo => (
          <li key={todo.id}>
            <input
              type="checkbox"
              checked={todo.done}
              onChange={() => toggleTodo(todo.id)}
            />
            <span>{todo.text}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

**Output:**
```csharp
using Minimact;
using System;
using System.Collections.Generic;
using System.Linq;

namespace Generated.Components
{
    [MinimactComponent]
    public class TodoList : MinimactComponent
    {
        [UseState(new List<object>())]
        private List<object> todos;

        [UseState("")]
        private string input;

        protected override VNode Render()
        {
            return new VElement("div", new Dictionary<string, string>
            {
                ["className"] = "todo-list"
            }, new VNode[]
            {
                new VElement("input", new Dictionary<string, string>
                {
                    ["type"] = "text",
                    ["value"] = $"{input}",
                    ["onInput"] = "HandleInput_0"
                }),
                new VElement("button", new Dictionary<string, string>
                {
                    ["onClick"] = "AddTodo"
                }, "Add"),
                new VElement("ul", todos.Select(todo =>
                    new VElement("li", new Dictionary<string, string>
                    {
                        ["key"] = $"{todo.id}"
                    }, new VNode[]
                    {
                        new VElement("input", new Dictionary<string, string>
                        {
                            ["type"] = "checkbox",
                            ["checked"] = $"{todo.done}",
                            ["onChange"] = $"ToggleTodo_{todo.id}"
                        }),
                        new VElement("span", $"{todo.text}")
                    })
                ).ToArray())
            });
        }

        private void AddTodo()
        {
            if (string.IsNullOrWhiteSpace(input)) return;

            SetState(nameof(todos), todos.Concat(new[] {
                new Todo {
                    Id = DateTimeOffset.Now.ToUnixTimeMilliseconds(),
                    Text = input,
                    Done = false
                }
            }).ToList());

            SetState(nameof(input), "");
        }

        private void ToggleTodo(long id)
        {
            SetState(nameof(todos), todos.Select(todo =>
                todo.Id == id ? new Todo { ...todo, Done = !todo.Done } : todo
            ).ToList());
        }

        private void HandleInput_0(InputEvent e)
        {
            SetState(nameof(input), e.Target.Value);
        }
    }
}
```

## Limitations

### Current Version (0.1.0)

⚠️ **Not yet implemented:**
- TypeScript interface definitions → C# classes
- Complex array operations (map, filter, reduce)
- Spread operators in objects
- Conditional rendering (ternary, &&)
- Fragment support (<>...</>)
- Custom hooks
- Context API
- Props (all components are currently prop-less)

### Planned Features

- Full TypeScript type preservation
- Props transformation
- Conditional rendering
- List rendering with .map()
- Fragment support
- Custom hook extraction
- Better error messages
- Source maps for debugging

## Development

### Project Structure

```
babel-plugin-minimact/
├── index.cjs          # Main plugin code
├── package.json       # Dependencies
├── README.md          # This file
└── test/
    ├── fixtures/      # Test input files
    └── expected/      # Expected output
```

### Testing

```bash
npm test
```

### Contributing

1. Add test cases in `test/fixtures/`
2. Run tests to ensure they fail
3. Implement feature
4. Run tests to ensure they pass
5. Submit PR

## License

MIT
