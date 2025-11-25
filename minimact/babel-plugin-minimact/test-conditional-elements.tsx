import { useState } from '@minimact/core';

export function ConditionalTest() {
  const [myState1, setMyState1] = useState(false);
  const [myState2, setMyState2] = useState(false);
  const [myState3, setMyState3] = useState('Hello World');

  return (
    <div>
      <h1>Conditional Element Test</h1>

      {/* Simple conditional */}
      {myState1 && <div>myState1 is true</div>}

      {/* Complex conditional */}
      {myState1 && !myState2 && (
        <div className="nested-content">
          <span>SomeNestedDOMElementsHere</span>
          <span>{myState3}</span>
        </div>
      )}

      {/* Ternary with elements */}
      {myState1 ? (
        <div className="active">Active State</div>
      ) : (
        <div className="inactive">Inactive State</div>
      )}

      <button onClick={() => setMyState1(!myState1)}>
        Toggle myState1
      </button>
    </div>
  );
}
