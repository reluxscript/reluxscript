# babel-plugin-minimact - Complete Implementation

Babel plugin that transforms JSX/TSX React-style components into C# Minimact components with full hybrid rendering support.

## Features

✅ **All Hooks**
- `useState` → `[State]` attributes in C#
- `useClientState` → Client-side only state (no server round-trips)
- `useEffect` → Lifecycle methods
- `useRef` → Reference tracking
- `useMarkdown` → Server-side markdown parsing
- `useTemplate` → Layout inheritance

✅ **JSX Transformation**
- Elements → `VElement` construction
- Text → `VText` nodes
- Fragments → `Fragment` wrapper
- Conditional rendering (ternary `?:` and short-circuit `&&`)
- List rendering (`.map()` with keys)
- Event handlers → C# methods

✅ **Hybrid Rendering**
- Dependency tracking (which JSX nodes depend on which state)
- Zone classification (client/server/hybrid/static)
- Smart span splitting for mixed dependencies
- `data-minimact-*-scope` attributes

✅ **Advanced**
- Props support with TypeScript interfaces → C# classes
- Partial classes for EF Core codebehind
- Template base class inheritance
- Event handler extraction

## Installation

```bash
npm install babel-plugin-minimact
```

## Configuration

**.babelrc:**
```json
{
  "plugins": [
    ["babel-plugin-minimact", {
      "namespace": "MyApp.Components"
    }]
  ],
  "presets": [
    "@babel/preset-react",
    "@babel/preset-typescript"
  ]
}
```

## Usage

### Basic Counter

**Input (Counter.tsx):**
```tsx
import { useState } from '@minimact/core';

export function Counter() {
  const [count, setCount] = useState(0);

  return (
    <div>
      <h1>Count: {count}</h1>
      <button onClick={() => setCount(count + 1)}>
        Increment
      </button>
    </div>
  );
}
```

**Output (Counter.cs):**
```csharp
using Minimact.AspNetCore.Core;

namespace MyApp.Components;

[Component]
public partial class Counter : MinimactComponent
{
    [State]
    private int count = 0;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new VNode[]
        {
            new VElement("h1", $"Count: {count}"),
            new VElement("button", new Dictionary<string, string>
            {
                ["onclick"] = "Handle0"
            }, "Increment")
        });
    }

    private void Handle0()
    {
        count = count + 1;
        SetState(nameof(count), count);
    }
}
```

### Hybrid Rendering (Client + Server State)

**Input:**
```tsx
import { useState, useClientState } from '@minimact/core';

export function SearchBox() {
  const [results, setResults] = useState([]);        // Server state
  const [query, setQuery] = useClientState('');      // Client state

  return (
    <div>
      {/* Client zone - instant updates */}
      <input
        value={query}
        onInput={e => setQuery(e.target.value)}
      />

      {/* Hybrid zone - mixed dependencies */}
      <p>Found {results.length} results for "{query}"</p>

      {/* Server zone */}
      <ul>
        {results.map(r => <li key={r.id}>{r.title}</li>)}
      </ul>
    </div>
  );
}
```

**Output:**
```csharp
[Component]
public partial class SearchBox : MinimactComponent
{
    [State]
    private List<object> results = new List<object>();

    // query is client-side only - NOT in C# state

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new VNode[]
        {
            // Client zone
            new VElement("input", new Dictionary<string, string>
            {
                ["data-minimact-client-scope"] = "true",
                ["data-state"] = "query",
                ["oninput"] = "HandleClientStateChange:query"
            }),

            // Hybrid zone - smart span splitting
            new VElement("p", new VNode[]
            {
                new VText("Found "),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-server-scope"] = "true"
                }, results.Count.ToString()),
                new VText(" results for \""),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-client-scope"] = "true",
                    ["data-bind"] = "query"
                }),
                new VText("\"")
            }),

            // Server zone
            new VElement("ul", results.Select(r =>
                new VElement("li", new Dictionary<string, string>
                {
                    ["key"] = r.id.ToString()
                }, r.title)
            ).ToArray())
        });
    }
}
```

### Conditional Rendering

**Input:**
```tsx
export function UserProfile() {
  const [user, setUser] = useState(null);

  return (
    <div>
      {user ? (
        <h1>{user.name}</h1>
      ) : (
        <p>Loading...</p>
      )}
      {user && <img src={user.avatar} />}
    </div>
  );
}
```

**Output:**
```csharp
[Component]
public partial class UserProfile : MinimactComponent
{
    [State]
    private object user = null;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", new VNode[]
        {
            user != null
                ? new VElement("h1", user.name)
                : new VElement("p", "Loading..."),
            user != null
                ? new VElement("img", new Dictionary<string, string>
                {
                    ["src"] = user.avatar
                })
                : new VText("")
        });
    }
}
```

### List Rendering

**Input:**
```tsx
export function TodoList() {
  const [todos, setTodos] = useState([]);

  return (
    <ul>
      {todos.map(todo => (
        <li key={todo.id}>
          <input type="checkbox" checked={todo.done} />
          <span>{todo.text}</span>
        </li>
      ))}
    </ul>
  );
}
```

**Output:**
```csharp
[Component]
public partial class TodoList : MinimactComponent
{
    [State]
    private List<object> todos = new List<object>();

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("ul",
            todos.Select(todo => new VElement("li",
                new Dictionary<string, string> { ["key"] = todo.id.ToString() },
                new VNode[]
                {
                    new VElement("input", new Dictionary<string, string>
                    {
                        ["type"] = "checkbox",
                        ["checked"] = todo.done.ToString()
                    }),
                    new VElement("span", todo.text)
                }
            )).ToArray()
        );
    }
}
```

### Markdown Support

**Input:**
```tsx
import { useState, useMarkdown, useTemplate } from '@minimact/core';

export function BlogPost() {
  const [post, setPost] = useState(null);
  const [content, setContent] = useMarkdown('');

  useTemplate('BlogLayout');

  return (
    <article>
      <h1>{post.title}</h1>
      <div dangerouslySetInnerHTML={{ __html: content }} />
    </article>
  );
}
```

**Output:**
```csharp
[Component]
public partial class BlogPost : BlogLayoutBase
{
    [State]
    private object post = null;

    [State]
    private string content = "";

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("article", new VNode[]
        {
            new VElement("h1", post.title),
            new VElement("div", new DivRawHtml(content))
        });
    }
}
```

**Codebehind (BlogPost.codebehind.cs):**
```csharp
using Microsoft.EntityFrameworkCore;

public partial class BlogPost
{
    private readonly AppDbContext _db;

    public BlogPost(AppDbContext db)
    {
        _db = db;
    }

    public override async Task OnInitializedAsync()
    {
        var id = RouteData.GetInt("id");
        post = await _db.Posts.FindAsync(id);
        content = post.Content; // Markdig parses this
        TriggerRender();
    }
}
```

### Props Support

**Input:**
```tsx
interface CardProps {
  title: string;
  count: number;
  icon?: string;
}

export function Card({ title, count, icon }: CardProps) {
  return (
    <div className="card">
      {icon && <img src={icon} />}
      <h3>{title}</h3>
      <p>{count}</p>
    </div>
  );
}
```

**Output:**
```csharp
public class CardProps
{
    public string Title { get; set; }
    public int Count { get; set; }
    public string? Icon { get; set; }
}

[Component]
public partial class Card : MinimactComponent
{
    private CardProps Props { get; set; }

    public Card(CardProps props)
    {
        Props = props;
    }

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div",
            new Dictionary<string, string> { ["className"] = "card" },
            new VNode[]
            {
                !string.IsNullOrEmpty(Props.Icon)
                    ? new VElement("img", new Dictionary<string, string>
                    {
                        ["src"] = Props.Icon
                    })
                    : new VText(""),
                new VElement("h3", Props.Title),
                new VElement("p", Props.Count.ToString())
            }
        );
    }
}
```

## Dependency Tracking

The plugin automatically tracks which JSX nodes depend on which state variables:

```tsx
const [serverCount, setServerCount] = useState(0);      // Server
const [clientInput, setClientInput] = useClientState(''); // Client

// Pure client zone
<p>You typed: {clientInput}</p>
// → data-minimact-client-scope

// Pure server zone
<p>Count: {serverCount}</p>
// → data-minimact-server-scope

// Hybrid zone (automatically split into spans)
<p>Typed {clientInput.length} chars, count is {serverCount}</p>
// → Split into:
//    <span data-minimact-client-scope>{clientInput.length}</span>
//    <span data-minimact-server-scope>{serverCount}</span>
```

## API

### Options

```typescript
{
  namespace?: string;  // C# namespace (default: "Minimact.Components")
}
```

### Supported Hooks

| Hook | Purpose | C# Output |
|------|---------|-----------|
| `useState(x)` | Server state | `[State] private T name = x;` |
| `useClientState(x)` | Client state | Not in C# (client-only) |
| `useEffect(() => {})` | Lifecycle | `OnStateChanged()` method |
| `useRef(x)` | Reference | `[Ref] private object name = x;` |
| `useMarkdown('')` | Markdown parsing | `[State] private string content = "";` + Markdig |
| `useTemplate('Layout')` | Layout inheritance | `public partial class X : LayoutBase` |

## Testing

Run the test fixtures:

```bash
npm test
```

See `test/fixtures/` for comprehensive examples.

## License

MIT
