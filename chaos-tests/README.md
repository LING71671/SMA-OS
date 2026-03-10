# Chaos Tests for SMA-OS

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
