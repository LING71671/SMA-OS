# Firecracker Configuration Guide

This directory contains configuration templates and specifications for Firecracker microVM management in SMA-OS execution layer.

## Configuration Files

### 1. `firecracker-config.json`
Main Firecracker VM configuration file. Specifies:
- **IPC Server**: Unix domain socket path for API communication
- **Boot Source**: Kernel image path and boot arguments
- **Drives**: Root filesystem and additional storage devices
- **Network Interfaces**: TAP devices for guest networking
- **Machine Config**: vCPU count, memory, CPU template
- **Vsock**: Virtual socket configuration for host-guest communication
- **Logging**: Log file path and level
- **Metrics**: Metrics collection configuration

**Key Fields:**
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

**Default Values:**
- **vCPUs**: 2 (configurable via vm-specs.json)
- **Memory**: 256 MiB (configurable via vm-specs.json)
- **CPU Template**: C3 (Intel architecture baseline)
- **Boot Args**: `console=ttyS0 reboot=k panic=1 pci=off`

### 2. `vm-specs.json`
VM size specifications and resource templates.

**Available Presets:**
- **small**: 1 vCPU, 128 MiB RAM (lightweight workloads)
- **medium**: 2 vCPU, 512 MiB RAM (standard workloads, default)
- **large**: 4 vCPU, 2 GiB RAM (heavy computations)

**Resource Limits:**
- CPU shares (relative priority)
- Memory limits (cgroup enforcement)
- Disk quotas (filesystem constraints)

**Usage:**
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
Network and firewall configuration for VMs.

**Features:**
- **Interface Configuration**: MAC address, IP, gateway, DHCP support
- **Firewall Rules**: Inbound/outbound rules with protocol and port filtering
- **NAT Rules**: Port forwarding from host to guest
- **Socket Configuration**: API, Vsock, and metrics socket paths

**Default Network Setup:**
- **eth0**: 192.168.1.0/24 (primary network)
- **eth1**: 10.0.0.0/24 (secondary network)
- **Gateway**: 192.168.1.1
- **DNS**: 8.8.8.8, 1.1.1.1

**Firewall Rules Examples:**
```json
{
  "direction": "inbound",
  "protocol": "tcp",
  "port": 22,
  "action": "allow",
  "description": "Allow SSH"
}
```

## Usage Guide

### 1. Loading Configuration in Code

Use `serde_json` to deserialize configurations:

```rust
use std::fs;
use serde_json::Value;

// Load main config
let config_str = fs::read_to_string("configs/firecracker-config.json")?;
let config: Value = serde_json::from_str(&config_str)?;

// Access fields
let socket_path = config["ipc_server_cfg"]["path"].as_str();
let vcpu_count = config["machine_cfg"]["vcpu_count"].as_i64();
```

### 2. Initializing a VM with Specific Size

```rust
// Load VM specs
let specs_str = fs::read_to_string("configs/vm-specs.json")?;
let specs: Value = serde_json::from_str(&specs_str)?;

// Get "medium" preset
let medium = &specs["medium"];
let vcpu_count = medium["vcpu_count"].as_i64().unwrap_or(2);
let memory_mib = medium["memory_size_mib"].as_i64().unwrap_or(512);
```

### 3. Applying Network Rules

```rust
// Load network config
let net_str = fs::read_to_string("configs/network-config.json")?;
let net_cfg: Value = serde_json::from_str(&net_str)?;

// Apply firewall rules
for rule in net_cfg["firewall_rules"].as_array().unwrap_or(&vec![]) {
    let direction = rule["direction"].as_str();
    let protocol = rule["protocol"].as_str();
    let port = rule["port"].as_i64();
    // Apply rule to eBPF or kernel firewall
}
```

## Socket Configuration

### API Socket Path
- **Default**: `/tmp/firecracker.socket`
- **Purpose**: RESTful API communication with Firecracker daemon
- **Protocol**: Unix domain socket (SOCK_STREAM)
- **Auth**: File permissions (chmod 0600)

### Vsock Configuration
- **Guest CID**: 3 (configurable base in vm-specs.json)
- **Purpose**: High-performance host-guest communication
- **Socket Path**: `/tmp/firecracker-vsock.socket`
- **Features**: Stream-oriented, low-latency

### Metrics Socket
- **Path**: `/tmp/firecracker-metrics.json`
- **Format**: JSON metrics dump
- **Contents**: vCPU metrics, memory stats, network throughput

## Security Considerations

### 1. Socket Permissions
```bash
# Restrict API socket access to sma-os user/group
chmod 0600 /tmp/firecracker.socket
chown sma-os:sma-os /tmp/firecracker.socket
```

### 2. Network Isolation
- Use separate TAP devices per VM for isolation
- Firewall rules prevent unauthorized access
- Vsock provides authenticated host-guest channel

### 3. CPU Template Selection
- **C3**: Intel baseline (widest compatibility)
- **T2**: Newer Intel with more features (requires Skylake+)
- **T2S**: Sensitive instruction filtering for untrusted code

### 4. Resource Limits
Enforce via cgroups to prevent:
- **CPU starvation**: CPU shares limit
- **Memory exhaustion**: Memory limits
- **Disk fillup**: Disk quotas

## Validation

### Check Configuration Syntax
```bash
# Validate JSON
jq . configs/firecracker-config.json
jq . configs/vm-specs.json
jq . configs/network-config.json
```

### Verify Socket Paths
```bash
# Ensure socket directory exists
mkdir -p /tmp
ls -la /tmp/firecracker*.socket 2>/dev/null || echo "Sockets not created yet"
```

### Test Network Configuration
```bash
# Test TAP interface availability
ip tap show
# Should show tap0, tap1, etc.
```

## Environment Variables

Configure at runtime via environment:

```bash
# Override socket paths
export FIRECRACKER_SOCKET_PATH="/var/run/firecracker.socket"
export FIRECRACKER_METRICS_PATH="/var/run/metrics.json"

# Set resource preset
export VM_SIZE="large"

# Network configuration
export TAP_DEVICE="tap0"
export GUEST_IP="192.168.1.2"
```

## Common Modifications

### Increase VM Memory
Edit `vm-specs.json`:
```json
"medium": {
  "memory_size_mib": 1024,  // Changed from 512
  ...
}
```

### Add New Network Interface
Edit `network-config.json`:
```json
"interfaces": [
  ...existing interfaces...,
  {
    "iface_id": "eth2",
    "guest_mac": "02:00:00:00:00:03",
    "ip_address": "10.1.0.2",
    "subnet_mask": "255.255.255.0",
    "host_dev_name": "tap2"
  }
]
```

### Custom Boot Arguments
Edit `firecracker-config.json`:
```json
"boot_source": {
  "kernel_image_path": "/opt/firecracker/images/vmlinux",
  "boot_args": "console=ttyS0 reboot=k panic=1 pci=off loglevel=debug"
}
```

## Troubleshooting

### Socket Not Found
```
Error: Cannot connect to /tmp/firecracker.socket
```
**Solution**: Ensure Firecracker daemon is running and socket path matches config.

### Network Interface Not Available
```
Error: TAP device tap0 not found
```
**Solution**: Pre-create TAP devices:
```bash
ip tuntap add dev tap0 mode tap
ip link set dev tap0 up
```

### Memory Allocation Failed
```
Error: Cannot allocate 2048 MiB
```
**Solution**: Check available system memory and adjust `memory_size_mib` in specs.

### CPU Template Mismatch
```
Error: CPU template C3 not supported
```
**Solution**: Check host CPU with `lscpu` and select compatible template (e.g., T2 or T2S).

## References

- [Firecracker Official Docs](https://github.com/firecracker-microvm/firecracker)
- [Firecracker API Documentation](https://github.com/firecracker-microvm/firecracker/tree/main/docs)
- [vCPU and Memory Sizing](https://docs.firecracker.dev)
- [Network Configuration Guide](https://github.com/firecracker-microvm/firecracker/blob/main/docs/network-setup.md)

## License

SMA-OS Execution Layer - All rights reserved
