import { useState, useServerTask } from '@minimact/core';

interface AnalysisResult {
  totalItems: number;
  processedAt: Date;
}

export function DataAnalysis() {
  const [datasetId, setDatasetId] = useState<string>('');

  // Simple server task
  const analysis = useServerTask(async (): Promise<AnalysisResult> => {
    const data = await fetch(`/api/datasets/${datasetId}`);
    const parsed = await data.json();

    const items = parsed.items
      .filter(item => item.active)
      .map(item => ({
        ...item,
        score: item.value * 100
      }));

    return {
      totalItems: items.length,
      processedAt: new Date()
    };
  });

  //Streaming server task
  const stream = useServerTask(async function* (query: string) {
    const pageSize = 50;
    let page = 0;

    while (page < 10) {
      const batch = await searchAPI(query, page, pageSize);

      yield {
        items: batch.items,
        page: page,
        total: 10
      };

      page++;
    }
  }, { stream: true });

  return (
    <div>
      <h1>Data Analysis</h1>

      <input
        value={datasetId}
        onChange={e => setDatasetId(e.target.value)}
      />

      {analysis.status === 'idle' && (
        <button onClick={() => analysis.start()}>Start Analysis</button>
      )}

      {analysis.status === 'running' && (
        <div>
          <p>Processing... {Math.round(analysis.progress * 100)}%</p>
          <button onClick={() => analysis.cancel()}>Cancel</button>
        </div>
      )}

      {analysis.status === 'complete' && (
        <div>
          <h2>Results</h2>
          <p>Total: {analysis.result.totalItems}</p>
        </div>
      )}

      {analysis.status === 'error' && (
        <div>
          <p>Error: {analysis.error.message}</p>
          <button onClick={() => analysis.retry()}>Retry</button>
        </div>
      )}

      {/* Streaming results */}
      {stream.running && (
        <p>Loading... {Math.round(stream.progress * 100)}%</p>
      )}

      <ul>
        {stream.chunks.flatMap(chunk => chunk.items).map(item => (
          <li key={item.id}>{item.title}</li>
        ))}
      </ul>
    </div>
  );
}

async function searchAPI(query: string, page: number, pageSize: number) {
  // Mock implementation
  return { items: [] };
}
