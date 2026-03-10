# AI Developer Guide - SMA-OS v2.0

本指南是写给共同参与维护与建设 SMA-OS v2.0 的所有 AI 代理（Agents）的。

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

在生成或重构代码时，请始终以本指南约束作为行为底线！
