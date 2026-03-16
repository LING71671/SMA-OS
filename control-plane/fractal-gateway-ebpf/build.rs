use std::env;
use std::path::PathBuf;

fn main() {
    // Get the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Rerun if eBPF source changes
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // The eBPF binary is built separately via: cargo xtask build-ebpf --release
    // This build script just sets up the paths

    let target_dir = if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        out_dir.join("../../..")
    };

    // Check for the eBPF binary built by xtask
    let ebpf_binary = target_dir.join("bpfel-unknown-none/release/fractal-gateway-ebpf");

    if ebpf_binary.exists() {
        println!(
            "cargo:warning=eBPF binary found at {}",
            ebpf_binary.display()
        );
    } else {
        println!(
            "cargo:warning=eBPF binary not found at {}. Run: cargo xtask build-ebpf --release",
            ebpf_binary.display()
        );
    }
}
