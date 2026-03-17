# AI Developer Guide - SMA-OS

本指南是写给共同参与维护与建设 SMA-OS 的所有 AI 代理（Agents）的。

## 1. 核心设计原则
**必须时刻牢记以下系统级原则：**
- **零妥协的安全性**：任何代码不能绕过系统已有的 eBPF 沙箱。
- **不可变日志 (Append-only Event Sourcing)**：状态变更是基于事件源追踪的；数据库中的决定和状态更改需以附加版本化的日志 (Version/TraceID/Tenant) 实现。
- **性能与隔离并重**：控制面 (Rust) 追求内存安全性与纳米级性能；调度平面 (Go) 强调高并发及代码的可读性、可扩展性。
- **中文与英文并存**：注释及文档应该对中文母语使用者非常友好，优先提供清晰的中文原理解释。

## 2. 架构技术栈强制规范
- **Rust (控制面与执行层)**
  - 使用 `Cargo workspace` 管理依赖。
  - 需要和 C/Linux Kernel 接口交互时极力使用安全的包装层（例如 `aya` 或 `ebpf` 生态）。
  - 在正式提交影响状态转换的 Rust 代码前，须考虑到 TLA+ 模型并撰写相应单元测试。
- **Go (认知编排与数据面)**
  - 使用传统的 `go mod` 进行模块结构化。
  - 大量使用 Goroutines，务必确保通道（Channels）与 Context 传参的正确结束以避免 Goroutine 泄漏。
- **Next.js/React (可观测 UI)**
  - 永远保持前端极高的视觉品质！不得使用简陋或者"临时工样式"。
  - 利用 3D 和动画展示复杂数据关系（DAG 树）。

## 3. 重要目录速查
- `control-plane/` 包含了所有关于内核状态、eBPF 拦截与强验证系统的核心逻辑。
- `orchestration/` 包含了高并发管理节点分配的 DAG 系统源码。
- `execution-layer/` 主要为与 Firecracker 虚拟机 API 通信的基础实现。
- `observability-ui/web-dashboard/` 最为关键的交互门面。

## 4. 故障排查逻辑

遇到微服务不能相互通信时：
1. 总是首先检查对应的 OTel Trace ID 和 Jaeger 的依赖拓扑链。
2. 检查 `docker-compose.yml` 内的环境变量是否正确注入。
3. 检查控制面的探针（eBPF）是否已经熔断（Circuit Broken）了该线程 / 容器的执行。

## 5. 推送前必须检查清单

**在提交代码前，必须完成以下步骤：**

### 5.1 模块文档更新

如果修改了某个模块的代码，必须同步更新对应的 AGENTS.md 文档：

| 模块路径 | 需要更新的文档 |
|---------|---------------|
| `control-plane/state-engine/` | `control-plane/state-engine/AGENTS.md` |
| `control-plane/identity/` | `control-plane/identity/AGENTS.md` |
| `control-plane/teardown-ctrl/` | `control-plane/teardown-ctrl/AGENTS.md` |
| `control-plane/fractal-gateway/` | `control-plane/fractal-gateway/AGENTS.md` |
| `control-plane/fractal-gateway-ebpf/` | `control-plane/fractal-gateway-ebpf/AGENTS.md` |
| `orchestration/manager/` | `orchestration/manager/AGENTS.md` |
| `orchestration/scheduler/` | `orchestration/scheduler/AGENTS.md` |
| `orchestration/evaluator/` | `orchestration/evaluator/AGENTS.md` |
| `memory-bus/ingestion/` | `memory-bus/ingestion/AGENTS.md` |
| `memory-bus/vector-kv/` | `memory-bus/vector-kv/AGENTS.md` |
| `observability-ui/web-dashboard/` | `observability-ui/web-dashboard/AGENTS.md` |

### 5.2 文档更新内容检查

更新模块文档时，必须包含：
- [ ] **新增/修改的功能描述**
- [ ] **API 变更说明**（如果有）
- [ ] **使用示例**（如果添加了新功能）
- [ ] **依赖变更**（如果新增了依赖）
- [ ] **反模式警示**（如果适用）

### 5.3 全局文档检查

以下情况需要更新全局文档：
- 修改了 README.md 中提到的功能 → 更新 `README.md`
- 添加了新模块 → 更新 `README.md` 组件列表
- 版本号变更 → 运行 `./scripts/update-version.sh <version>`
- 新增 CI/CD 配置 → 更新 `docs/DEPLOYMENT.md`

### 5.4 版本号管理

**版本号统一策略：**
- 项目版本存储在 `VERSION` 文件中
- Rust crate 使用 workspace 继承：`version.workspace = true`
- 更新版本号时运行：`./scripts/update-version.sh <new_version>`

## 6. 代码提交规范

### 提交信息格式
```
<type>: <subject>

<body>

<footer>
```

### Type 类型
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

### 示例
```
feat: 添加 DAG 执行超时配置

- 支持全局超时设置
- 支持单个任务超时配置
- 超时后自动清理资源
- 更新了 manager/AGENTS.md 文档

Closes #123
```

在生成或重构代码时，请始终以本指南约束作为行为底线！
