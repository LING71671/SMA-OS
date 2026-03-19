# Chaos Tests for SMA-OS / SMA-OS 混沌测试

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

用于验证系统弹性和容错能力的混沌工程测试框架。

## 概述

本框架为 SMA-OS 提供自动化混沌测试能力，包括：

- **节点故障**: 杀死容器并验证自动恢复
- **网络分区**: 模拟网络分割并测试分区容错
- **资源耗尽**: 消耗 CPU/内存以测试压力下的系统行为

## 快速开始

### 前提条件

- Docker 和 Docker Compose
- Rust 1.70+
- SMA-OS 服务运行中

### 安装

```bash
# 构建混沌测试
cd chaos-tests
cargo build --release
```

### 运行测试

```bash
# 运行所有场景
cargo run --release -- --scenario all

# 运行特定场景
cargo run --release -- --scenario node-failure
cargo run --release -- --scenario network-partition
cargo run --release -- --scenario resource-exhaustion

# 演练模式（不实际注入故障）
cargo run --release -- --scenario all --dry-run
```

## 配置

编辑 `configs/chaos-config.yaml` 以自定义：

- 目标服务
- 测试持续时间
- 故障概率
- 超时设置

### 配置示例

```yaml
cluster:
  docker_compose_file: "../../docker-compose.yml"
  services:
    - state-engine
    - fractal-gateway
  health_check_url: "http://localhost:8080/health"

scenarios:
  - name: "Node Failure Test"
    type: "node_failure"
    duration: 30
    probability: 1.0
    targets:
      - state-engine

timeouts:
  scenario_timeout_secs: 300
  recovery_timeout_secs: 30
```

## 场景

### 节点故障

杀死容器并通过以下方式验证自动恢复：
- 容器重启
- 从事件日志恢复状态
- 健康检查验证

### 网络分区

注入网络延迟和分区：
- 使用 `tc`（流量控制）注入延迟
- 测试脑裂防护
- 验证分区期间的共识

### 资源耗尽

消耗系统资源以测试压力下的行为：
- 使用无限循环进行 CPU 耗尽
- 使用大内存分配进行内存耗尽
- 使用文件创建进行磁盘耗尽

## 输出

### 文本输出

```
=== SMA-OS Chaos Tests ===
Scenario: Node Failure
Status: PASSED
Duration: 45.23s
```

### JSON 输出

```bash
cargo run --release -- --scenario all --output json
```

```json
{
  "scenario_name": "Node Failure",
  "status": "PASSED",
  "duration_secs": 45.23,
  "errors": [],
  "timestamp": "2026-03-10T12:34:56Z"
}
```

## 集成

### CI/CD 集成

```yaml
# .github/workflows/chaos-tests.yml
name: Chaos Tests
on: [push, pull_request]

jobs:
  chaos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run chaos tests
        run: |
          cd chaos-tests
          cargo run --release -- --scenario all --dry-run
```

### Docker Compose 集成

```yaml
# docker-compose.chaos.yml
version: '3'
services:
  chaos-tests:
    build: ./chaos-tests
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: ["--scenario", "all"]
```

## 故障排查

### "Docker 套接字未找到"

确保 Docker 套接字已挂载：
```bash
docker run -v /var/run/docker.sock:/var/run/docker.sock ...
```

### "权限被拒绝"

使用适当权限运行：
```bash
sudo cargo run --release
```

### "服务恢复失败"

检查服务日志：
```bash
docker logs <container-id>
```

## 最佳实践

1. **先演练**: 始终先在演练模式下测试场景
2. **在预发布环境使用**: 未经充分测试，切勿在生产环境运行混沌测试
3. **密切监控**: 测试期间监控系统指标
4. **设置超时**: 始终配置适当的超时
5. **清理**: 确保即使测试失败也运行清理

## 后续步骤

- 任务 8: 实现特定混沌测试场景
- 任务 12: 在 CI/CD 中自动化混沌测试执行

## 参考资料

- [混沌工程原则](https://principlesofchaos.org/)
- [Chaos Toolkit](https://chaostoolkit.org/)
- [Chaos Mesh](https://chaos-mesh.org/)

---
---

<a name="english"></a>
## English

Chaos engineering test framework for validating system resilience and fault tolerance.

## Overview

This framework provides automated chaos testing capabilities for SMA-OS, including:

- **Node Failure**: Kill containers and verify automatic recovery
- **Network Partition**: Simulate network splits and test partition tolerance
- **Resource Exhaustion**: Consume CPU/memory to test system behavior under pressure

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.70+
- SMA-OS services running

### Installation

```bash
# Build the chaos tests
cd chaos-tests
cargo build --release
```

### Running Tests

```bash
# Run all scenarios
cargo run --release -- --scenario all

# Run specific scenario
cargo run --release -- --scenario node-failure
cargo run --release -- --scenario network-partition
cargo run --release -- --scenario resource-exhaustion

# Dry run (no actual failures injected)
cargo run --release -- --scenario all --dry-run
```

## Configuration

Edit `configs/chaos-config.yaml` to customize:

- Target services
- Test duration
- Failure probability
- Timeout settings

### Example Configuration

```yaml
cluster:
  docker_compose_file: "../../docker-compose.yml"
  services:
    - state-engine
    - fractal-gateway
  health_check_url: "http://localhost:8080/health"

scenarios:
  - name: "Node Failure Test"
    type: "node_failure"
    duration: 30
    probability: 1.0
    targets:
      - state-engine

timeouts:
  scenario_timeout_secs: 300
  recovery_timeout_secs: 30
```

## Scenarios

### Node Failure

Kills containers and verifies automatic recovery through:
- Container restart
- State recovery from event log
- Health check validation

### Network Partition

Injects network latency and partitions:
- Uses `tc` (traffic control) for latency injection
- Tests split-brain prevention
- Validates consensus during partitions

### Resource Exhaustion

Consumes system resources to test behavior under pressure:
- CPU exhaustion using infinite loops
- Memory exhaustion using large allocations
- Disk exhaustion using file creation

## Output

### Text Output

```
=== SMA-OS Chaos Tests ===
Scenario: Node Failure
Status: PASSED
Duration: 45.23s
```

### JSON Output

```bash
cargo run --release -- --scenario all --output json
```

```json
{
  "scenario_name": "Node Failure",
  "status": "PASSED",
  "duration_secs": 45.23,
  "errors": [],
  "timestamp": "2026-03-10T12:34:56Z"
}
```

## Integration

### CI/CD Integration

```yaml
# .github/workflows/chaos-tests.yml
name: Chaos Tests
on: [push, pull_request]

jobs:
  chaos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run chaos tests
        run: |
          cd chaos-tests
          cargo run --release -- --scenario all --dry-run
```

### Docker Compose Integration

```yaml
# docker-compose.chaos.yml
version: '3'
services:
  chaos-tests:
    build: ./chaos-tests
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: ["--scenario", "all"]
```

## Troubleshooting

### "Docker socket not found"

Ensure Docker socket is mounted:
```bash
docker run -v /var/run/docker.sock:/var/run/docker.sock ...
```

### "Permission denied"

Run with appropriate privileges:
```bash
sudo cargo run --release
```

### "Service failed to recover"

Check service logs:
```bash
docker logs <container-id>
```

## Best Practices

1. **Start with dry-run**: Always test scenarios in dry-run mode first
2. **Use in staging**: Never run chaos tests in production without thorough testing
3. **Monitor closely**: Watch system metrics during tests
4. **Set timeouts**: Always configure appropriate timeouts
5. **Clean up**: Ensure cleanup runs even on test failure

## Next Steps

- Task 8: Implement specific chaos test scenarios
- Task 12: Automate chaos test execution in CI/CD

## References

- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Chaos Toolkit](https://chaostoolkit.org/)
- [Chaos Mesh](https://chaos-mesh.org/)
