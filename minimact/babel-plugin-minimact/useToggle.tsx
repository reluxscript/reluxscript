import { useState } from '@minimact/core';

/**
 * Custom Hook: useToggle
 * Simple boolean toggle hook with UI
 *
 * @param namespace - Unique identifier for this hook instance (REQUIRED)
 * @param initial - Initial boolean value
 * @returns [on, toggle, ui]
 */
function useToggle(namespace: string, initial: boolean = false) {
  const [on, setOn] = useState(initial);

  const toggle = () => setOn(!on);

  const ui = (
    <button onClick={toggle} className="toggle-button">
      {on ? 'ON' : 'OFF'}
    </button>
  );

  return [on, toggle, ui];
}

export default useToggle;
