//! Fractal Gateway eBPF Loader
//!
//! This module provides functionality to load, manage, and unload
//! the eBPF XDP program for network filtering.

use anyhow::Context;
use anyhow::Result;
use aya::programs::Xdp;
use aya::programs::XdpFlags;
use aya::Ebpf;
use tracing::info;
use tracing::warn;

/// Fractal Gateway eBPF manager
pub struct FractalGatewayEbpf {
    ebpf: Ebpf,
}

impl FractalGatewayEbpf {
    /// Load the eBPF program from the embedded ELF file
    pub fn load() -> Result<Self> {
        info!("Loading Fractal Gateway eBPF program");

        let ebpf = Ebpf::load_elf(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/fractal-gateway-ebpf"
        )))
        .context("Failed to load eBPF ELF")?;

        Ok(Self { ebpf })
    }

    /// Attach the XDP program to the specified interface
    pub fn attach_xdp(&mut self, interface: &str) -> Result<()> {
        info!("Attaching XDP program to interface: {}", interface);

        let xdp_program: &mut Xdp = self
            .ebpf
            .get_mut("fractal_gateway")
            .and_then(|program| program.try_into())
            .context("Failed to get XDP program")?;

        // Attach with flags that allow fallback to other XDP modes
        let flags = XdpFlags::default();

        xdp_program
            .attach(interface, flags)
            .with_context(|| format!("Failed to attach XDP to interface {}", interface))?;

        info!("XDP program attached successfully");
        Ok(())
    }

    /// Detach the XDP program
    pub fn detach(&mut self) -> Result<()> {
        info!("Detaching XDP program");

        let xdp_program: &mut Xdp = self
            .ebpf
            .get_mut("fractal_gateway")
            .and_then(|program| program.try_into())
            .context("Failed to get XDP program")?;

        xdp_program
            .detach()
            .context("Failed to detach XDP program")?;

        info!("XDP program detached successfully");
        Ok(())
    }

    /// Add an IP to the blocked list
    pub fn block_ip(&mut self, ip: u32) -> Result<()> {
        let ip_str = u32_to_ip(ip);
        info!("Blocking IP: {} ({})", ip_str, ip);

        let blocked_ips = self
            .ebpf
            .get_mut("BLOCKED_IPS")
            .and_then(|map| map.try_into())
            .context("Failed to get BLOCKED_IPS map")?;

        blocked_ips.insert(ip, 1).context("Failed to insert IP")?;

        Ok(())
    }

    /// Remove an IP from the blocked list
    pub fn unblock_ip(&mut self, ip: u32) -> Result<()> {
        let ip_str = u32_to_ip(ip);
        info!("Unblocking IP: {} ({})", ip_str, ip);

        let blocked_ips = self
            .ebpf
            .get_mut("BLOCKED_IPS")
            .and_then(|map| map.try_into())
            .context("Failed to get BLOCKED_IPS map")?;

        blocked_ips.remove(&ip).context("Failed to remove IP")?;

        Ok(())
    }

    /// Get the count of blocked IPs
    pub fn get_blocked_count(&mut self) -> Result<usize> {
        let blocked_ips: aya::maps::HashMap<_, u32, u8> = self
            .ebpf
            .get_mut("BLOCKED_IPS")
            .and_then(|map| map.try_into())
            .context("Failed to get BLOCKED_IPS map")?;

        Ok(blocked_ips.iter().count())
    }
}

/// Helper function to convert IP string to u32
pub fn ip_to_u32(ip: &str) -> Result<u32> {
    let parts: Vec<u8> = ip
        .split('.')
        .map(|s| s.parse::<u8>())
        .collect::<Result<Vec<u8>, _>>()
        .context("Invalid IP format")?;

    if parts.len() != 4 {
        return Err(anyhow::anyhow!("Invalid IP address: {}", ip));
    }

    Ok(u32::from_be_bytes([parts[0], parts[1], parts[2], parts[3]]))
}

/// Helper function to convert u32 to IP string
pub fn u32_to_ip(ip: u32) -> String {
    let bytes = ip.to_be_bytes();
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_conversion() {
        let ip_str = "192.168.1.1";
        let ip_num = ip_to_u32(ip_str).unwrap();
        let ip_back = u32_to_ip(ip_num);
        assert_eq!(ip_back, ip_str);
    }

    #[test]
    fn test_invalid_ip() {
        assert!(ip_to_u32("256.1.1.1").is_err());
        assert!(ip_to_u32("1.1.1").is_err());
    }
}
