interface CriticalPathHighlightProps {
  criticalPath: string[];
  totalNodes: number;
}

export function CriticalPathHighlight({ criticalPath, totalNodes }: CriticalPathHighlightProps) {
  if (criticalPath.length === 0) {
    return <div className="critical-path-container">No critical path found.</div>;
  }

  const pct = totalNodes > 0
    ? ((criticalPath.length / totalNodes) * 100).toFixed(1)
    : '0.0';

  return (
    <div className="critical-path-container">
      <h3>关键路径 ({criticalPath.length} 个任务)</h3>
      <div className="path-display" style={{ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: '4px' }}>
        {criticalPath.map((id, index) => (
          <span key={id} style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <span className="path-node" style={{
              background: '#ff6b6b', color: '#fff',
              padding: '2px 8px', borderRadius: '4px', fontWeight: 600,
            }}>{id}</span>
            {index < criticalPath.length - 1 && (
              <span className="arrow" style={{ color: '#888' }}>→</span>
            )}
          </span>
        ))}
      </div>
      <div className="critical-path-stats" style={{ marginTop: '8px', color: '#666', fontSize: '13px' }}>
        <span>最长执行时间: {criticalPath.length} 步</span>
        {' · '}
        <span>关键任务占比: {pct}%</span>
      </div>
    </div>
  );
}
