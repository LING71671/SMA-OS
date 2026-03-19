# Fractal Gateway eBPF / 分形网关 eBPF

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

基于 eBPF XDP（eXpress Data Path）的纳秒级网络过滤，用于 SMA-OS。

## 概述

本模块提供基于 eBPF 的网络过滤解决方案，运行于内核级别，提供：

- **纳秒级包过滤**: 在恶意数据包到达应用层之前丢弃
- **动态 IP 封禁**: 无需重新加载程序即可更新封禁 IP 列表
- **零拷贝操作**: 数据包在内核中过滤，无需拷贝到用户空间
- **资源隔离**: 保护执行层 VM 免受未授权访问

## 架构

```
用户空间 (Fractal Gateway)
↓
eBPF 运行时 (aya)
↓
XDP 程序 (fractal_gateway)
↓
网络接口 (eth0)
```

## 构建

### 前提条件

- 支持 eBPF 的 Linux 内核 4.19+
- 已安装 `bpf` 目标的 `rustup`
- `libbpf` 开发文件

安装 BPF 目标：

```bash
rustup target add bpf-unknown-none-elf
```

### 构建命令

```bash
# 构建 eBPF 程序
cd control-plane/fractal-gateway-ebpf
cargo build-bpf --release

# 构建加载器
cargo build --release
```

## 使用

### 加载 eBPF 程序

```rust
use fractal_gateway_ebpf::FractalGatewayEbpf;

fn main() -> anyhow::Result<()> {
    // 加载 eBPF 程序
    let mut gateway = FractalGatewayEbpf::load()?;

    // 附加到网络接口
    gateway.attach_xdp("eth0")?;

    // 封禁恶意 IP
    let malicious_ip = "192.168.1.100";
    let ip_num = fractal_gateway_ebpf::ip_to_u32(malicious_ip)?;
    gateway.block_ip(ip_num)?;

    println!("eBPF 程序已加载并附加");

    // 保持程序运行
    std::thread::sleep(std::time::Duration::from_secs(3600));

    // 清理
    gateway.detach()?;

    Ok(())
}
```

### CLI 示例

```bash
# 运行网关守护进程
cargo run --release -- --interface eth0

# 封禁 IP 地址
cargo run --release -- block-ip 192.168.1.100

# 解封 IP 地址
cargo run --release -- unblock-ip 192.168.1.100

# 列出已封禁 IP
cargo run --release -- list-blocked
```

## API 参考

### `FractalGatewayEbpf::load() -> Result<Self>`

从嵌入的 ELF 文件加载 eBPF 程序。

### `attach_xdp(interface: &str) -> Result<()>`

将 XDP 程序附加到指定网络接口。

### `detach() -> Result<()>`

从所有接口分离 XDP 程序。

### `block_ip(ip: u32) -> Result<()>`

将 IP 地址添加到封禁列表。

### `unblock_ip(ip: u32) -> Result<()>`

从封禁列表移除 IP 地址。

### `get_blocked_count() -> Result<usize>`

获取当前封禁的 IP 地址数量。

## 配置

### XDP 标志

XDP 程序可以使用不同标志附加：

- `XdpFlags::default()`: 默认模式，允许驱动回退
- `XdpFlags::DRV_MODE`: 强制驱动模式
- `XdpFlags::SKB_MODE`: 强制 SKB 模式（较慢但更兼容）

### 映射配置

| 映射名称 | 类型 | 最大条目数 | 描述 |
|----------|------|------------|------|
| `BLOCKED_IPS` | HashMap | 1024 | 封禁的 IP 地址 |

## 测试

```bash
# 运行单元测试
cargo test

# 运行集成测试（需要 root）
sudo cargo test --test integration
```

## 故障排查

### "eBPF 不支持"

确保内核支持 eBPF：

```bash
zcat /proc/config.gz | grep CONFIG_BPF
```

应显示 `CONFIG_BPF=y` 和 `CONFIG_BPF_SYSCALL=y`。

### "权限被拒绝"

eBPF 程序需要提升权限：

```bash
# 使用 sudo 运行
sudo ./target/release/fractal-gateway

# 或使用 capabilities
setcap cap_bpf+ep ./target/release/fractal-gateway
```

### "XDP 附加失败"

某些网络驱动不支持 XDP。尝试 SKB 模式：

```rust
let flags = XdpFlags::SKB_MODE;
xdp_program.attach("eth0", flags)?;
```

## 性能

预期性能特征：

- **包过滤延迟**: 每包 < 100ns
- **内存开销**: 1024 个封禁 IP < 1MB
- **CPU 开销**: 10Gbps 吞吐量下 < 1%

## 安全注意事项

1. **需要 root**: 加载 eBPF 程序需要 root 或 `CAP_BPF`
2. **程序验证**: 所有 eBPF 程序由内核验证
3. **映射访问控制**: 只有加载进程可以访问 eBPF 映射
4. **审计日志**: 所有封禁/解封操作都被记录

## 参考资料

- [Aya 文档](https://aya-rs.dev/)
- [Linux eBPF 文档](https://ebpf.io/)
- [XDP 文档](https://github.com/iovisor/bpf-docs/blob/master/eBPF_Introduction.rst)

## 许可证

与 SMA-OS 项目许可证相同。

---
---

<a name="english"></a>
## English

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
