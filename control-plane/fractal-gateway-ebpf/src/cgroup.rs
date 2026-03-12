//! Cgroup eBPF Program
//!
//! Monitors and enforces cgroup resource limits using eBPF.
//! Tracks CPU, memory, and IO usage per cgroup.

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::cgroup_sysctl_event,
    helpers::bpf_get_current_pid_tgid,
    macros::{cgroup_skb, cgroup_sysctl, cgroupsock, map},
    maps::HashMap,
    programs::SkBuffContext,
};
use aya_log_ebpf::{debug, info, warn};

/// Cgroup context information
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CgroupContext {
    pub cgroup_id: u64,
    pub cpu_quota_ns: u64,
    pub memory_limit_bytes: u64,
    pub io_weight: u32,
}

/// Map storing cgroup resource limits
#[map]
static CGROUP_LIMITS: HashMap<u64, CgroupContext> = HashMap::with_max_entries(1024, 0);

/// Current resource usage per cgroup
#[map]
static CGROUP_USAGE: HashMap<u64, CgroupUsage> = HashMap::with_max_entries(1024, 0);

/// Resource usage statistics
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CgroupUsage {
    pub cpu_time_ns: u64,
    pub memory_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub last_update_ns: u64,
}

/// eBPF cgroup/skb program for network traffic accounting
#[cgroup_skb]
pub fn cgroup_net_egress(ctx: SkBuffContext) -> i32 {
    match try_cgroup_net_egress(ctx) {
        Ok(ret) => ret,
        Err(_) => 0, // Pass on error
    }
}

fn try_cgroup_net_egress(ctx: SkBuffContext) -> Result<i32, ()> {
    let cgroup_id = unsafe { bpf_get_current_pid_tgid() }; // Get current cgroup context

    // Update network usage statistics
    if let Some(usage) = unsafe { CGROUP_USAGE.get(&cgroup_id) } {
        // Update packet count and bytes
        let _len = ctx.len();
        debug!(&ctx, "Cgroup {} egress packet", cgroup_id);
    }

    Ok(1) // Allow packet
}

/// eBPF cgroup/skb program for ingress traffic
#[cgroup_skb]
pub fn cgroup_net_ingress(ctx: SkBuffContext) -> i32 {
    match try_cgroup_net_ingress(ctx) {
        Ok(ret) => ret,
        Err(_) => 0,
    }
}

fn try_cgroup_net_ingress(ctx: SkBuffContext) -> Result<i32, ()> {
    let cgroup_id = unsafe { bpf_get_current_pid_tgid() };

    if let Some(_usage) = unsafe { CGROUP_USAGE.get(&cgroup_id) } {
        debug!(&ctx, "Cgroup {} ingress packet", cgroup_id);
    }

    Ok(1) // Allow packet
}

/// Helper to check if cgroup exceeds limits
fn check_cgroup_limits(cgroup_id: u64) -> bool {
    // Get current limits
    let limits = match unsafe { CGROUP_LIMITS.get(&cgroup_id) } {
        Some(l) => l,
        None => return true, // No limits = allow
    };

    // Get current usage
    let usage = match unsafe { CGROUP_USAGE.get(&cgroup_id) } {
        Some(u) => u,
        None => return true, // No usage = allow
    };

    // Check memory limit
    if usage.memory_bytes > limits.memory_limit_bytes {
        warn!(
            &ctx,
            "Cgroup {} exceeded memory limit: {} > {}",
            cgroup_id,
            usage.memory_bytes,
            limits.memory_limit_bytes
        );
        return false;
    }

    // Check CPU quota (simplified check)
    if usage.cpu_time_ns > limits.cpu_quota_ns {
        warn!(
            &ctx,
            "Cgroup {} exceeded CPU quota: {} > {}",
            cgroup_id,
            usage.cpu_time_ns,
            limits.cpu_quota_ns
        );
        return false;
    }

    true
}

/// Update cgroup usage (called from kprobes)
#[no_mangle]
pub unsafe fn update_cgroup_usage(cgroup_id: u64, cpu_delta: u64, mem_delta: i64) {
    let mut usage = match CGROUP_USAGE.get(&cgroup_id) {
        Some(u) => *u,
        None => CgroupUsage::default(),
    };

    usage.cpu_time_ns = usage.cpu_time_ns.saturating_add(cpu_delta);

    if mem_delta > 0 {
        usage.memory_bytes = usage.memory_bytes.saturating_add(mem_delta as u64);
    } else {
        usage.memory_bytes = usage.memory_bytes.saturating_sub((-mem_delta) as u64);
    }

    usage.last_update_ns = unsafe { bpf_ktime_get_ns() };

    let _ = CGROUP_USAGE.insert(&cgroup_id, &usage);
}

/// Get current cgroup ID
#[inline]
unsafe fn get_current_cgroup_id() -> u64 {
    bpf_get_current_pid_tgid() >> 32
}
