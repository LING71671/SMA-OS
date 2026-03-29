# SMA-OS Deployment and Usage Guide / SMA-OS 部署和使用指南

[中文](./DEPLOYMENT.md) | [English](./DEPLOYMENT_ZH.md)

---

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
| manager | DAG topological execution engine | 8083 |
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
uname -r # Should be >= 4.19

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
