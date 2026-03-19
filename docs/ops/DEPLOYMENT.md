# SMA-OS Deployment and Usage Guide / SMA-OS 部署和使用指南

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

## 📋 目录

1. [系统要求](#系统要求)
2. [快速开始](#快速开始)
3. [组件说明](#组件说明)
4. [部署方式](#部署方式)
5. [配置说明](#配置说明)
6. [API 端点](#api-端点)
7. [监控和观测](#监控和观测)
8. [故障排查](#故障排查)

---

## 系统要求

### 最低要求
- **操作系统**: Windows 10/11, Linux (Ubuntu 20.04+), macOS 12+
- **Docker**: 20.10+ with Docker Compose
- **内存**: 8GB RAM (推荐 16GB)
- **存储**: 20GB 可用空间

### eBPF 要求 (仅 Linux)
- Linux 内核 4.19+ 且支持 BTF
- root 权限

---

## 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS
```

### 2. 启动基础设施

```bash
# 复制环境配置
cp .env.example .env

# 编辑 .env 设置密码
# POSTGRES_PASSWORD=your_password
# CLICKHOUSE_PASSWORD=your_password

# 启动所有基础设施服务
docker-compose up -d
```

### 3. 验证服务

```bash
# 检查服务状态
docker ps

# 应该看到 6 个服务运行中:
# - postgres (5432)
# - redis (6379)
# - clickhouse (8123, 9000)
# - weaviate (8088)
# - jaeger (16686)
# - prometheus (9090)
```

### 4. 构建服务

#### Go 服务 (Windows/Linux/macOS)
```bash
cd memory-bus && go build -o bin/ingestion ./ingestion
cd orchestration && go build -o bin/manager ./manager
```

#### Rust 服务 (需要 Linux/Docker)
```bash
# 在 Docker 中构建
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "apt-get update && apt-get install -y protobuf-compiler && \
  cd control-plane && cargo build --release"
```

#### eBPF 程序
```bash
# 使用提供的脚本
./scripts/build-ebpf.sh
```

### 5. 运行服务

```bash
# 使用启动脚本
./scripts/start-services.sh

# 或手动启动
./memory-bus/bin/ingestion &
./orchestration/bin/manager &
```

---

## 组件说明

### Control Plane (Rust)

| 组件 | 功能 | 端口 |
|------|------|------|
| state-engine | 事件溯源状态内核 | 8080 |
| teardown-ctrl | 级联清理控制器 | 8081 |
| identity | 身份认证管理 | 8082 |
| fractal-gateway | eBPF 安全网关 | - |
| fractal-gateway-ebpf | XDP 包过滤 | 内核态 |

### Orchestration (Go)

| 组件 | 功能 | 端口 |
|------|------|------|
| manager | DAG 拓扑执行引擎 | 8083 |
| scheduler | Worker 调度器 | 8084 |
| evaluator | 输出验证器 | 8085 |

### Memory Bus (Go)

| 组件 | 功能 | 端口 |
|------|------|------|
| ingestion | SLM 意图提取 (AI 大模型) | 8086 |
| vector-kv | 向量+KV 存储 | 8087 |

### Observability UI

| 组件 | 功能 | 端口 |
|------|------|------|
| web-dashboard | 实时 DAG 可视化 | 3000 |

---

## 部署方式

### 开发环境

```bash
# 本地运行所有服务
docker-compose up -d
./scripts/start-services.sh
```

### Docker 部署

```bash
# 构建 Docker 镜像
docker build -f docker/Dockerfile.services -t sma-os:latest .

# 运行容器
docker run -d --name sma-os \
  --network host \
  -e DATABASE_URL=postgresql://... \
  sma-os:latest
```

### Kubernetes 部署 (生产环境)

```bash
# 使用 Helm 部署
helm install sma-os ./helm/sma-os \
  --set postgres.enabled=true \
  --set redis.enabled=true
```

---

## 配置说明

### 环境变量

```bash
# .env 文件示例
POSTGRES_USER=sma
POSTGRES_PASSWORD=smaos123
POSTGRES_DB=sma_state
DATABASE_URL=postgresql://sma:smaos123@localhost:5432/sma_state
REDIS_URL=redis://localhost:6379
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=smaos123
DEEPSEEK_API_KEY=your_api_key
```

### 配置文件

- `control-plane/state-engine/config.toml` - 状态引擎配置
- `orchestration/manager/config.yaml` - 调度器配置
- `memory-bus/ingestion/config.yaml` - 意图提取配置

---

## API 端点

### State Engine

```bash
# 健康检查
curl http://localhost:8080/health

# 获取状态快照
curl http://localhost:8080/api/v1/snapshot/{id}

# 追加事件
curl -X POST http://localhost:8080/api/v1/events \
  -H "Content-Type: application/json" \
  -d '{"type": "TaskCreated", "payload": {...}}'
```

### Ingestion

```bash
# 处理用户输入
curl -X POST http://localhost:8086/process \
  -H "Content-Type: application/json" \
  -d '{"input": "create vm in pool A with cpu=2"}'

# 获取指标
curl http://localhost:8086/metrics
```

### Manager

```bash
# 创建 DAG
curl -X POST http://localhost:8083/api/v1/dag \
  -H "Content-Type: application/json" \
  -d '{"nodes": [...], "edges": [...]}'

# 执行 DAG
curl -X POST http://localhost:8083/api/v1/dag/{id}/execute
```

---

## 监控和观测

### Prometheus 指标

```bash
# 访问 Prometheus
http://localhost:9090

# 常用查询
rate(http_requests_total[5m])
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

### Jaeger 追踪

```bash
# 访问 Jaeger UI
http://localhost:16686

# 查看追踪数据
# - 按服务筛选
# - 按时间范围筛选
# - 查看 span 详情
```

### Grafana 仪表盘 (可选)

```bash
# 部署 Grafana
docker run -d --name=grafana -p 3001:3000 grafana/grafana

# 添加 Prometheus 数据源
# 导入预构建仪表盘
```

---

## 故障排查

### 常见问题

#### 1. PostgreSQL 连接失败

```bash
# 检查 PostgreSQL 状态
docker logs sma-os-postgres-1

# 检查连接
docker exec sma-os-postgres-1 psql -U sma -d sma_state -c "SELECT 1"
```

#### 2. Redis 连接超时

```bash
# 检查 Redis
docker exec sma-os-redis-1 redis-cli ping

# 如果返回 PONG，则正常
```

#### 3. eBPF 加载失败

```bash
# 检查内核版本
uname -r  # 应该 >= 4.19

# 检查 BTF 支持
ls /sys/kernel/btf/vmlinux

# 检查权限
# eBPF 需要 root 权限
sudo ./fractal-gateway-ebpf
```

#### 4. AI 大模型 API 错误

```bash
# 检查 API 密钥
echo $AI_API_KEY

# 测试 API 连接（示例）
curl -X POST https://api.example.com/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model": "deepseek-chat", "messages": [{"role": "user", "content": "test"}]}'
```

### 日志查看

```bash
# Docker 容器日志
docker logs -f sma-os-postgres-1

# 服务日志
# 日志输出到 stdout，可重定向到文件
./bin/ingestion 2>&1 | tee ingestion.log
```

---

## 安全建议

1. **生产环境**:
   - 修改所有默认密码
   - 启用 TLS/SSL
   - 配置防火墙规则
   - 定期备份数据库

2. **eBPF**:
   - 仅在可信网络中运行
   - 限制 root 访问
   - 审计所有 eBPF 操作

3. **API 密钥**:
   - 使用环境变量，不要硬编码
   - 定期轮换密钥
   - 限制 API 密钥权限

---

## 更多资源

- [API 文档](../api.md)
- [架构设计](../architecture.md)
- [贡献指南](../contributing/CONTRIBUTING.md)
- [更新日志](./RELEASE_NOTES.md)

---

## 支持

- GitHub Issues: https://github.com/LING71671/SMA-OS/issues
- 文档: https://github.com/LING71671/SMA-OS/wiki

---
---

<a name="english"></a>
## English

## 📋 Table of Contents

1. [System Requirements](#system-requirements)
2. [Quick Start](#quick-start)
3. [Components](#components)
4. [Deployment Methods](#deployment-methods)
5. [Configuration](#configuration)
6. [API Endpoints](#api-endpoints)
7. [Monitoring and Observability](#monitoring-and-observability)
8. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements
- **OS**: Windows 10/11, Linux (Ubuntu 20.04+), macOS 12+
- **Docker**: 20.10+ with Docker Compose
- **Memory**: 8GB RAM (16GB recommended)
- **Storage**: 20GB available space

### eBPF Requirements (Linux only)
- Linux kernel 4.19+ with BTF support
- root privileges

---

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS
```

### 2. Start Infrastructure

```bash
# Copy environment configuration
cp .env.example .env

# Edit .env to set passwords
# POSTGRES_PASSWORD=your_password
# CLICKHOUSE_PASSWORD=your_password

# Start all infrastructure services
docker-compose up -d
```

### 3. Verify Services

```bash
# Check service status
docker ps

# Should see 6 services running:
# - postgres (5432)
# - redis (6379)
# - clickhouse (8123, 9000)
# - weaviate (8088)
# - jaeger (16686)
# - prometheus (9090)
```

### 4. Build Services

#### Go Services (Windows/Linux/macOS)
```bash
cd memory-bus && go build -o bin/ingestion ./ingestion
cd orchestration && go build -o bin/manager ./manager
```

#### Rust Services (requires Linux/Docker)
```bash
# Build in Docker
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "apt-get update && apt-get install -y protobuf-compiler && \
  cd control-plane && cargo build --release"
```

#### eBPF Programs
```bash
# Use provided script
./scripts/build-ebpf.sh
```

### 5. Run Services

```bash
# Use startup script
./scripts/start-services.sh

# Or start manually
./memory-bus/bin/ingestion &
./orchestration/bin/manager &
```

---

## Components

### Control Plane (Rust)

| Component | Function | Port |
|-----------|----------|------|
| state-engine | Event sourcing state kernel | 8080 |
| teardown-ctrl | Cascading cleanup controller | 8081 |
| identity | Identity and access management | 8082 |
| fractal-gateway | eBPF security gateway | - |
| fractal-gateway-ebpf | XDP packet filtering | Kernel mode |

### Orchestration (Go)

| Component | Function | Port |
|-----------|----------|------|
| manager | DAG topology execution engine | 8083 |
| scheduler | Worker scheduler | 8084 |
| evaluator | Output validator | 8085 |

### Memory Bus (Go)

| Component | Function | Port |
|-----------|----------|------|
| ingestion | SLM intent extraction (AI LLM) | 8086 |
| vector-kv | Vector + KV storage | 8087 |

### Observability UI

| Component | Function | Port |
|-----------|----------|------|
| web-dashboard | Real-time DAG visualization | 3000 |

---

## Deployment Methods

### Development Environment

```bash
# Run all services locally
docker-compose up -d
./scripts/start-services.sh
```

### Docker Deployment

```bash
# Build Docker image
docker build -f docker/Dockerfile.services -t sma-os:latest .

# Run container
docker run -d --name sma-os \
  --network host \
  -e DATABASE_URL=postgresql://... \
  sma-os:latest
```

### Kubernetes Deployment (Production)

```bash
# Deploy using Helm
helm install sma-os ./helm/sma-os \
  --set postgres.enabled=true \
  --set redis.enabled=true
```

---

## Configuration

### Environment Variables

```bash
# .env file example
POSTGRES_USER=sma
POSTGRES_PASSWORD=smaos123
POSTGRES_DB=sma_state
DATABASE_URL=postgresql://sma:smaos123@localhost:5432/sma_state
REDIS_URL=redis://localhost:6379
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=smaos123
DEEPSEEK_API_KEY=your_api_key
```

### Configuration Files

- `control-plane/state-engine/config.toml` - State engine configuration
- `orchestration/manager/config.yaml` - Scheduler configuration
- `memory-bus/ingestion/config.yaml` - Intent extraction configuration

---

## API Endpoints

### State Engine

```bash
# Health check
curl http://localhost:8080/health

# Get state snapshot
curl http://localhost:8080/api/v1/snapshot/{id}

# Append event
curl -X POST http://localhost:8080/api/v1/events \
  -H "Content-Type: application/json" \
  -d '{"type": "TaskCreated", "payload": {...}}'
```

### Ingestion

```bash
# Process user input
curl -X POST http://localhost:8086/process \
  -H "Content-Type: application/json" \
  -d '{"input": "create vm in pool A with cpu=2"}'

# Get metrics
curl http://localhost:8086/metrics
```

### Manager

```bash
# Create DAG
curl -X POST http://localhost:8083/api/v1/dag \
  -H "Content-Type: application/json" \
  -d '{"nodes": [...], "edges": [...]}'

# Execute DAG
curl -X POST http://localhost:8083/api/v1/dag/{id}/execute
```

---

## Monitoring and Observability

### Prometheus Metrics

```bash
# Access Prometheus
http://localhost:9090

# Common queries
rate(http_requests_total[5m])
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

### Jaeger Tracing

```bash
# Access Jaeger UI
http://localhost:16686

# View trace data
# - Filter by service
# - Filter by time range
# - View span details
```

### Grafana Dashboard (Optional)

```bash
# Deploy Grafana
docker run -d --name=grafana -p 3001:3000 grafana/grafana

# Add Prometheus data source
# Import pre-built dashboards
```

---

## Troubleshooting

### Common Issues

#### 1. PostgreSQL Connection Failed

```bash
# Check PostgreSQL status
docker logs sma-os-postgres-1

# Check connection
docker exec sma-os-postgres-1 psql -U sma -d sma_state -c "SELECT 1"
```

#### 2. Redis Connection Timeout

```bash
# Check Redis
docker exec sma-os-redis-1 redis-cli ping

# Should return PONG if working
```

#### 3. eBPF Loading Failed

```bash
# Check kernel version
uname -r  # Should be >= 4.19

# Check BTF support
ls /sys/kernel/btf/vmlinux

# Check permissions
# eBPF requires root privileges
sudo ./fractal-gateway-ebpf
```

#### 4. AI LLM API Error

```bash
# Check API key
echo $AI_API_KEY

# Test API connection (example)
curl -X POST https://api.example.com/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model": "deepseek-chat", "messages": [{"role": "user", "content": "test"}]}'
```

### Viewing Logs

```bash
# Docker container logs
docker logs -f sma-os-postgres-1

# Service logs
# Logs output to stdout, can be redirected to file
./bin/ingestion 2>&1 | tee ingestion.log
```

---

## Security Recommendations

1. **Production Environment**:
   - Change all default passwords
   - Enable TLS/SSL
   - Configure firewall rules
   - Regularly backup databases

2. **eBPF**:
   - Run only in trusted networks
   - Restrict root access
   - Audit all eBPF operations

3. **API Keys**:
   - Use environment variables, don't hardcode
   - Rotate keys regularly
   - Limit API key permissions

---

## More Resources

- [API Documentation](../api.md)
- [Architecture Design](../architecture.md)
- [Contributing Guide](../contributing/CONTRIBUTING.md)
- [Release Notes](./RELEASE_NOTES.md)

---

## Support

- GitHub Issues: https://github.com/LING71671/SMA-OS/issues
- Documentation: https://github.com/LING71671/SMA-OS/wiki
