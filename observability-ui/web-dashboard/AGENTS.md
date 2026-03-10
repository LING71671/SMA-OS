# Web Dashboard Guide

**Location**: `observability-ui/web-dashboard/`  
**Domain**: Real-time DAG visualization and observability  
**Language**: TypeScript / Next.js  
**Score**: 15/25 (frontend complexity, distinct UI domain)

## Overview

Next.js-based observability UI providing real-time DAG visualization, task flow monitoring, and execution logs. Uses ReactFlow for interactive graph rendering with Framer Motion animations.

## Structure

```
web-dashboard/
├── app/                # Next.js app directory
│   ├── page.tsx       # Main DAG viewer component
│   └── globals.css    # Global styles
├── src/               # Source directory
│   └── app/          # Additional pages/components
├── public/           # Static assets
├── package.json      # Dependencies (reactflow, framer-motion, lucide-react)
└── tsconfig.json    # TypeScript config (strict: true)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| DAG visualization | `app/page.tsx` | ReactFlow canvas, node definitions |
| Live logs panel | `app/page.tsx:75-92` | Side panel with streaming logs |
| Node definitions | `app/page.tsx:16-24` | Initial nodes: Ingestion, Orchestrator, Evaluator, Worker |
| Edge definitions | `app/page.tsx:26-34` | Connections with animated flows |
| Top banner | `app/page.tsx:45-58` | Header with neon text, icons |

## Conventions (This Module)

### Component Structure
```typescript
"use client";

export interface DagViewerProps {
  initialNodes: Node[];
  initialEdges: Edge[];
}

export default function DagViewer({ initialNodes }: DagViewerProps) {
  // 1. Hooks first
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // 2. Event handlers (useCallback)
  const onNodeClick = useCallback((node: Node) => {
    // Handler logic
  }, []);

  // 3. Render
  return <div>{/* JSX */}</div>;
}
```

### Styling
- **Tailwind CSS**: Not used (custom CSS in `globals.css`)
- **Framer Motion**: For all animations (`motion.div`, `AnimatePresence`)
- **Lucide React**: Icon library (`Activity`, `ShieldAlert`, `Zap`, `Server`)
- **ReactFlow**: DAG visualization with custom nodes

## Anti-Patterns (This Module)

### Forbidden
```typescript
// NEVER use inline styles for layout
<div style={{ position: "absolute" }}>  // WRONG

// ALWAYS use CSS classes
<div className="absolute top-0 left-0">  // CORRECT
```

### State Management
```typescript
// WRONG: Direct state mutation
nodes.push(newNode);
setNodes(nodes);

// CORRECT: Immutable updates
setNodes(nds => [...nds, newNode]);
```

### Event Handlers
```typescript
// WRONG: New function every render
<ReactFlow onNodeClick={(node) => handleNodeClick(node)} />

// CORRECT: Memoized handler
const onNodeClick = useCallback((node: Node) => {
  handleNodeClick(node);
}, []);
<ReactFlow onNodeClick={onNodeClick} />
```

## Unique Styles

### Import Order
```typescript
// 1. React and Next.js
import { useState, useCallback } from "react";

// 2. Third-party libraries
import ReactFlow, { Background, Controls } from "reactflow";
import { motion } from "framer-motion";
import { Activity, Zap } from "lucide-react";

// 3. Internal components
import { DagNode } from "@/components/DagNode";

// 4. Styles
import "reactflow/dist/style.css";
```

### Animation Patterns
```typescript
<motion.div
  initial={{ y: -50, opacity: 0 }}
  animate={{ y: 0, opacity: 1 }}
  transition={{ duration: 0.8 }}
  className="glass-panel"
>
```

## Commands

```bash
# Install dependencies
npm install

# Development
npm run dev

# Build
npm run build

# Lint
npm run lint

# Type check
npx tsc --noEmit
```

## Dependencies

| Package | Purpose |
|---------|---------|
| reactflow | DAG visualization |
| framer-motion | Animations |
| lucide-react | Icons |
| next | React framework |
| typescript | Type checking (strict mode) |

## Notes

- **Next.js App Router**: Uses `app/` directory structure (not `pages/`)
- **Client components**: All interactive components use `"use client"` directive
- **Real-time updates**: Designed for WebSocket/GraphQL subscription integration
- **Visual quality**: High emphasis on animations, glass-morphism effects
- **Responsive**: Designed for desktop-first, scales to tablet
