# SMA-OS Web Dashboard

[中文](./README.md) | [English](./README_ZH.md)

---

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
GET /api/v1/tasks/{id}/progress # 进度数据 (0-100%)
POST /api/v1/tasks/{id}/pause # 暂停运行中的任务
POST /api/v1/tasks/{id}/resume # 恢复已暂停的任务
```

### 依赖分析 API
```
GET /api/v1/dags/analysis # 完整依赖分析
GET /api/v1/dags/critical-path # 关键路径数据
GET /api/v1/dags/parallelism # 并行度信息
GET /api/v1/tasks/{id}/impact # 失败影响范围
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
│   ├── page.tsx # 主 DAG 查看器
│   ├── layout.tsx # 根布局
│   ├── globals.css # 全局样式
│   └── components/ # UI 组件
│       ├── ProgressBar.tsx
│       ├── TaskControls.tsx
│       ├── TaskTree.tsx
│       ├── DependencyGraph.tsx
│       └── CriticalPathHighlight.tsx
├── public/ # 静态资源
├── package.json # 依赖
└── tsconfig.json # TypeScript 配置 (strict: true)
```

## 组件使用

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
  status="RUNNING" // 显示暂停按钮
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
