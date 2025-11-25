import { useState } from '@minimact/core';
import useToggle from './useToggle';

/**
 * Component that uses an imported custom hook
 * Tests cross-file hook imports and transpilation
 */
export default function TestImportedHook() {
  const [count, setCount] = useState(0);

  // Use imported hook with two independent instances
  const [isOn1, toggle1, toggleUI1] = useToggle('toggle1', false);
  const [isOn2, toggle2, toggleUI2] = useToggle('toggle2', true);

  return (
    <div className="app">
      <h1>Imported Hook Test</h1>

      <div className="counter-section">
        <h2>Regular State (for comparison)</h2>
        <button onClick={() => setCount(count + 1)}>
          Count: {count}
        </button>
      </div>

      <div className="toggle-section">
        <h2>Toggle 1 (starts OFF)</h2>
        <p>Status: {isOn1 ? 'Active' : 'Inactive'}</p>
        <button onClick={toggle1}>External Toggle 1</button>
        {toggleUI1}
      </div>

      <div className="toggle-section">
        <h2>Toggle 2 (starts ON)</h2>
        <p>Status: {isOn2 ? 'Active' : 'Inactive'}</p>
        <button onClick={toggle2}>External Toggle 2</button>
        {toggleUI2}
      </div>

      <div className="combined-state">
        <p>Both toggles on: {isOn1 && isOn2 ? 'YES' : 'NO'}</p>
        <p>At least one on: {isOn1 || isOn2 ? 'YES' : 'NO'}</p>
      </div>
    </div>
  );
}
