# Minimact Babel Plugin - Feature Overview

## What's Been Built

A Babel plugin that transforms JSX/TSX React components into C# Minimact components for ASP.NET Core server-side rendering.

---

## Core Features ✅

### 1. **useState → [State] Attribute**

**Input:**
```tsx
const [count, setCount] = useState(0);
const [name, setName] = useState('John');
```

**Output:**
```csharp
[State]
private int count = 0;

[State]
private string name = "John";
```

### 2. **useEffect → Lifecycle Methods**

**Input:**
```tsx
useEffect(() => {
  console.log('Count changed');
}, [count]);
```

**Output:**
```csharp
[OnStateChanged("count")]
private void Effect_0()
{
    Console.WriteLine("Count changed");
}
```

### 3. **useRef → [Ref] Attribute**

**Input:**
```tsx
const buttonRef = useRef(null);
```

**Output:**
```csharp
[Ref]
private ElementRef buttonRef;
```

### 4. **JSX → VNode Construction**

**Input:**
```tsx
<div className="container">
  <h1>Title</h1>
  <p>Count: {count}</p>
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
    new VElement("p", $"Count: {count}")
})
```

### 5. **Event Handlers → C# Methods**

**Input:**
```tsx
<button onClick={() => setCount(count + 1)}>
  Increment
</button>
```

**Output:**
```csharp
new VElement("button", new Dictionary<string, string>
{
    ["onClick"] = "HandleClick_0"
}, "Increment")

// Method:
private void HandleClick_0()
{
    SetState(nameof(count), count + 1);
}
```

---

## Advanced Features ✅

### 6. **Partial Classes for Codebehind** (NEW!)

**Purpose**: Enable EF Core integration, dependency injection, and server-side logic.

**Generated (Babel):**
```csharp
[Component]
public partial class UserProfile : MinimactComponent
{
    [State]
    private User user;

    protected override VNode Render()
    {
        return user != null
            ? new Div($"Hello, {user.Name}")
            : new Div("Loading...");
    }
}
```

**Codebehind (User creates):**
```csharp
public partial class UserProfile
{
    private readonly AppDbContext _db;

    public UserProfile(AppDbContext db)
    {
        _db = db;
    }

    public override async Task OnInitializedAsync()
    {
        user = await _db.Users.FirstOrDefaultAsync();
        TriggerRender();
    }
}
```

**Benefits:**
- ✅ Full EF Core support
- ✅ Dependency injection
- ✅ Async data loading
- ✅ Service layer integration
- ✅ Keep business logic separate from generated code

---

### 7. **useMarkdown Hook** (NEW!)

**Purpose**: Server-side markdown parsing and rendering.

**Input:**
```tsx
const [content, setContent] = useMarkdown(`
# Hello World

This is **markdown**!
`);

return <div markdown>{content}</div>;
```

**Output:**
```csharp
[Markdown]
[State]
private string content = @"
# Hello World

This is **markdown**!
";

protected override VNode Render()
{
    return new DivRawHtml(content); // Parsed to HTML server-side
}
```

**Server Runtime** (using Markdig):
```csharp
private string _markdownRaw;

[Markdown]
public string Content
{
    get => Markdig.Markdown.ToHtml(_markdownRaw);
    set => _markdownRaw = value;
}
```

**Use Cases:**
- Blog posts
- Documentation sites
- CMS content
- Comments
- Rich text editors

---

### 8. **useTemplate Hook** (NEW!)

**Purpose**: Layout inheritance and composition.

**Input:**
```tsx
export function Dashboard() {
  useTemplate("DefaultLayout", { title: "Dashboard" });

  return (
    <div>
      <h1>Welcome to your dashboard</h1>
    </div>
  );
}
```

**Output:**
```csharp
[Component]
public partial class Dashboard : DefaultLayout
{
    protected override VNode RenderContent()
    {
        return new Div(
            new H1("Welcome to your dashboard")
        );
    }
}
```

**DefaultLayout.cs:**
```csharp
public abstract class DefaultLayout : MinimactComponent
{
    protected abstract VNode RenderContent();

    protected override VNode Render()
    {
        return new VElement("div", new VNode[]
        {
            new Header(), // Shared header
            RenderContent(), // Child content
            new Footer() // Shared footer
        });
    }
}
```

**Built-in Templates:**
- `DefaultLayout` - Header, content, footer
- `SidebarLayout` - Sidebar nav + main area
- `AuthLayout` - Login/register pages
- `AdminLayout` - Admin dashboard

**Benefits:**
- ✅ DRY layouts
- ✅ Consistent UI structure
- ✅ Easy to override
- ✅ Type-safe inheritance

---

## Complete Example

### Input (TSX)
```tsx
import { useState, useEffect, useMarkdown, useTemplate } from '@minimact/core';

export function BlogPost() {
  const [post, setPost] = useState(null);

  const [markdown, setMarkdown] = useMarkdown(`
# Article Title

Content here...
  `);

  useTemplate("DefaultLayout");

  useEffect(() => {
    console.log('Post loaded');
  }, [post]);

  return (
    <article>
      {post ? (
        <>
          <h1>{post.title}</h1>
          <div markdown>{markdown}</div>
          <p>Views: {post.views}</p>
        </>
      ) : (
        <div>Loading...</div>
      )}
    </article>
  );
}
```

### Output (C#)
```csharp
using Minimact;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Generated.Components
{
    [Component]
    public partial class BlogPost : DefaultLayout
    {
        [State]
        private Post post = null;

        [Markdown]
        [State]
        private string markdown = @"
# Article Title

Content here...
  ";

        [OnStateChanged("post")]
        private void Effect_0()
        {
            Console.WriteLine("Post loaded");
        }

        protected override VNode RenderContent()
        {
            return new VElement("article", post != null
                ? new VNode[]
                  {
                      new VElement("h1", $"{post.Title}"),
                      new DivRawHtml(markdown),
                      new VElement("p", $"Views: {post.Views}")
                  }
                : new VNode[]
                  {
                      new VElement("div", "Loading...")
                  }
            );
        }
    }
}
```

### Codebehind (User creates)
```csharp
namespace Generated.Components
{
    public partial class BlogPost
    {
        private readonly AppDbContext _db;

        public BlogPost(AppDbContext db)
        {
            _db = db;
        }

        public override async Task OnInitializedAsync()
        {
            post = await _db.Posts.FindAsync(RouteData.GetInt("id"));
            TriggerRender();
        }
    }
}
```

---

## Type Mapping

| JavaScript         | C#                          |
|--------------------|-----------------------------|
| `number`           | `int`                       |
| `string`           | `string`                    |
| `boolean`          | `bool`                      |
| `array`            | `List<object>`              |
| `object`           | `Dictionary<string, object>` |
| `function`         | `Action`                    |
| `null`/`undefined` | `null`                      |

---

## Hook Reference

| Hook           | C# Attribute/Method          | Purpose                          |
|----------------|------------------------------|----------------------------------|
| `useState`     | `[State]`                    | Component state                  |
| `useEffect`    | `[OnStateChanged(...)]`      | Side effects on state change     |
| `useRef`       | `[Ref]`                      | DOM element references           |
| `useMarkdown`  | `[Markdown][State]`          | Server-parsed markdown content   |
| `useTemplate`  | Base class inheritance       | Layout composition               |

---

## Architecture Benefits

### Developer Experience
- ✅ Write familiar React/TSX syntax
- ✅ Full TypeScript support
- ✅ Hot reload during development
- ✅ No webpack configuration needed

### Performance
- ✅ Zero JavaScript bundle (5KB client for SignalR)
- ✅ Server-side rendering
- ✅ Predictive reconciliation (Rust engine)
- ✅ Instant perceived updates

### Security
- ✅ Business logic stays on server
- ✅ Database queries never exposed
- ✅ Safe markdown rendering
- ✅ No client-side secrets

### Integration
- ✅ ASP.NET Core ecosystem
- ✅ Entity Framework Core
- ✅ Dependency injection
- ✅ .NET libraries and tools

---

## Workflow

```
Developer writes TSX
       ↓
Babel plugin transforms to C#
       ↓
Generated partial class
       ↓
Developer adds codebehind (optional)
       ↓
ASP.NET Core compiles
       ↓
Runtime: SignalR + Rust reconciliation
       ↓
Client: Instant UI updates
```

---

## Next Steps

### Phase 1 (Current)
- [x] useState → [State]
- [x] useEffect → lifecycle
- [x] useRef → [Ref]
- [x] JSX → VNode
- [x] Event handlers
- [x] Partial classes
- [x] useMarkdown
- [x] useTemplate

### Phase 2 (Next)
- [ ] Props support
- [ ] Conditional rendering (ternary, &&)
- [ ] List rendering (.map)
- [ ] Fragment support
- [ ] Custom hooks
- [ ] Context API

### Phase 3 (Future)
- [ ] TypeScript interface → C# class
- [ ] Advanced template features
- [ ] Component composition
- [ ] Error boundaries
- [ ] Suspense/async boundaries

---

## Files

```
babel-plugin-minimact/
├── index.cjs               # Original implementation
├── index-enhanced.cjs      # With useMarkdown & useTemplate
├── package.json
├── README.md
├── FEATURES.md             # This file
├── examples/
│   ├── Counter.input.jsx
│   ├── BlogPost.input.tsx
│   └── BlogPost.expected.cs
└── test/
    └── (test files)
```

---

## Credits

Based on insights from `convo.txt` discussing:
- EF Core integration via codebehind
- useMarkdown for content-heavy apps
- useTemplate for layout inheritance
- Partial classes for separation of concerns

Integrates with:
- **Rust reconciliation engine** (already built in `/src`)
- **ASP.NET Core runtime** (to be built)
- **SignalR** for real-time patches
- **Markdig** for markdown parsing
