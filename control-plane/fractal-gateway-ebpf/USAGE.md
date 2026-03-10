# Fractal Gateway eBPF - Usage Guide

## Quick Start

### 1. Build the eBPF Program

```bash
cd control-plane/fractal-gateway-ebpf
cargo build-bpf --release
```

### 2. Build the Loader

```bash
cd control-plane/fractal-gateway
cargo build --release
```

### 3. Run the Gateway

```bash
# Run with default interface (eth0)
sudo ./target/release/fractal-gateway

# Run with specific interface
sudo ./target/release/fractal-gateway --interface eth1

# Run in dry-run mode (no XDP attachment)
./target/release/fractal-gateway --dry-run
```

## Testing

### Test eBPF Compilation

```bash
cd control-plane/fractal-gateway-ebpf
cargo build-bpf
```

Expected output:
```
Finished release [optimized] target(s) in X.XXs
```

### Test Loader

```bash
cd control-plane/fractal-gateway
cargo test
```

## Troubleshooting

### Build Errors

If you see "target not found":
```bash
rustup target add bpf-unknown-none-elf
```

If you see "cargo-build-bpf not found":
```bash
cargo install cargo-bpf
```

### Runtime Errors

**"Permission denied"**: eBPF requires root privileges
```bash
sudo ./target/release/fractal-gateway
```

**"Interface not found"**: Check available interfaces
```bash
ip link show
```

**"XDP attachment failed"**: Your network driver may not support XDP
Try SKB mode by modifying the flags in the code.

## Advanced Usage

### Custom IP Blocking

The gateway automatically blocks IPs added to the `BLOCKED_IPS` map.
To add custom IPs, modify the `main.rs` file or use the API.

### Performance Tuning

For better performance, use DRV mode instead of default:
```rust
program.attach(&opt.iface, XdpFlags::DRV_MODE)?;
```

## Next Steps

- Task 7: eBPF data collection and reporting
- Task 11: Integration with observability UI
