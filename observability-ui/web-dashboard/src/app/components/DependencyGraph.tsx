interface DependencyNode {
  id: string;
  status: string;
  dependencies: string[];
  dependents: string[];
  depth: number;
  is_critical: boolean;
}

interface DependencyEdge {
  from: string;
  to: string;
  is_critical: boolean;
}

interface DependencyGraphData {
  nodes: DependencyNode[];
  edges: DependencyEdge[];
}

interface DependencyGraphProps {
  graph: DependencyGraphData;
  highlightCritical?: boolean;
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'COMPLETED': return '#4caf50';
    case 'RUNNING':   return '#2196f3';
    case 'FAILED':    return '#f44336';
    case 'PAUSED':    return '#ff9800';
    default:          return '#9e9e9e';
  }
}

export function DependencyGraph({ graph, highlightCritical = true }: DependencyGraphProps) {
  // Group nodes by depth for layout
  const maxDepth = Math.max(...graph.nodes.map(n => n.depth), 0);
  const colWidth = 160;
  const rowHeight = 80;

  const nodePositions: Record<string, { x: number; y: number }> = {};
  const byDepth: Record<number, DependencyNode[]> = {};
  for (const node of graph.nodes) {
    byDepth[node.depth] = byDepth[node.depth] ?? [];
    byDepth[node.depth].push(node);
  }
  for (const [depth, nodes] of Object.entries(byDepth)) {
    nodes.forEach((node, i) => {
      nodePositions[node.id] = {
        x: Number(depth) * colWidth + 20,
        y: i * rowHeight + 20,
      };
    });
  }

  const svgWidth = (maxDepth + 1) * colWidth + 120;
  const svgHeight = Math.max(...graph.nodes.map((_, i) => i * rowHeight), 1) + 100;

  return (
    <div className="dependency-graph" style={{ overflowX: 'auto' }}>
      <svg width={svgWidth} height={svgHeight} aria-label="Dependency graph">
        <defs>
          <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
            <path d="M0,0 L0,6 L8,3 z" fill="#888" />
          </marker>
          <marker id="arrow-critical" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
            <path d="M0,0 L0,6 L8,3 z" fill="#ff6b6b" />
          </marker>
        </defs>

        {/* Edges */}
        {graph.edges.map(edge => {
          const from = nodePositions[edge.from];
          const to = nodePositions[edge.to];
          if (!from || !to) return null;
          const isCrit = edge.is_critical && highlightCritical;
          return (
            <line
              key={`${edge.from}-${edge.to}`}
              x1={from.x + 50} y1={from.y + 18}
              x2={to.x} y2={to.y + 18}
              stroke={isCrit ? '#ff6b6b' : '#888'}
              strokeWidth={isCrit ? 2 : 1}
              markerEnd={isCrit ? 'url(#arrow-critical)' : 'url(#arrow)'}
            />
          );
        })}

        {/* Nodes */}
        {graph.nodes.map(node => {
          const pos = nodePositions[node.id];
          if (!pos) return null;
          const isCrit = node.is_critical && highlightCritical;
          return (
            <g key={node.id} transform={`translate(${pos.x},${pos.y})`}>
              <rect
                width={100} height={36} rx={6}
                fill={isCrit ? '#ff6b6b' : getStatusColor(node.status)}
                stroke={isCrit ? '#c0392b' : '#555'}
                strokeWidth={1}
              />
              <text x={50} y={22} textAnchor="middle" fill="#fff" fontSize={12}>
                {node.id}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
