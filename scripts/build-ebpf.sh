#!/bin/bash
# Build script for SMA-OS eBPF and Rust components

set -e

echo "===================================="
echo "Building SMA-OS eBPF and Components"
echo "===================================="

# Check if running in Docker
if [ ! -f /.dockerenv ]; then
    echo "Not running in Docker container."
    echo "Please run this script inside the Rust Docker container:"
    echo ""
    echo "docker run --rm -v '$(pwd):/workspace' -w /workspace rust:latest bash build-ebpf.sh"
    echo ""
    exit 1
fi

echo "Installing dependencies..."
apt-get update -qq
apt-get install -y -qq protobuf-compiler clang llvm lld libelf-dev libbpf-dev

echo ""
echo "Installing Rust toolchain..."
rustup toolchain install nightly --component rust-src 2>/dev/null || true

echo ""
echo "Installing bpf-linker..."
cargo install bpf-linker 2>/dev/null || true

echo ""
echo "Building eBPF program..."
cd control-plane
cargo +nightly build -p fractal-gateway-ebpf \
    --target bpfel-unknown-none \
    -Z build-std=core \
    --release

echo ""
echo "Building userspace programs..."
cargo build --release -p state-engine -p teardown-ctrl -p fractal-gateway -p identity

echo ""
echo "===================================="
echo "Build Complete!"
echo "===================================="
echo ""
echo "eBPF binary:"
ls -lh target/bpfel-unknown-none/release/fractal-gateway-ebpf
echo ""
echo "Userspace binaries:"
ls -lh target/release/{state-engine,teardown-ctrl,fractal-gateway} 2>/dev/null || true
