//! xtask - Build automation for eBPF programs
//!
//! Usage: cargo xtask build-ebpf

use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for SMA-OS eBPF programs")]
enum Cli {
    /// Build the eBPF program
    BuildEbpf {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::BuildEbpf { release } => build_ebpf(release),
    }
}

fn build_ebpf(release: bool) -> Result<()> {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();

    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let target_dir = workspace_root.join("target");

    println!("Building eBPF program...");
    println!("Workspace: {}", workspace_root.display());
    println!("Target dir: {}", target_dir.display());

    // Build the eBPF program using cargo with bpf target
    let mut args = vec![
        "build".to_string(),
        "--package".to_string(),
        "fractal-gateway-ebpf".to_string(),
        "--target".to_string(),
        "bpfel-unknown-none".to_string(),
        "-Z".to_string(),
        "build-std=core".to_string(),
    ];

    if release {
        args.push("--release".to_string());
    }

    let status = Command::new("cargo")
        .args(&args)
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(workspace_root)
        .status()
        .expect("Failed to execute cargo build for eBPF");

    if !status.success() {
        anyhow::bail!("eBPF build failed");
    }

    // Copy the built binary
    let profile = if release { "release" } else { "debug" };
    let ebpf_binary = target_dir
        .join("bpfel-unknown-none")
        .join(profile)
        .join("fractal-gateway-ebpf");

    if ebpf_binary.exists() {
        println!("eBPF binary built successfully: {}", ebpf_binary.display());
    } else {
        anyhow::bail!("eBPF binary not found at {}", ebpf_binary.display());
    }

    Ok(())
}
