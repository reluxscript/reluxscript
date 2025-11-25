import { useState } from '@minimact/core';

/**
 * Test component for Phases 5 & 6 template extraction
 *
 * This should extract:
 * - Structural templates (Phase 5): Conditionals
 * - Expression templates (Phase 6): Computed values
 */

// =============================================================================
// PHASE 5: Structural Templates (Conditional Rendering)
// =============================================================================

export function UserProfile() {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [user, setUser] = useState<any>(null);

  return (
    <div className="user-profile">
      {/* Ternary conditional - structural template */}
      {isLoggedIn ? (
        <div className="dashboard">
          <h1>Welcome, {user?.name}!</h1>
          <button onClick={() => setIsLoggedIn(false)}>Logout</button>
        </div>
      ) : (
        <div className="login-form">
          <h1>Please log in</h1>
          <button onClick={() => setIsLoggedIn(true)}>Login</button>
        </div>
      )}
    </div>
  );
}

export function LoadingState() {
  const [isLoading, setIsLoading] = useState(true);
  const [data, setData] = useState<any>(null);

  return (
    <div>
      {/* Another ternary - loading state */}
      {isLoading ? (
        <div className="spinner">Loading...</div>
      ) : (
        <div className="content">
          <h2>{data.title}</h2>
          <p>{data.description}</p>
        </div>
      )}
    </div>
  );
}

export function ErrorBoundary() {
  const [error, setError] = useState<string | null>(null);
  const [content, setContent] = useState('Hello');

  return (
    <div>
      {/* Logical AND - error message */}
      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className="content">{content}</div>
    </div>
  );
}

// =============================================================================
// PHASE 6: Expression Templates (Computed Values)
// =============================================================================

export function PriceDisplay() {
  const [price, setPrice] = useState(99.95);
  const [quantity, setQuantity] = useState(2);
  const [discount, setDiscount] = useState(0.1);

  return (
    <div className="price-display">
      {/* toFixed() - number formatting */}
      <p>Price: ${price.toFixed(2)}</p>

      {/* Arithmetic expression */}
      <p>Quantity: {quantity}</p>
      <p>Subtotal: ${(price * quantity).toFixed(2)}</p>

      {/* Complex arithmetic */}
      <p>Discount: ${(price * quantity * discount).toFixed(2)}</p>
      <p>Total: ${(price * quantity * (1 - discount)).toFixed(2)}</p>
    </div>
  );
}

export function StringOperations() {
  const [name, setName] = useState('john doe');
  const [text, setText] = useState('  Hello World  ');

  return (
    <div>
      {/* String methods */}
      <p>Uppercase: {name.toUpperCase()}</p>
      <p>Lowercase: {name.toLowerCase()}</p>
      <p>Trimmed: "{text.trim()}"</p>
    </div>
  );
}

export function ArrayOperations() {
  const [items, setItems] = useState(['apple', 'banana', 'cherry']);

  return (
    <div>
      {/* Array length property */}
      <p>Count: {items.length} items</p>

      {/* Array join method */}
      <p>List: {items.join(', ')}</p>
    </div>
  );
}

export function MixedExpressions() {
  const [count, setCount] = useState(5);
  const [multiplier, setMultiplier] = useState(10);

  return (
    <div>
      {/* Binary expressions */}
      <p>Double: {count * 2}</p>
      <p>Plus one: {count + 1}</p>
      <p>Percentage: {count * multiplier}%</p>

      {/* Unary expressions */}
      <p>Negative: {-count}</p>
      <p>Positive: {+count}</p>
    </div>
  );
}

// =============================================================================
// COMBINED: Structural + Expression Templates
// =============================================================================

export function MetricsDashboard() {
  const [metrics, setMetrics] = useState<any>({
    hitRate: 0.856,
    avgLatency: 4.2,
    totalRequests: 12345
  });
  const [isLoading, setIsLoading] = useState(false);

  return (
    <div className="dashboard">
      {/* Structural template - loading state */}
      {isLoading ? (
        <div>Loading metrics...</div>
      ) : (
        <div className="metrics">
          {/* Expression templates - formatting */}
          <div className="metric">
            <label>Hit Rate:</label>
            <span>{(metrics.hitRate * 100).toFixed(1)}%</span>
          </div>

          <div className="metric">
            <label>Avg Latency:</label>
            <span>{metrics.avgLatency.toFixed(2)}ms</span>
          </div>

          <div className="metric">
            <label>Total Requests:</label>
            <span>{metrics.totalRequests.toLocaleString()}</span>
          </div>
        </div>
      )}
    </div>
  );
}

export function BlogPost() {
  const [post, setPost] = useState<any>(null);
  const [views, setViews] = useState(0);

  return (
    <div>
      {/* Structural template - null check */}
      {post ? (
        <article>
          {/* Expression template - string method */}
          <h1>{post.title.toUpperCase()}</h1>

          <p>{post.content}</p>

          {/* Expression template - arithmetic */}
          <footer>
            Views: {views} | Likes: {Math.floor(views * 0.1)}
          </footer>
        </article>
      ) : (
        <div>Loading post...</div>
      )}
    </div>
  );
}
