export function ManualKeys() {
  return (
    <div key="1">
      <h1 key="1.1">First</h1>
      <p key="1.5">Custom key (manually added)</p>
      <p key="1.2">Last</p>
      <span>No key (should auto-generate)</span>
    </div>
  );
}
