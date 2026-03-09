# SMA-OS Protobuf Definitions (`sma-proto`)

This directory contains the universal language-agnostic interface definitions (IDL) used for high-performance RPC communication between the Go Orchestration Layer and the Rust Physical Execution Layer.

## Usage
These `.proto` files should be compiled using `protoc` (Protocol Buffers Compiler) along with the respective `tonic-build` (Rust) and `protoc-gen-go` (Go) plugins to generate the native client/server stubs.

### Defined Services
1. `SandboxManager` (in `sandbox.proto`)
   - Allows Go to request a warm MicroVM from the Firecracker daemon pool.
   - Allows Go to trigger MicroVM teardown upon task completion or security breach.
