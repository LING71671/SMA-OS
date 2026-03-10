# Fractal Gateway eBPF

Nanosecond-level network filtering using eBPF XDP (eXpress Data Path) for SMA-OS.

## Overview

This module provides an eBPF-based network filtering solution that operates at the kernel level, providing:

- **Nanosecond-level packet filtering**: Drop malicious packets before they reach the application layer
- **Dynamic IP blocking**: Update blocked IP list without reloading the program
- **Zero-copy operation**: Packets are filtered in-kernel without copying to userspace
- **Resource isolation**: Protect execution layer VMs from unauthorized access

## Architecture

```
User Space (Fractal Gateway)
    ↓
eBPF Runtime (aya)
    ↓
XDP Program (fractal_gateway)
    ↓
Network Interface (eth0)
```

## Building

### Prerequisites

- Linux kernel 4.19+ with eBPF support
- `rustup` with `bpf` target installed
- `libbpf` development files

Install the BPF target:

```bash
rustup target add bpf-unknown-none-elf
```

### Build Commands

```bash
# Build the eBPF program
cd control-plane/fractal-gateway-ebpf
cargo build-bpf --release

# Build the loader
cargo build --release
```

## Usage

### Loading the eBPF Program

```rust
use fractal_gateway_ebpf::FractalGatewayEbpf;

fn main() -> anyhow::Result<()> {
    // Load the eBPF program
    let mut gateway = FractalGatewayEbpf::load()?;
    
    // Attach to network interface
    gateway.attach_xdp("eth0")?;
    
    // Block malicious IP
    let malicious_ip = "192.168.1.100";
    let ip_num = fractal_gateway_ebpf::ip_to_u32(malicious_ip)?;
    gateway.block_ip(ip_num)?;
    
    println!("eBPF program loaded and attached");
    
    // Keep the program running
    std::thread::sleep(std::time::Duration::from_secs(3600));
    
    // Cleanup
    gateway.detach()?;
    
    Ok(())
}
```

### CLI Example

```bash
# Run the gateway daemon
cargo run --release -- --interface eth0

# Block an IP address
cargo run --release -- block-ip 192.168.1.100

# Unblock an IP address
cargo run --release -- unblock-ip 192.168.1.100

# List blocked IPs
cargo run --release -- list-blocked
```

## API Reference

### `FractalGatewayEbpf::load() -> Result<Self>`

Load the eBPF program from the embedded ELF file.

### `attach_xdp(interface: &str) -> Result<()>`

Attach the XDP program to the specified network interface.

### `detach() -> Result<()>`

Detach the XDP program from all interfaces.

### `block_ip(ip: u32) -> Result<()>`

Add an IP address to the blocked list.

### `unblock_ip(ip: u32) -> Result<()>`

Remove an IP address from the blocked list.

### `get_blocked_count() -> Result<usize>`

Get the number of currently blocked IP addresses.

## Configuration

### XDP Flags

The XDP program can be attached with different flags:

- `XdpFlags::default()`: Default mode, allows driver fallback
- `XdpFlags::DRV_MODE`: Force driver mode
- `XdpFlags::SKB_MODE`: Force SKB mode (slower but more compatible)

### Map Configuration

| Map Name | Type | Max Entries | Description |
|----------|------|-------------|-------------|
| `BLOCKED_IPS` | HashMap | 1024 | Blocked IP addresses |

## Testing

```bash
# Run unit tests
cargo test

# Run integration test (requires root)
sudo cargo test --test integration
```

## Troubleshooting

### "eBPF not supported"

Ensure your kernel has eBPF support:

```bash
zcat /proc/config.gz | grep CONFIG_BPF
```

Should show `CONFIG_BPF=y` and `CONFIG_BPF_SYSCALL=y`.

### "Permission denied"

eBPF programs require elevated privileges:

```bash
# Run with sudo
sudo ./target/release/fractal-gateway

# Or use capabilities
setcap cap_bpf+ep ./target/release/fractal-gateway
```

### "XDP attachment failed"

Some network drivers don't support XDP. Try SKB mode:

```rust
let flags = XdpFlags::SKB_MODE;
xdp_program.attach("eth0", flags)?;
```

## Performance

Expected performance characteristics:

- **Packet filtering latency**: < 100ns per packet
- **Memory overhead**: < 1MB for 1024 blocked IPs
- **CPU overhead**: < 1% at 10Gbps throughput

## Security Considerations

1. **Root required**: Loading eBPF programs requires root or `CAP_BPF`
2. **Program verification**: All eBPF programs are verified by the kernel
3. **Map access control**: Only the loading process can access eBPF maps
4. **Audit logging**: All block/unblock operations are logged

## References

- [Aya Documentation](https://aya-rs.dev/)
- [Linux eBPF Documentation](https://ebpf.io/)
- [XDP Documentation](https://github.com/iovisor/bpf-docs/blob/master/eBPF_Introduction.rst)

## License

Same as SMA-OS project license.
