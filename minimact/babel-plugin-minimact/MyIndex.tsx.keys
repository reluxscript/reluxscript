import { useState, useEffect, useRef } from '@minimact/core';

/**
 * Hook Examples Index
 *
 * This page lists all hook examples generated for your project.
 * Click on an example to see it in action!
 */
export function Index() {
  const [activeExample, setActiveExample] = useState<string | null>(null);
  const [viewCount, setViewCount] = useState(0);
  const modalRef = useRef<HTMLDivElement>(null);
  const timerRef = useRef<number>(0);

  // Effect 1: Log when active example changes
  useEffect(() => {
    if (activeExample) {
      console.log(`Opened example: ${activeExample}`);
    }
  }, [activeExample]);

  // Effect 2: Track view count on mount
  useEffect(() => {
    setViewCount(viewCount + 1);
    console.log('Index page mounted');
  }, []);

  // Effect 3: Focus modal when it opens
  useEffect(() => {
    if (activeExample && modalRef.current) {
      modalRef.current.focus();
    }
  }, [activeExample]);

  // Effect 4: Set up a timer that runs on every render
  useEffect(() => {
    timerRef.current = Date.now();
  });

  return (
    <div
      key='1'
      style={{ padding: '20px', fontFamily: 'system-ui, sans-serif', maxWidth: '1200px', margin: '0 auto' }}>
      <h1 key='1.1' style={{ marginBottom: '10px' }}>Minimact Hook Examples</h1>
      <p key='1.2' style={{ color: '#666', marginBottom: '10px' }}>
        This project includes examples for 3 hooks.
        Select an example below to see the code in action.
      </p>
      <p key='1.2.1' style={{ color: '#999', fontSize: '14px', marginBottom: '30px' }}>
        Page views: {viewCount} | Timer ref: {timerRef.current}
      </p>
      {/* Hook Categories */}
      <div key='1.3' style={{ display: 'grid', gap: '30px' }}>
        {/* Core Hooks */}
        <div key='1.3.1'>
          <h2
            key='1.3.1.1'
            style={{ fontSize: '20px', marginBottom: '16px', color: '#333' }}>Core Hooks</h2>
          <div
            key='1.3.1.2'
            style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '12px' }}>
          <button
            key='1.3.1.2.1'
            onClick={() => setActiveExample('useState')}
            style={{
              padding: '12px 16px',
              border: '1px solid #ddd',
              borderRadius: '6px',
              background: 'white',
              cursor: 'pointer',
              textAlign: 'left',
              transition: 'all 0.2s'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = '#4CAF50';
              e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = '#ddd';
              e.currentTarget.style.boxShadow = 'none';
            }}>
            <div
              key='1.3.1.2.1.1'
              style={{ fontWeight: '600', color: '#333', fontFamily: 'monospace', fontSize: '14px' }}>
              useState
            </div>
            <div
              key='1.3.1.2.1.2'
              style={{ fontSize: '12px', color: '#666', marginTop: '4px' }}>
              Manage component state with instant updates and template prediction
            </div>
            
          </button>

          <button
            key='1.3.1.2.2'
            onClick={() => setActiveExample('useEffect')}
            style={{
              padding: '12px 16px',
              border: '1px solid #ddd',
              borderRadius: '6px',
              background: 'white',
              cursor: 'pointer',
              textAlign: 'left',
              transition: 'all 0.2s'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = '#4CAF50';
              e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = '#ddd';
              e.currentTarget.style.boxShadow = 'none';
            }}>
            <div
              key='1.3.1.2.2.1'
              style={{ fontWeight: '600', color: '#333', fontFamily: 'monospace', fontSize: '14px' }}>
              useEffect
            </div>
            <div
              key='1.3.1.2.2.2'
              style={{ fontSize: '12px', color: '#666', marginTop: '4px' }}>
              Run side effects after component renders (timers, subscriptions, etc.)
            </div>
            
          </button>

          <button
            key='1.3.1.2.3'
            onClick={() => setActiveExample('useRef')}
            style={{
              padding: '12px 16px',
              border: '1px solid #ddd',
              borderRadius: '6px',
              background: 'white',
              cursor: 'pointer',
              textAlign: 'left',
              transition: 'all 0.2s'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = '#4CAF50';
              e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = '#ddd';
              e.currentTarget.style.boxShadow = 'none';
            }}>
            <div
              key='1.3.1.2.3.1'
              style={{ fontWeight: '600', color: '#333', fontFamily: 'monospace', fontSize: '14px' }}>
              useRef
            </div>
            <div
              key='1.3.1.2.3.2'
              style={{ fontSize: '12px', color: '#666', marginTop: '4px' }}>
              Create mutable refs that persist across renders without triggering updates
            </div>
            
          </button>
          </div>
        </div>
      </div>
      {/* Active Example Display */}
      {activeExample && (
        <div
          key='1.4.1'
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000
          }}
          onClick={() => setActiveExample(null)}>
          <div
            key='1.4.1.1'
            ref={modalRef}
            tabIndex={-1}
            style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              maxWidth: '90%',
              maxHeight: '90%',
              overflow: 'auto',
              position: 'relative',
              outline: 'none'
            }}
            onClick={(e) => e.stopPropagation()}>
            <button
              key='1.4.1.1.1'
              onClick={() => setActiveExample(null)}
              style={{
                position: 'absolute',
                top: '10px',
                right: '10px',
                padding: '5px 10px',
                border: 'none',
                background: '#f0f0f0',
                borderRadius: '4px',
                cursor: 'pointer'
              }}>
              Close
            </button>
            <h2 key='1.4.1.1.2'>Active Example: {activeExample}</h2>
            <p key='1.4.1.1.3' style={{ color: '#666' }}>
              This is where the example component would render.
              Check the source file in <code key='1.4.1.1.3.2'>Pages/Examples/</code> for the full implementation.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
