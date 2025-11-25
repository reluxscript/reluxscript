/**
 * Comprehensive test fixture demonstrating all Minimact features
 */
import { useState, useClientState, useEffect, useRef, useMarkdown, useTemplate } from '@minimact/core';

// Example 1: Basic Counter
export function Counter() {
  const [count, setCount] = useState(0);

  return (
    <div>
      <h1>Count: {count}</h1>
      <button onClick={() => setCount(count + 1)}>Increment</button>
      <button onClick={() => setCount(count - 1)}>Decrement</button>
    </div>
  );
}

// Example 2: Conditional Rendering
export function UserProfile() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  return (
    <div>
      {loading ? (
        <p>Loading...</p>
      ) : (
        <div>
          <h1>{user.name}</h1>
          <p>{user.email}</p>
        </div>
      )}
      {user && <img src={user.avatar} />}
    </div>
  );
}

// Example 3: List Rendering
export function TodoList() {
  const [todos, setTodos] = useState([]);

  return (
    <div>
      <h1>My Todos</h1>
      <ul>
        {todos.map(todo => (
          <li key={todo.id}>
            <input type="checkbox" checked={todo.completed} />
            <span>{todo.text}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

// Example 4: Client State (Hybrid Rendering)
export function SearchBox() {
  const [results, setResults] = useState([]);
  const [query, setQuery] = useClientState('');

  const search = () => {
    // Server will handle the actual search
  };

  return (
    <div>
      {/* Client zone - instant updates */}
      <input
        value={query}
        onInput={e => setQuery(e.target.value)}
        placeholder="Search..."
      />

      <button onClick={search}>Search</button>

      {/* Hybrid zone - both client and server state */}
      <p>Found {results.length} results for "{query}"</p>

      {/* Server zone - search results */}
      <ul>
        {results.map(r => (
          <li key={r.id}>{r.title}</li>
        ))}
      </ul>
    </div>
  );
}

// Example 5: Fragments
export function MultiColumn() {
  return (
    <>
      <div className="column">Column 1</div>
      <div className="column">Column 2</div>
      <div className="column">Column 3</div>
    </>
  );
}

// Example 6: Markdown (for blog)
export function BlogPost() {
  const [post, setPost] = useState(null);
  const [content, setContent] = useMarkdown('');

  useTemplate('BlogLayout');

  return (
    <article>
      {post && (
        <>
          <h1>{post.title}</h1>
          <div className="markdown" dangerouslySetInnerHTML={{ __html: content }} />
        </>
      )}
    </article>
  );
}

// Example 7: Complex nested structure
export function Dashboard() {
  const [stats, setStats] = useState({ views: 0, clicks: 0, users: 0 });
  const [selectedPeriod, setSelectedPeriod] = useState('week');

  return (
    <div className="dashboard">
      <header>
        <h1>Analytics Dashboard</h1>
        <select value={selectedPeriod} onChange={e => setSelectedPeriod(e.target.value)}>
          <option value="day">Today</option>
          <option value="week">This Week</option>
          <option value="month">This Month</option>
        </select>
      </header>

      <div className="stats-grid">
        <div className="stat-card">
          <h3>Views</h3>
          <p className="stat-value">{stats.views}</p>
        </div>
        <div className="stat-card">
          <h3>Clicks</h3>
          <p className="stat-value">{stats.clicks}</p>
        </div>
        <div className="stat-card">
          <h3>Users</h3>
          <p className="stat-value">{stats.users}</p>
        </div>
      </div>
    </div>
  );
}

// Example 8: With props (component composition)
interface CardProps {
  title: string;
  count: number;
  icon?: string;
}

export function Card({ title, count, icon }: CardProps) {
  return (
    <div className="card">
      {icon && <img src={icon} alt={title} />}
      <h3>{title}</h3>
      <p className="count">{count}</p>
    </div>
  );
}
