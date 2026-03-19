# Minimal Linux Images for Firecracker / Firecracker 最小化 Linux 镜像

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

本目录包含用于获取和验证适用于 SMA-OS 执行层 Firecracker microVM 的最小化 Linux 镜像的文档。

## 概述

Firecracker microVM 需要：
1. **Linux 内核镜像** (vmlinux): 不含模块的编译内核
2. **根文件系统** (rootfs.ext4): 包含基本工具的最小化文件系统

## 预构建镜像

### Alpine Linux（推荐）

**优势：**
- 最小占用空间（压缩后约 200 MB，解压后约 500 MB）
- 适合容器化工作负载
- 最小依赖
- 快速启动时间

**下载：**

```bash
# 创建镜像目录
mkdir -p /opt/firecracker/images
cd /opt/firecracker/images

# 下载 Alpine 最小化 rootfs
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.0-x86_64.tar.gz
SHA256SUM="7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db"
echo "${SHA256SUM} alpine-minirootfs-3.19.0-x86_64.tar.gz" | sha256sum -c -

# 解压并创建 ext4 文件系统
tar xzf alpine-minirootfs-3.19.0-x86_64.tar.gz -C /tmp/alpine_root/
dd if=/dev/zero of=rootfs.ext4 bs=1M count=512
mkfs.ext4 -F rootfs.ext4
sudo mount -o loop rootfs.ext4 /mnt/rootfs
sudo cp -r /tmp/alpine_root/* /mnt/rootfs/
sudo umount /mnt/rootfs
```

**校验和验证：**
```bash
# SHA256 哈希值
7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db alpine-minirootfs-3.19.0-x86_64.tar.gz
```

### Debian 最小化

**优势：**
- 标准软件包仓库
- 更好的工具生态
- 适合复杂工作负载
- 良好的文档

**下载：**

```bash
cd /opt/firecracker/images

# 使用 debootstrap
sudo debootstrap --arch=amd64 --variant=minbase bookworm /tmp/debian_root

# 创建 ext4 文件系统
dd if=/dev/zero of=rootfs.ext4 bs=1M count=1024
mkfs.ext4 -F rootfs.ext4
sudo mount -o loop rootfs.ext4 /mnt/rootfs
sudo cp -r /tmp/debian_root/* /mnt/rootfs/
sudo umount /mnt/rootfs
```

**包大小：** 约 400 MB（根文件系统）

### 自定义最小化镜像

**构建带特定软件包的自定义 Alpine：**

```bash
#!/bin/bash
# build_minimal_image.sh

set -e

WORK_DIR="/tmp/fc-image-build"
OUTPUT_DIR="/opt/firecracker/images"
ALPINE_VER="3.19"

mkdir -p ${WORK_DIR}
cd ${WORK_DIR}

# 下载 Alpine
curl -O https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VER}/releases/x86_64/alpine-minirootfs-${ALPINE_VER}.0-x86_64.tar.gz
tar xzf alpine-minirootfs-${ALPINE_VER}.0-x86_64.tar.gz

# 添加软件包
cat > etc/apk/repositories << 'EOF'
https://dl-cdn.alpinelinux.org/alpine/v3.19/main
https://dl-cdn.alpinelinux.org/alpine/v3.19/community
EOF

# 创建根文件系统镜像
dd if=/dev/zero of=${OUTPUT_DIR}/rootfs.ext4 bs=1M count=512
mkfs.ext4 -F ${OUTPUT_DIR}/rootfs.ext4
mount -o loop ${OUTPUT_DIR}/rootfs.ext4 /mnt/rootfs
cp -r . /mnt/rootfs/
umount /mnt/rootfs

echo "镜像已创建于 ${OUTPUT_DIR}/rootfs.ext4"
```

## Linux 内核镜像

### 预构建内核

**Firecracker 推荐内核（Linux 6.x）：**

```bash
cd /opt/firecracker/images

# 下载预构建内核
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin
mv vmlinux.bin vmlinux

# 下载校验和
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin.sha256
cat vmlinux.bin.sha256
# 预期: abc123...def456（参见官方发布）
```

### 自定义内核编译

**为 Firecracker 构建内核（Ubuntu/Debian）：**

```bash
#!/bin/bash
# build_kernel.sh

set -e

KERNEL_VER="6.1.46"
WORK_DIR="/tmp/kernel-build"

mkdir -p ${WORK_DIR}
cd ${WORK_DIR}

# 下载内核源码
wget https://kernel.org/pub/linux/kernel/v6.x/linux-${KERNEL_VER}.tar.xz
tar xf linux-${KERNEL_VER}.tar.xz
cd linux-${KERNEL_VER}

# 使用 Firecracker 推荐配置
curl -O https://raw.githubusercontent.com/firecracker-microvm/firecracker/main/resources/linux_config/.config

# 构建内核
make -j$(nproc) vmlinux

# 复制到镜像目录
cp vmlinux /opt/firecracker/images/vmlinux
chmod 644 /opt/firecracker/images/vmlinux

echo "内核已构建于 /opt/firecracker/images/vmlinux"
```

**内核要求：**
- 架构：x86_64
- 最低版本：Linux 4.20+（推荐 5.10+）
- 功能：KVM、virtio、ext4 文件系统支持
- 大小：约 10-15 MB（压缩后）

### 内核配置标志

Firecracker 必需的内核标志：

```bash
# 来自 Linux 源码 .config
CONFIG_VIRTIO=y
CONFIG_VIRTIO_BLK=y
CONFIG_VIRTIO_NET=y
CONFIG_VIRTIO_CONSOLE=y
CONFIG_EXT4_FS=y
CONFIG_SERIAL_8250=y
CONFIG_SERIAL_8250_CONSOLE=y
CONFIG_PRINTK=y
CONFIG_KVM_GUEST=y

# 性能可选
CONFIG_CPU_FREQ=y
CONFIG_CPU_IDLE=y
CONFIG_ACPI=y
```

## 镜像验证

### SHA256 校验和验证

```bash
# 为镜像生成校验和
sha256sum /opt/firecracker/images/rootfs.ext4
sha256sum /opt/firecracker/images/vmlinux

# 示例验证
echo "7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db rootfs.ext4" | sha256sum -c -

# 输出应显示：rootfs.ext4: OK
```

### 文件系统完整性

```bash
# 检查文件系统一致性
fsck.ext4 -n /opt/firecracker/images/rootfs.ext4

# 应报告：0 errors found
```

### 内核兼容性

```bash
# 验证内核与 Firecracker 兼容性
file /opt/firecracker/images/vmlinux
# 输出：ELF 64-bit LSB executable, x86-64, ...

# 检查内核版本
strings /opt/firecracker/images/vmlinux | grep "Linux version" | head -1
# 输出：Linux version 6.1.46 (buildhost@domain) ...
```

## 目录结构

```
/opt/firecracker/images/
├── vmlinux           # Linux 内核镜像（约 10-15 MB）
├── rootfs.ext4       # 根文件系统（约 300-500 MB）
├── checksums.txt     # SHA256 校验和用于验证
└── build_logs/
    ├── kernel_build.log
    └── filesystem_build.log
```

## 安装步骤

### 1. 创建目录结构

```bash
sudo mkdir -p /opt/firecracker/images
sudo chmod 755 /opt/firecracker/images
```

### 2. 下载预构建镜像

```bash
cd /opt/firecracker/images

# Alpine 内核（示例）
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin -O vmlinux

# Alpine rootfs（示例）
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.0-x86_64.tar.gz
tar xzf alpine-minirootfs-3.19.0-x86_64.tar.gz -C .
```

### 3. 验证完整性

```bash
# 创建校验和文件
sha256sum vmlinux rootfs.ext4 > checksums.txt

# 验证
sha256sum -c checksums.txt
```

### 4. 设置权限

```bash
# 使 VM 服务可读
sudo chown root:sma-os /opt/firecracker/images/*
sudo chmod 640 /opt/firecracker/images/{vmlinux,rootfs.ext4}
```

## 安全注意事项

### 1. 镜像完整性
- 下载后始终验证校验和
- 将校验和存储在单独位置
- 定期审计镜像以检测未授权修改

### 2. 最小化镜像理念
- 仅包含必要的二进制文件
- 移除不必要的软件包
- 减少攻击面

### 3. 只读文件系统
对于无状态 VM，考虑：

```bash
# 以只读方式挂载 rootfs
"drives": [
  {
    "drive_id": "rootfs",
    "path_on_host": "/opt/firecracker/images/rootfs.ext4",
    "is_read_only": true,
    "is_root_device": true
  }
]
```

### 4. 镜像缓存
- 在本地缓存已验证的镜像
- 上游更新时失效缓存
- 明确跟踪镜像版本

## 性能优化

### 镜像大小缩减

```bash
# 移除不必要的文件
mount -o loop rootfs.ext4 /mnt/rootfs
sudo rm -rf /mnt/rootfs/usr/share/doc/*
sudo rm -rf /mnt/rootfs/usr/share/man/*
sudo rm -rf /mnt/rootfs/var/cache/apk/*
sudo umount /mnt/rootfs

# 重新打包文件系统
e2fsck -f rootfs.ext4
resize2fs -M rootfs.ext4
```

### 启动时间优化

```json
{
  "boot_source": {
    "kernel_image_path": "/opt/firecracker/images/vmlinux",
    "boot_args": "console=ttyS0 reboot=k panic=1 pci=off nofb nomodules"
  }
}
```

## 故障排查

### 镜像挂载失败

```
Error: Could not mount /dev/vda on /
```

**解决方案：**
- 验证 rootfs.ext4 是有效的 ext4 文件系统
- 检查内核已启用 EXT4_FS
- 确保配置中 rootfs 路径正确

### 校验和不匹配

```
sha256sum: WARNING: 1 computed checksum did NOT match
```

**解决方案：**
- 重新下载镜像（可能已损坏）
- 验证下载 URL 是官方地址
- 检查传输期间的磁盘空间

### 启动时间慢

```
Time to boot VM: >10 seconds
```

**解决方案：**
- 使用 Alpine Linux（比 Debian 更快）
- 减小 rootfs 大小（更少文件需要挂载）
- 启用内核优化标志
- 检查磁盘 I/O 性能

## 参考资料

- [Alpine Linux 下载](https://alpinelinux.org/downloads/)
- [Firecracker 快速入门](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md)
- [Linux 内核构建指南](https://www.kernel.org/)
- [Buildroot 最小化镜像](https://buildroot.org/)

## 许可证

SMA-OS 执行层 - 保留所有权利

---
---

<a name="english"></a>
## English

This directory contains documentation for obtaining and validating minimal Linux images suitable for Firecracker microVMs in the SMA-OS execution layer.

## Overview

Firecracker microVMs require:
1. **Linux kernel image** (vmlinux): Compiled kernel without modules
2. **Root filesystem** (rootfs.ext4): Minimal filesystem with essential utilities

## Pre-built Images

### Alpine Linux (Recommended)

**Advantages:**
- Smallest footprint (~200 MB compressed, ~500 MB extracted)
- Excellent for containerized workloads
- Minimal dependencies
- Fast boot time

**Download:**

```bash
# Create images directory
mkdir -p /opt/firecracker/images
cd /opt/firecracker/images

# Download Alpine minimal rootfs
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.0-x86_64.tar.gz
SHA256SUM="7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db"
echo "${SHA256SUM} alpine-minirootfs-3.19.0-x86_64.tar.gz" | sha256sum -c -

# Extract and create ext4 filesystem
tar xzf alpine-minirootfs-3.19.0-x86_64.tar.gz -C /tmp/alpine_root/
dd if=/dev/zero of=rootfs.ext4 bs=1M count=512
mkfs.ext4 -F rootfs.ext4
sudo mount -o loop rootfs.ext4 /mnt/rootfs
sudo cp -r /tmp/alpine_root/* /mnt/rootfs/
sudo umount /mnt/rootfs
```

**Checksum Verification:**
```bash
# SHA256 Hash
7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db alpine-minirootfs-3.19.0-x86_64.tar.gz
```

### Debian Minimal

**Advantages:**
- Standard package repository
- Better tool ecosystem
- Suitable for complex workloads
- Good documentation

**Download:**

```bash
cd /opt/firecracker/images

# Using debootstrap
sudo debootstrap --arch=amd64 --variant=minbase bookworm /tmp/debian_root

# Create ext4 filesystem
dd if=/dev/zero of=rootfs.ext4 bs=1M count=1024
mkfs.ext4 -F rootfs.ext4
sudo mount -o loop rootfs.ext4 /mnt/rootfs
sudo cp -r /tmp/debian_root/* /mnt/rootfs/
sudo umount /mnt/rootfs
```

**Package Size:** ~400 MB (root filesystem)

### Custom Minimal Image

**Build Custom Alpine with Specific Packages:**

```bash
#!/bin/bash
# build_minimal_image.sh

set -e

WORK_DIR="/tmp/fc-image-build"
OUTPUT_DIR="/opt/firecracker/images"
ALPINE_VER="3.19"

mkdir -p ${WORK_DIR}
cd ${WORK_DIR}

# Download Alpine
curl -O https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VER}/releases/x86_64/alpine-minirootfs-${ALPINE_VER}.0-x86_64.tar.gz
tar xzf alpine-minirootfs-${ALPINE_VER}.0-x86_64.tar.gz

# Add packages
cat > etc/apk/repositories << 'EOF'
https://dl-cdn.alpinelinux.org/alpine/v3.19/main
https://dl-cdn.alpinelinux.org/alpine/v3.19/community
EOF

# Create root filesystem image
dd if=/dev/zero of=${OUTPUT_DIR}/rootfs.ext4 bs=1M count=512
mkfs.ext4 -F ${OUTPUT_DIR}/rootfs.ext4
mount -o loop ${OUTPUT_DIR}/rootfs.ext4 /mnt/rootfs
cp -r . /mnt/rootfs/
umount /mnt/rootfs

echo "Image created at ${OUTPUT_DIR}/rootfs.ext4"
```

## Linux Kernel Images

### Pre-built Kernels

**Firecracker Recommended Kernel (Linux 6.x):**

```bash
cd /opt/firecracker/images

# Download pre-built kernel
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin
mv vmlinux.bin vmlinux

# Download checksum
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin.sha256
cat vmlinux.bin.sha256
# Expected: abc123...def456 (see official release)
```

### Custom Kernel Compilation

**Build Kernel for Firecracker (Ubuntu/Debian):**

```bash
#!/bin/bash
# build_kernel.sh

set -e

KERNEL_VER="6.1.46"
WORK_DIR="/tmp/kernel-build"

mkdir -p ${WORK_DIR}
cd ${WORK_DIR}

# Download kernel source
wget https://kernel.org/pub/linux/kernel/v6.x/linux-${KERNEL_VER}.tar.xz
tar xf linux-${KERNEL_VER}.tar.xz
cd linux-${KERNEL_VER}

# Use Firecracker recommended config
curl -O https://raw.githubusercontent.com/firecracker-microvm/firecracker/main/resources/linux_config/.config

# Build kernel
make -j$(nproc) vmlinux

# Copy to images directory
cp vmlinux /opt/firecracker/images/vmlinux
chmod 644 /opt/firecracker/images/vmlinux

echo "Kernel built at /opt/firecracker/images/vmlinux"
```

**Kernel Requirements:**
- Architecture: x86_64
- Minimum version: Linux 4.20+ (recommended 5.10+)
- Features: KVM, virtio, ext4 filesystem support
- Size: ~10-15 MB (compressed)

### Kernel Configuration Flags

Essential kernel flags for Firecracker:

```bash
# From Linux source .config
CONFIG_VIRTIO=y
CONFIG_VIRTIO_BLK=y
CONFIG_VIRTIO_NET=y
CONFIG_VIRTIO_CONSOLE=y
CONFIG_EXT4_FS=y
CONFIG_SERIAL_8250=y
CONFIG_SERIAL_8250_CONSOLE=y
CONFIG_PRINTK=y
CONFIG_KVM_GUEST=y

# Optional for performance
CONFIG_CPU_FREQ=y
CONFIG_CPU_IDLE=y
CONFIG_ACPI=y
```

## Image Verification

### SHA256 Checksum Validation

```bash
# Generate checksum for your image
sha256sum /opt/firecracker/images/rootfs.ext4
sha256sum /opt/firecracker/images/vmlinux

# Example verification
echo "7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db rootfs.ext4" | sha256sum -c -

# Output should show: rootfs.ext4: OK
```

### Filesystem Integrity

```bash
# Check filesystem consistency
fsck.ext4 -n /opt/firecracker/images/rootfs.ext4

# Should report: 0 errors found
```

### Kernel Compatibility

```bash
# Verify kernel for Firecracker compatibility
file /opt/firecracker/images/vmlinux
# Output: ELF 64-bit LSB executable, x86-64, ...

# Check kernel version
strings /opt/firecracker/images/vmlinux | grep "Linux version" | head -1
# Output: Linux version 6.1.46 (buildhost@domain) ...
```

## Directory Structure

```
/opt/firecracker/images/
├── vmlinux           # Linux kernel image (~10-15 MB)
├── rootfs.ext4       # Root filesystem (~300-500 MB)
├── checksums.txt     # SHA256 checksums for validation
└── build_logs/
    ├── kernel_build.log
    └── filesystem_build.log
```

## Installation Steps

### 1. Create Directory Structure

```bash
sudo mkdir -p /opt/firecracker/images
sudo chmod 755 /opt/firecracker/images
```

### 2. Download Pre-built Images

```bash
cd /opt/firecracker/images

# Alpine kernel (example)
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.4.0/vmlinux.bin -O vmlinux

# Alpine rootfs (example)
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.0-x86_64.tar.gz
tar xzf alpine-minirootfs-3.19.0-x86_64.tar.gz -C .
```

### 3. Verify Integrity

```bash
# Create checksums file
sha256sum vmlinux rootfs.ext4 > checksums.txt

# Verify
sha256sum -c checksums.txt
```

### 4. Set Permissions

```bash
# Make readable by VM service
sudo chown root:sma-os /opt/firecracker/images/*
sudo chmod 640 /opt/firecracker/images/{vmlinux,rootfs.ext4}
```

## Security Considerations

### 1. Image Integrity
- Always verify checksums after download
- Store checksums in separate location
- Periodically audit images for unauthorized modifications

### 2. Minimal Image Philosophy
- Include only necessary binaries
- Remove unnecessary packages
- Reduce attack surface

### 3. Read-Only Filesystems
For stateless VMs, consider:

```bash
# Mount rootfs as read-only
"drives": [
  {
    "drive_id": "rootfs",
    "path_on_host": "/opt/firecracker/images/rootfs.ext4",
    "is_read_only": true,
    "is_root_device": true
  }
]
```

### 4. Image Caching
- Cache verified images locally
- Invalidate on upstream updates
- Track image versions explicitly

## Performance Optimization

### Image Size Reduction

```bash
# Remove unnecessary files
mount -o loop rootfs.ext4 /mnt/rootfs
sudo rm -rf /mnt/rootfs/usr/share/doc/*
sudo rm -rf /mnt/rootfs/usr/share/man/*
sudo rm -rf /mnt/rootfs/var/cache/apk/*
sudo umount /mnt/rootfs

# Repack filesystem
e2fsck -f rootfs.ext4
resize2fs -M rootfs.ext4
```

### Boot Time Optimization

```json
{
  "boot_source": {
    "kernel_image_path": "/opt/firecracker/images/vmlinux",
    "boot_args": "console=ttyS0 reboot=k panic=1 pci=off nofb nomodules"
  }
}
```

## Troubleshooting

### Image Mount Failures

```
Error: Could not mount /dev/vda on /
```

**Solution:**
- Verify rootfs.ext4 is valid ext4 filesystem
- Check kernel has EXT4_FS enabled
- Ensure correct rootfs path in config

### Checksum Mismatch

```
sha256sum: WARNING: 1 computed checksum did NOT match
```

**Solution:**
- Re-download image (possible corruption)
- Verify download URL is official
- Check disk space during transfer

### Slow Boot Time

```
Time to boot VM: >10 seconds
```

**Solution:**
- Use Alpine Linux (faster than Debian)
- Reduce rootfs size (fewer files to mount)
- Enable kernel optimization flags
- Check disk I/O performance

## References

- [Alpine Linux Downloads](https://alpinelinux.org/downloads/)
- [Firecracker Getting Started](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md)
- [Linux Kernel Build Guide](https://www.kernel.org/)
- [Buildroot for Minimal Images](https://buildroot.org/)

## License

SMA-OS Execution Layer - All rights reserved
