# Minimact Hybrid Rendering Architecture

Based on insights from `convo2.txt` - handling the critical boundary between server state and client state.

---

## The Challenge

When a component uses **both** server state (`useState`) and client state (`useClientState`):

```tsx
function Hybrid() {
  const [serverCount, setServerCount] = useState(0);      // Server-side
  const [clientInput, setClientInput] = useClientState(""); // Client-side

  return (
    <div>
      <input value={clientInput} onInput={e => setClientInput(e.target.value)} />
      <button onClick={() => setServerCount(serverCount + 1)}>
        Count: {serverCount}
      </button>
      <p>You typed: {clientInput}</p>
      <p>You typed {clientInput.length} chars, count is {serverCount}</p> {/* MIXED! */}
    </div>
  );
}
```

**Key Questions:**
1. How do we avoid full server round-trips for client-only changes?
2. How do we handle nodes that depend on BOTH types of state?
3. How do we partition DOM ownership between client and server?

---

## Solution: Hybrid Reconciliation Model

### 1. **State Types**

| Hook | Where it Lives | Updates Trigger |
|------|----------------|-----------------|
| `useState(x)` | Server (C#) | SignalR patch from server |
| `useClientState(x)` | Client (JS) | Local DOM update |
| `useSyncedState(x)` | Both (synced) | Debounced round-trip |

### 2. **DOM Partitioning**

The DOM is logically split into **zones**:

```
┌─────────────────────────────────────┐
│  <div>                              │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 🎛️ CLIENT ZONE              │   │
│  │ <input> (clientInput)        │   │
│  │ <p>You typed: {clientInput}</p>│   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 🔒 SERVER ZONE              │   │
│  │ <button> Count: {serverCount}│   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 🔀 MIXED ZONE (hybrid)      │   │
│  │ <p>                         │   │
│  │   <span client>{length}</span>│   │
│  │   <span server>{count}</span>│   │
│  │ </p>                        │   │
│  └─────────────────────────────┘   │
│                                     │
│  </div>                             │
└─────────────────────────────────────┘
```

---

## Dependency Tracking (Compile-Time)

The **Babel plugin** tracks which state each JSX expression depends on:

### Algorithm:

```javascript
function analyzeDependencies(jsxExpression) {
  const deps = new Set();

  traverse(jsxExpression, {
    Identifier(path) {
      const binding = path.scope.getBinding(path.node.name);

      if (binding && isStateVariable(binding)) {
        const stateType = getStateType(binding); // 'server' | 'client' | 'synced'
        deps.add({ name: path.node.name, type: stateType });
      }
    }
  });

  return deps;
}
```

### Classification:

```javascript
if (deps.size === 0) {
  // Static node - no state dependency
  zone = 'static';
} else if (deps.every(d => d.type === 'client')) {
  // Pure client zone
  zone = 'client';
  addAttribute('data-minimact-client-scope');
} else if (deps.every(d => d.type === 'server')) {
  // Pure server zone
  zone = 'server';
  addAttribute('data-minimact-server-scope');
} else {
  // Mixed zone - needs smart splitting
  zone = 'hybrid';
  performSmartSplit(node, deps);
}
```

---

## Rendering Strategies

### **Pure Client Zone** ✅

**Input:**
```tsx
<input value={clientInput} onInput={e => setClientInput(e.target.value)} />
<p>You typed: {clientInput}</p>
```

**Output (Babel):**
```csharp
// C# Render() method DOES NOT include this in VNode tree
// Client handles it entirely

// In generated HTML:
<input data-minimact-client-scope data-state="clientInput" value="" />
<p data-minimact-client-scope>You typed: <span data-bind="clientInput"></span></p>
```

**Client Runtime (minimact.js):**
```javascript
// Hydrates client zones
const clientStates = {
  clientInput: ''
};

function updateClientZone(stateName, value) {
  clientStates[stateName] = value;

  // Find all elements bound to this state
  const elements = document.querySelectorAll(`[data-bind="${stateName}"]`);
  elements.forEach(el => {
    el.textContent = value;
  });
}
```

---

### **Pure Server Zone** ✅

**Input:**
```tsx
<button onClick={() => setServerCount(serverCount + 1)}>
  Count: {serverCount}
</button>
```

**Output (Babel → C#):**
```csharp
new VElement("button", new Dictionary<string, string>
{
    ["onClick"] = "IncrementCount",
    ["data-minimact-server-scope"] = "true"
}, $"Count: {serverCount}")
```

**Behavior:**
1. Click triggers SignalR message to server
2. Server runs `SetState(nameof(serverCount), serverCount + 1)`
3. Server calls `Render()` → generates new VNode
4. Rust reconciliation engine diffs old vs new
5. Server sends patch via SignalR
6. Client applies patch **only to server zones**

---

### **Mixed Zone (Hybrid)** 🔀

**Input:**
```tsx
<p>You typed {clientInput.length} chars, count is {serverCount}</p>
```

**Problem:** Depends on BOTH `clientInput` (client) and `serverCount` (server)

#### **Option A: Client Template + Server Values**

**Output:**
```html
<p data-minimact-hybrid data-template="typed-${clientInputLength}-count-${serverCount}">
  You typed 0 chars, count is 0
</p>
```

**Behavior:**
- Client owns the template string
- Server sends value updates: `{ serverCount: 5 }`
- Client re-renders template locally

**Use when:** Small value interpolation in text

#### **Option B: Full Server Control (Fallback)**

**Output:**
```csharp
// Treat as server-controlled
new VElement("p", $"You typed {clientInput.Length} chars, count is {serverCount}")
```

**Behavior:**
- Any change to `clientInput` sends update to server
- Server re-renders full `<p>`
- Round-trip on every keystroke (slow!)

**Use when:** Rare edge cases, strict SSR mode

#### **Option C: Smart Splitting** ✨ (RECOMMENDED)

**Babel Transform:**
```javascript
// Original JSX:
<p>You typed {clientInput.length} chars, count is {serverCount}</p>

// After dependency analysis, split into:
<p>
  You typed
  <span data-minimact-client-scope data-bind="clientInput.length">0</span>
  chars, count is
  <span data-minimact-server-scope data-bind="serverCount">0</span>
</p>
```

**Generated C# (Server):**
```csharp
new VElement("p", new VNode[]
{
    new VText("You typed "),
    new VElement("span", new Dictionary<string, string>
    {
        ["data-minimact-client-scope"] = "true",
        ["data-bind"] = "clientInput.length"
    }, "0"), // Initial value from server
    new VText(" chars, count is "),
    new VElement("span", new Dictionary<string, string>
    {
        ["data-minimact-server-scope"] = "true",
        ["data-bind"] = "serverCount"
    }, $"{serverCount}")
})
```

**Client Runtime:**
```javascript
// Client updates only its span
document.querySelector('[data-bind="clientInput.length"]').textContent = clientInput.length;

// Server patches only its span via SignalR
```

**Benefits:**
- ✅ Precise updates (no unnecessary re-renders)
- ✅ Zero round-trips for client state
- ✅ Granular patching
- ✅ Optimal for Rust prediction engine

---

## Full Example: Hybrid Component

### Input (TSX)

```tsx
import { useState, useClientState } from '@minimact/core';

export function SearchBox() {
  const [results, setResults] = useState([]);        // Server
  const [query, setQuery] = useClientState('');      // Client

  const search = async () => {
    // Fetch from server
    const data = await fetch(`/api/search?q=${query}`);
    setResults(await data.json());
  };

  return (
    <div>
      {/* Pure client zone */}
      <input
        value={query}
        onInput={e => setQuery(e.target.value)}
        placeholder="Search..."
      />

      <button onClick={search}>Search</button>

      {/* Pure client zone */}
      <p>Query: {query}</p>

      {/* Hybrid zone */}
      <p>Found {results.length} results for "{query}"</p>

      {/* Pure server zone */}
      <ul>
        {results.map(r => <li key={r.id}>{r.title}</li>)}
      </ul>
    </div>
  );
}
```

### Output (C# - Partial Class)

```csharp
[Component]
public partial class SearchBox : MinimactComponent
{
    [State]
    private List<SearchResult> results = new();

    // Client state NOT in C# - handled by JS

    protected override VNode Render()
    {
        return new VElement("div", new VNode[]
        {
            // Client zone - not rendered by C#
            // Handled by minimact.js hydration

            // Server button
            new VElement("button", new Dictionary<string, string>
            {
                ["onClick"] = "Search"
            }, "Search"),

            // Client zone - not in server tree

            // Hybrid zone - smart split
            new VElement("p", new VNode[]
            {
                new VText("Found "),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-server-scope"] = "true"
                }, $"{results.Count}"),
                new VText(" results for \""),
                new VElement("span", new Dictionary<string, string>
                {
                    ["data-minimact-client-scope"] = "true",
                    ["data-bind"] = "query"
                }, ""),
                new VText("\"")
            }),

            // Server zone
            new VElement("ul", results.Select(r =>
                new VElement("li", new Dictionary<string, string>
                {
                    ["key"] = r.Id.ToString()
                }, r.Title)
            ).ToArray())
        });
    }

    private async Task Search()
    {
        // Get query from client (sent via SignalR)
        var query = Context.GetClientState("query");

        results = await _searchService.SearchAsync(query);
        TriggerRender();
    }
}
```

### Client Runtime (minimact.js)

```javascript
class MinimactClient {
  constructor() {
    this.clientStates = {};
    this.serverConnection = new signalR.HubConnectionBuilder()
      .withUrl('/minimact')
      .build();

    this.hydrateClientZones();
  }

  hydrateClientZones() {
    // Find all client-scoped elements
    const clientElements = document.querySelectorAll('[data-minimact-client-scope]');

    clientElements.forEach(el => {
      const stateName = el.dataset.state;
      if (stateName) {
        // Bind input events
        el.addEventListener('input', (e) => {
          this.updateClientState(stateName, e.target.value);
        });
      }
    });
  }

  updateClientState(stateName, value) {
    this.clientStates[stateName] = value;

    // Update all elements bound to this state
    const boundElements = document.querySelectorAll(`[data-bind="${stateName}"]`);
    boundElements.forEach(el => {
      el.textContent = value;
    });
  }

  applyServerPatch(patch) {
    // Only apply patches to server-scoped elements
    // Skip client zones entirely
    if (patch.path.startsWith('[data-minimact-server-scope]')) {
      this.applyPatch(patch);
    }
  }
}
```

---

## Performance Characteristics

### **Client State Change** (e.g., typing in input)
1. User types → 0ms
2. Client updates local state → <1ms
3. Client updates bound DOM elements → <1ms
**Total: ~1ms** ✅ (feels instant!)

### **Server State Change** (e.g., search results)
1. User clicks Search → 0ms
2. SignalR message to server → 20ms (network)
3. Server processes, diffs → 5ms
4. Patch sent back → 20ms (network)
5. Client applies patch → 2ms
**Total: ~47ms** ✅ (acceptable for server data)

### **Hybrid Update** (both states change)
1. Client state updates immediately → 1ms
2. Server state patches arrive → 47ms
3. **No conflict!** Each zone updates independently
**Perceived latency: 1ms** ✅ (instant feedback!)

---

## Summary

| Scenario | Strategy | Latency |
|----------|----------|---------|
| Pure client state (`useClientState`) | Client-only updates | ~1ms |
| Pure server state (`useState`) | SignalR patch | ~47ms |
| Mixed dependencies | Smart span splitting | ~1ms (client) + ~47ms (server) |

**Key Insight:** By partitioning the DOM and using compile-time dependency tracking, Minimact achieves:
- ✅ Instant client interactivity
- ✅ Server-controlled business logic
- ✅ Zero unnecessary re-renders
- ✅ Optimal use of Rust prediction engine

This is the **surgical precision** that makes Minimact unique! 🧠⚡
