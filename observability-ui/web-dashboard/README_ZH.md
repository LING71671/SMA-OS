# SMA-OS Web Dashboard

[中文](./README.md) | [English](./README_ZH.md)

---

Real-time observability UI for DAG visualization, task management, and dependency analysis.

## Features

- **DAG Visualization**: Interactive graph with ReactFlow
- **Task Progress**: Real-time progress bars with percentage display
- **Task Controls**: Pause/resume buttons with status-based visibility
- **Dependency Graph**: Visual dependency analysis with interactive nodes
- **Critical Path**: Highlighted longest execution path
- **Task Hierarchy**: Tree view of parent/child task relationships

## Components

| Component | Purpose | File |
|-----------|---------|------|
| ProgressBar | Visual progress indicator (0-100%) | `src/app/components/ProgressBar.tsx` |
| TaskControls | Pause/resume buttons with status logic | `src/app/components/TaskControls.tsx` |
| TaskTree | Hierarchical task structure display | `src/app/components/TaskTree.tsx` |
| DependencyGraph | Interactive dependency graph | `src/app/components/DependencyGraph.tsx` |
| CriticalPathHighlight | Critical path overlay | `src/app/components/CriticalPathHighlight.tsx` |

## Getting Started

```bash
# Install dependencies
npm install

# Development server
npm run dev

# Build for production
npm run build

# Type check
npx tsc --noEmit

# Lint
npm run lint
```

Open [http://localhost:3000](http://localhost:3000) with your browser.

## API Integration

The dashboard connects to these backend endpoints:

### Task Management API
```
GET /api/v1/tasks/{id}/progress # Progress data (0-100%)
POST /api/v1/tasks/{id}/pause # Pause running task
POST /api/v1/tasks/{id}/resume # Resume paused task
```

### Dependency Analysis API
```
GET /api/v1/dags/analysis # Full dependency analysis
GET /api/v1/dags/critical-path # Critical path data
GET /api/v1/dags/parallelism # Parallelism info
GET /api/v1/tasks/{id}/impact # Failure impact scope
```

## Tech Stack

| Technology | Purpose |
|------------|---------|
| Next.js 14 | React framework (App Router) |
| ReactFlow | DAG and dependency graph visualization |
| Framer Motion | Animations and transitions |
| Lucide React | Icon library |
| TypeScript | Type safety (strict mode) |
| Tailwind CSS | Styling (optional, mostly custom CSS) |

## Project Structure

```
web-dashboard/
├── app/
│   ├── page.tsx # Main DAG viewer
│   ├── layout.tsx # Root layout
│   ├── globals.css # Global styles
│   └── components/ # UI components
│       ├── ProgressBar.tsx
│       ├── TaskControls.tsx
│       ├── TaskTree.tsx
│       ├── DependencyGraph.tsx
│       └── CriticalPathHighlight.tsx
├── public/ # Static assets
├── package.json # Dependencies
└── tsconfig.json # TypeScript config (strict: true)
```

## Component Usage

### ProgressBar
```tsx
import { ProgressBar } from '@/app/components/ProgressBar';

<ProgressBar
  taskId="task-1"
  progress={75.5} // 0-100
/>
```

### TaskControls
```tsx
import { TaskControls } from '@/app/components/TaskControls';

<TaskControls
  taskId="task-1"
  status="RUNNING" // Shows pause button
  onPause={() => handlePause()}
  onResume={() => handleResume()}
/>
```

### DependencyGraph
```tsx
import { DependencyGraph } from '@/app/components/DependencyGraph';

<DependencyGraph
  graph={analysisResult.graph}
  highlightCritical={true}
/>
```

## Styling

- **Global styles**: `app/globals.css` with glass-morphism effects
- **Framer Motion**: All animations use `<motion.div>` components
- **Lucide Icons**: `Activity`, `Zap`, `Server`, `ShieldAlert`, etc.

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [ReactFlow Documentation](https://reactflow.dev/docs/)
- [Framer Motion Documentation](https://www.framer.com/motion/)

## Deploy on Vercel

The easiest way to deploy is using the [Vercel Platform](https://vercel.com):

```bash
npm run build
# Deploy via Vercel CLI or GitHub integration
```
