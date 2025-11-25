/**
 * Test Custom Hook: useCounter
 *
 * This tests the custom hooks transpilation system
 */

import { useState } from '@minimact/core';

// Custom hook definition
function useCounter(namespace: string, start: number = 0) {
  const [count, setCount] = useState(start);

  const increment = () => {
    setCount(count + 1);
  };

  const decrement = () => {
    setCount(count - 1);
  };

  const reset = () => {
    setCount(start);
  };

  const ui = (
    <div className="counter-widget">
      <button onClick={decrement}>-</button>
      <span className="count-display">{count}</span>
      <button onClick={increment}>+</button>
      <button onClick={reset}>Reset</button>
    </div>
  );

  return [count, increment, decrement, reset, ui];
}

// Component using the hook
export default function TestCustomHook() {
  const [count, increment, decrement, reset, counterUI] = useCounter('myCounter', 0);

  return (
    <div className="test-container">
      <h1>Custom Hook Test</h1>
      <p>Count: {count}</p>

      <div className="controls">
        <button onClick={increment}>External +1</button>
        <button onClick={decrement}>External -1</button>
        <button onClick={reset}>External Reset</button>
      </div>

      <hr />

      <div className="hook-ui">
        <h2>Hook UI:</h2>
        {counterUI}
      </div>
    </div>
  );
}
