import { useState, useEffect, useRef } from '@minimact/core';

export function Counter() {
  const [count, setCount] = useState(0);
  const buttonRef = useRef(null);

  useEffect(() => {
    console.log('Count changed to:', count);
  }, [count]);

  return (
    <div className="counter">
      <h1>Counter Example</h1>
      <p>Current count: {count}</p>
      <button ref={buttonRef} onClick={() => setCount(count + 1)}>
        Increment
      </button>
      <button onClick={() => setCount(0)}>
        Reset
      </button>
    </div>
  );
}
