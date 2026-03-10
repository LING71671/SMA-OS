use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Get the target directory
    let target_dir = if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        out_dir.join("../../..")
    };

    // Build the eBPF program
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let status = Command::new("cargo")
        .args(&[
            "build-bpf",
            "--release",
            "--target-dir",
            &target_dir.to_string_lossy(),
        ])
        .status()
        .expect("Failed to execute cargo build-bpf");

    if !status.success() {
        panic!("eBPF build failed");
    }

    // Copy the built eBPF program to the output directory
    let ebpf_binary = target_dir
        .join("bpf-unknown-none-elf/release/fractal-gateway-ebpf")
        .canonicalize()
        .unwrap_or_else(|_| {
            // Fallback for different build configurations
            target_dir.join("bpf-unknown-none-elf/release/fractal-gateway-ebpf")
        });

    if ebpf_binary.exists() {
        let dest = out_dir.join("fractal-gateway-ebpf");
        std::fs::copy(&ebpf_binary, &dest)
            .unwrap_or_else(|e| panic!("Failed to copy eBPF binary to {}: {}", dest.display(), e));
        println!("cargo:warning=eBPF binary copied to {}", dest.display());
    } else {
        println!(
            "cargo:warning=eBPF binary not found at {}",
            ebpf_binary.display()
        );
    }
}
