# Firecracker Configuration Guide / Firecracker 配置指南

[中文](./README.md) | [English](./README_ZH.md)

---

本目录包含 SMA-OS 执行层中 Firecracker microVM 管理的配置模板和规范。

## 配置文件

### 1. `firecracker-config.json`

主要 Firecracker VM 配置文件。指定：
- **IPC 服务器**: API 通信的 Unix 域套接字路径
- **启动源**: 内核镜像路径和启动参数
- **驱动器**: 根文件系统和额外存储设备
- **网络接口**: 客户机网络的 TAP 设备
- **机器配置**: vCPU 数量、内存、CPU 模板
- **Vsock**: 主机-客户机通信的虚拟套接字配置
- **日志**: 日志文件路径和级别
- **指标**: 指标收集配置

**关键字段：**
```json
{
  "ipc_server_cfg": {
    "path": "/tmp/firecracker.socket",
    "bind_mount": false
  },
  "machine_cfg": {
    "vcpu_count": 2,
    "memory_size_mib": 256,
    "cpu_template": "C3",
    "ht_enabled": false,
    "smt": false
  }
}
```

**默认值：**
- **vCPU**: 2（可通过 vm-specs.json 配置）
- **内存**: 256 MiB（可通过 vm-specs.json 配置）
- **CPU 模板**: C3（Intel 架构基准）
- **启动参数**: `console=ttyS0 reboot=k panic=1 pci=off`

### 2. `vm-specs.json`

VM 大小规格和资源模板。

**可用预设：**
- **small**: 1 vCPU, 128 MiB RAM（轻量级工作负载）
- **medium**: 2 vCPU, 512 MiB RAM（标准工作负载，默认）
- **large**: 4 vCPU, 2 GiB RAM（重计算）

**资源限制：**
- CPU 份额（相对优先级）
- 内存限制（cgroup 强制）
- 磁盘配额（文件系统约束）

**用法：**
```json
{
  "small": {
    "vcpu_count": 1,
    "memory_size_mib": 128,
    "resource_limits": {
      "cpu_shares": 256,
      "memory_limit": "128M",
      "disk_quota": "1G"
    }
  }
}
```

### 3. `network-config.json`

VM 的网络和防火墙配置。

**功能：**
- **接口配置**: MAC 地址、IP、网关、DHCP 支持
- **防火墙规则**: 入站/出站规则，支持协议和端口过滤
- **NAT 规则**: 从主机到客户机的端口转发
- **套接字配置**: API、Vsock 和指标套接字路径

**默认网络设置：**
- **eth0**: 192.168.1.0/24（主网络）
- **eth1**: 10.0.0.0/24（辅助网络）
- **网关**: 192.168.1.1
- **DNS**: 8.8.8.8, 1.1.1.1

**防火墙规则示例：**
```json
{
  "direction": "inbound",
  "protocol": "tcp",
  "port": 22,
  "action": "allow",
  "description": "Allow SSH"
}
```

## 使用指南

### 1. 在代码中加载配置

使用 `serde_json` 反序列化配置：

```rust
use std::fs;
use serde_json::Value;

// 加载主配置
let config_str = fs::read_to_string("configs/firecracker-config.json")?;
let config: Value = serde_json::from_str(&config_str)?;

// 访问字段
let socket_path = config["ipc_server_cfg"]["path"].as_str();
let vcpu_count = config["machine_cfg"]["vcpu_count"].as_i64();
```

### 2. 使用特定大小初始化 VM

```rust
// 加载 VM 规格
let specs_str = fs::read_to_string("configs/vm-specs.json")?;
let specs: Value = serde_json::from_str(&specs_str)?;

// 获取 "medium" 预设
let medium = &specs["medium"];
let vcpu_count = medium["vcpu_count"].as_i64().unwrap_or(2);
let memory_mib = medium["memory_size_mib"].as_i64().unwrap_or(512);
```

### 3. 应用网络规则

```rust
// 加载网络配置
let net_str = fs::read_to_string("configs/network-config.json")?;
let net_cfg: Value = serde_json::from_str(&net_str)?;

// 应用防火墙规则
for rule in net_cfg["firewall_rules"].as_array().unwrap_or(&vec![]) {
    let direction = rule["direction"].as_str();
    let protocol = rule["protocol"].as_str();
    let port = rule["port"].as_i64();
    // 将规则应用到 eBPF 或内核防火墙
}
```

## 套接字配置

### API 套接字路径
- **默认**: `/tmp/firecracker.socket`
- **用途**: 与 Firecracker 守护进程的 RESTful API 通信
- **协议**: Unix 域套接字 (SOCK_STREAM)
- **认证**: 文件权限 (chmod 0600)

### Vsock 配置
- **客户机 CID**: 3（可在 vm-specs.json 中配置基数）
- **用途**: 高性能主机-客户机通信
- **套接字路径**: `/tmp/firecracker-vsock.socket`
- **特性**: 面向流、低延迟

### 指标套接字
- **路径**: `/tmp/firecracker-metrics.json`
- **格式**: JSON 指标转储
- **内容**: vCPU 指标、内存统计、网络吞吐量

## 安全注意事项

### 1. 套接字权限
```bash
# 限制 API 套接字访问 sma-os 用户/组
chmod 0600 /tmp/firecracker.socket
chown sma-os:sma-os /tmp/firecracker.socket
```

### 2. 网络隔离
- 每个 VM 使用独立的 TAP 设备进行隔离
- 防火墙规则防止未授权访问
- Vsock 提供已认证的主机-客户机通道

### 3. CPU 模板选择
- **C3**: Intel 基准（最广泛兼容）
- **T2**: 更新的 Intel，功能更多（需要 Skylake+）
- **T2S**: 敏感指令过滤，适用于不可信代码

### 4. 资源限制
通过 cgroups 强制执行，防止：
- **CPU 饥饿**: CPU 份额限制
- **内存耗尽**: 内存限制
- **磁盘填满**: 磁盘配额

## 验证

### 检查配置语法
```bash
# 验证 JSON
jq . configs/firecracker-config.json
jq . configs/vm-specs.json
jq . configs/network-config.json
```

### 验证套接字路径
```bash
# 确保套接字目录存在
mkdir -p /tmp
ls -la /tmp/firecracker*.socket 2>/dev/null || echo "套接字尚未创建"
```

### 测试网络配置
```bash
# 测试 TAP 接口可用性
ip tap show
# 应显示 tap0, tap1 等
```

## 环境变量

运行时通过环境变量配置：

```bash
# 覆盖套接字路径
export FIRECRACKER_SOCKET_PATH="/var/run/firecracker.socket"
export FIRECRACKER_METRICS_PATH="/var/run/metrics.json"

# 设置资源预设
export VM_SIZE="large"

# 网络配置
export TAP_DEVICE="tap0"
export GUEST_IP="192.168.1.2"
```

## 常见修改

### 增加 VM 内存
编辑 `vm-specs.json`:
```json
"medium": {
  "memory_size_mib": 1024, // 从 512 改变
  ...
}
```

### 添加新网络接口
编辑 `network-config.json`:
```json
"interfaces": [
  ...现有接口...,
  {
    "iface_id": "eth2",
    "guest_mac": "02:00:00:00:00:03",
    "ip_address": "10.1.0.2",
    "subnet_mask": "255.255.255.0",
    "host_dev_name": "tap2"
  }
]
```

### 自定义启动参数
编辑 `firecracker-config.json`:
```json
"boot_source": {
  "kernel_image_path": "/opt/firecracker/images/vmlinux",
  "boot_args": "console=ttyS0 reboot=k panic=1 pci=off loglevel=debug"
}
```

## 故障排查

### 套接字未找到
```
Error: Cannot connect to /tmp/firecracker.socket
```
**解决方案**: 确保 Firecracker 守护进程正在运行且套接字路径与配置匹配。

### 网络接口不可用
```
Error: TAP device tap0 not found
```
**解决方案**: 预创建 TAP 设备：
```bash
ip tuntap add dev tap0 mode tap
ip link set dev tap0 up
```

### 内存分配失败
```
Error: Cannot allocate 2048 MiB
```
**解决方案**: 检查可用系统内存并在规格中调整 `memory_size_mib`。

### CPU 模板不匹配
```
Error: CPU template C3 not supported
```
**解决方案**: 使用 `lscpu` 检查主机 CPU 并选择兼容模板（如 T2 或 T2S）。

## 参考资料

- [Firecracker 官方文档](https://github.com/firecracker-microvm/firecracker)
- [Firecracker API 文档](https://github.com/firecracker-microvm/firecracker/tree/main/docs)
- [vCPU 和内存规格](https://docs.firecracker.dev)
- [网络配置指南](https://github.com/firecracker-microvm/firecracker/blob/main/docs/network-setup.md)

## 许可证

SMA-OS 执行层 - 保留所有权利
