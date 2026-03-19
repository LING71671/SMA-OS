# SMA-OS Protobuf Definitions (`sma-proto`)

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

本目录包含通用的语言无关接口定义（IDL），用于 Go 编排层和 Rust 物理执行层之间的高性能 RPC 通信。

## 用法

这些 `.proto` 文件应使用 `protoc`（Protocol Buffers 编译器）以及相应的 `tonic-build`（Rust）和 `protoc-gen-go`（Go）插件进行编译，以生成原生客户端/服务端存根。

### 定义的服务

1. `SandboxManager`（在 `sandbox.proto` 中）
   - 允许 Go 从 Firecracker 守护进程池请求热 MicroVM
   - 允许 Go 在任务完成或安全违规时触发 MicroVM 拆卸

---
---

<a name="english"></a>
## English

This directory contains the universal language-agnostic interface definitions (IDL) used for high-performance RPC communication between the Go Orchestration Layer and the Rust Physical Execution Layer.

## Usage

These `.proto` files should be compiled using `protoc` (Protocol Buffers Compiler) along with the respective `tonic-build` (Rust) and `protoc-gen-go` (Go) plugins to generate the native client/server stubs.

### Defined Services

1. `SandboxManager` (in `sandbox.proto`)
   - Allows Go to request a warm MicroVM from the Firecracker daemon pool.
   - Allows Go to trigger MicroVM teardown upon task completion or security breach.
