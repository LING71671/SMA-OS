# Minimal Linux Images for Firecracker

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
echo "${SHA256SUM}  alpine-minirootfs-3.19.0-x86_64.tar.gz" | sha256sum -c -

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
7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db  alpine-minirootfs-3.19.0-x86_64.tar.gz
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
echo "7ce1fbfb9c4b89c5b0aebae88b35ebb82549e33e6f75b5e9e8c11eef9d19e1db  rootfs.ext4" | sha256sum -c -

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
├── vmlinux              # Linux kernel image (~10-15 MB)
├── rootfs.ext4          # Root filesystem (~300-500 MB)
├── checksums.txt        # SHA256 checksums for validation
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
