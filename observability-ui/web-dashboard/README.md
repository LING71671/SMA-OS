# SMA-OS Web Dashboard

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

SMA-OS 的实时可观测性 UI，用于 DAG 可视化、任务管理和依赖分析。

## 功能特性

- **DAG 可视化**: 使用 ReactFlow 的交互式图形
- **任务进度**: 带百分比显示的实时进度条
- **任务控制**: 基于状态显示的暂停/恢复按钮
- **依赖图**: 带交互节点的可视化依赖分析
- **关键路径**: 高亮显示的最长执行路径
- **任务层级**: 父/子任务关系的树形视图

## 组件

| 组件 | 用途 | 文件 |
|------|------|------|
| ProgressBar | 可视化进度指示器 (0-100%) | `src/app/components/ProgressBar.tsx` |
| TaskControls | 带状态逻辑的暂停/恢复按钮 | `src/app/components/TaskControls.tsx` |
| TaskTree | 层级任务结构展示 | `src/app/components/TaskTree.tsx` |
| DependencyGraph | 交互式依赖图 | `src/app/components/DependencyGraph.tsx` |
| CriticalPathHighlight | 关键路径覆盖层 | `src/app/components/CriticalPathHighlight.tsx` |

## 快速开始

```bash
# 安装依赖
npm install

# 开发服务器
npm run dev

# 生产构建
npm run build

# 类型检查
npx tsc --noEmit

# 代码检查
npm run lint
```

在浏览器中打开 [http://localhost:3000](http://localhost:3000)。

## API 集成

仪表盘连接以下后端端点：

### 任务管理 API
```
GET /api/v1/tasks/{id}/progress    # 进度数据 (0-100%)
POST /api/v1/tasks/{id}/pause      # 暂停运行中的任务
POST /api/v1/tasks/{id}/resume     # 恢复已暂停的任务
```

### 依赖分析 API
```
GET /api/v1/dags/analysis          # 完整依赖分析
GET /api/v1/dags/critical-path     # 关键路径数据
GET /api/v1/dags/parallelism       # 并行度信息
GET /api/v1/tasks/{id}/impact      # 失败影响范围
```

## 技术栈

| 技术 | 用途 |
|------|------|
| Next.js 14 | React 框架 (App Router) |
| ReactFlow | DAG 和依赖图可视化 |
| Framer Motion | 动画和过渡效果 |
| Lucide React | 图标库 |
| TypeScript | 类型安全 (严格模式) |
| Tailwind CSS | 样式 (可选，主要使用自定义 CSS) |

## 项目结构

```
web-dashboard/
├── app/
│   ├── page.tsx           # 主 DAG 查看器
│   ├── layout.tsx         # 根布局
│   ├── globals.css        # 全局样式
│   └── components/        # UI 组件
│       ├── ProgressBar.tsx
│       ├── TaskControls.tsx
│       ├── TaskTree.tsx
│       ├── DependencyGraph.tsx
│       └── CriticalPathHighlight.tsx
├── public/                # 静态资源
├── package.json           # 依赖
└── tsconfig.json          # TypeScript 配置 (strict: true)
```

## 组件使用

### ProgressBar
```tsx
import { ProgressBar } from '@/app/components/ProgressBar';

<ProgressBar
  taskId="task-1"
  progress={75.5}  // 0-100
/>
```

### TaskControls
```tsx
import { TaskControls } from '@/app/components/TaskControls';

<TaskControls
  taskId="task-1"
  status="RUNNING"  // 显示暂停按钮
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

## 样式

- **全局样式**: `app/globals.css` 使用玻璃拟态效果
- **Framer Motion**: 所有动画使用 `<motion.div>` 组件
- **Lucide 图标**: `Activity`, `Zap`, `Server`, `ShieldAlert` 等

## 了解更多

- [Next.js 文档](https://nextjs.org/docs)
- [ReactFlow 文档](https://reactflow.dev/docs/)
- [Framer Motion 文档](https://www.framer.com/motion/)

## 部署到 Vercel

最简单的部署方式是使用 [Vercel 平台](https://vercel.com):

```bash
npm run build
# 通过 Vercel CLI 或 GitHub 集成部署
```

---
---

<a name="english"></a>
## English

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
GET /api/v1/tasks/{id}/progress    # Progress data (0-100%)
POST /api/v1/tasks/{id}/pause      # Pause running task
POST /api/v1/tasks/{id}/resume     # Resume paused task
```

### Dependency Analysis API
```
GET /api/v1/dags/analysis          # Full dependency analysis
GET /api/v1/dags/critical-path     # Critical path data
GET /api/v1/dags/parallelism       # Parallelism info
GET /api/v1/tasks/{id}/impact      # Failure impact scope
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
│   ├── page.tsx           # Main DAG viewer
│   ├── layout.tsx         # Root layout
│   ├── globals.css        # Global styles
│   └── components/        # UI components
│       ├── ProgressBar.tsx
│       ├── TaskControls.tsx
│       ├── TaskTree.tsx
│       ├── DependencyGraph.tsx
│       └── CriticalPathHighlight.tsx
├── public/                # Static assets
├── package.json           # Dependencies
└── tsconfig.json          # TypeScript config (strict: true)
```

## Component Usage

### ProgressBar
```tsx
import { ProgressBar } from '@/app/components/ProgressBar';

<ProgressBar
  taskId="task-1"
  progress={75.5}  // 0-100
/>
```

### TaskControls
```tsx
import { TaskControls } from '@/app/components/TaskControls';

<TaskControls
  taskId="task-1"
  status="RUNNING"  // Shows pause button
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
