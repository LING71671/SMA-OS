//! Quota Enforcement eBPF Program
//!
//! Implements O(1) quota enforcement for reward/punishment system.
//! Uses eBPF maps to store dynamic quotas and enforces them
//! at syscall entry with nanosecond-level latency.

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::pt_regs,
    helpers::bpf_get_current_pid_tgid,
    macros::{kprobe, kretprobe, lsm, map},
    maps::HashMap,
};
use aya_log_ebpf::{debug, info, warn};

/// Process quota context - O(1) lookup
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessQuota {
    pub pid: u32,
    pub agent_id: u32,
    /// CPU quota multiplier (0.1 - 10.0, scaled to u16)
    pub cpu_multiplier: u16, // 100 = 1.0x
    /// Memory quota multiplier
    pub mem_multiplier: u16,
    /// Bandwidth quota multiplier  
    pub bw_multiplier: u16,
    /// Priority level (0-100)
    pub priority: u8,
    /// Current enforcement flags
    pub flags: u8,
}

/// O(1) quota map - keyed by PID
#[map]
static PROCESS_QUOTAS: HashMap<u32, ProcessQuota> = HashMap::with_max_entries(65536, 0);

/// Resource usage tracking per process
#[map]
static PROCESS_USAGE: HashMap<u32, ResourceUsage> = HashMap::with_max_entries(65536, 0);

/// Resource usage statistics
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ResourceUsage {
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub last_update_ns: u64,
}

/// Quota enforcement flags
const FLAG_THROTTLED: u8 = 0x01;
const FLAG_MEMORY_PRESSURE: u8 = 0x02;
const FLAG_CPU_THROTTLED: u8 = 0x04;

/// Syscall numbers to monitor
const SYS_CLONE: i64 = 56;
const SYS_FORK: i64 = 57;
const SYS_VFORK: i64 = 58;
const SYS_EXECVE: i64 = 59;
const SYS_EXIT: i64 = 60;
const SYS_EXIT_GROUP: i64 = 231;
const SYS_MMAP: i64 = 9;
const SYS_BRK: i64 = 12;

/// O(1) quota check at syscall entry
#[kprobe]
pub fn quota_syscall_entry(ctx: ProbeContext) -> u32 {
    match try_quota_syscall_entry(ctx) {
        Ok(ret) => ret,
        Err(_) => 0,
    }
}

fn try_quota_syscall_entry(ctx: ProbeContext) -> Result<u32, ()> {
    let pid = unsafe { bpf_get_current_pid_tgid() } as u32;

    // O(1) lookup - direct map access
    let quota = match unsafe { PROCESS_QUOTAS.get(&pid) } {
        Some(q) => q,
        None => return Ok(0), // No quota = allow
    };

    // Check if process is throttled
    if quota.flags & FLAG_THROTTLED != 0 {
        warn!(&ctx, "Process {} throttled, blocking syscall", pid);
        return Err(()); // Block syscall
    }

    // Get syscall number from pt_regs
    let syscall_nr = ctx.arg::<i64>(0).ok_or(())?;

    // Apply quota multipliers based on syscall type
    match syscall_nr {
        // Memory allocation syscalls
        SYS_MMAP | SYS_BRK => {
            if quota.flags & FLAG_MEMORY_PRESSURE != 0 {
                debug!(&ctx, "Process {} memory pressure, throttling mmap", pid);
                return Err(());
            }
        }

        // Process creation syscalls - check limits
        SYS_CLONE | SYS_FORK | SYS_VFORK => {
            if quota.priority > 80 {
                warn!(&ctx, "Process {} low priority, blocking fork", pid);
                return Err(());
            }
        }

        _ => {}
    }

    // O(1) decision - allow syscall
    Ok(0)
}

/// Update process quota - O(1) operation
#[no_mangle]
pub unsafe fn update_process_quota(
    pid: u32,
    agent_id: u32,
    cpu_mult: u16,
    mem_mult: u16,
    bw_mult: u16,
    priority: u8,
) -> i32 {
    let quota = ProcessQuota {
        pid,
        agent_id,
        cpu_multiplier: cpu_mult,
        mem_multiplier: mem_mult,
        bw_multiplier: bw_mult,
        priority,
        flags: 0,
    };

    match PROCESS_QUOTAS.insert(&pid, &quota) {
        Ok(_) => {
            info!(
                &ctx,
                "Updated quota for PID {}: CPU={}%, PRIO={}", pid, cpu_mult, priority
            );
            0
        }
        Err(_) => -1,
    }
}

/// Throttle process - O(1) enforcement
#[no_mangle]
pub unsafe fn throttle_process(pid: u32) -> i32 {
    if let Some(mut quota) = PROCESS_QUOTAS.get(&pid) {
        quota.flags |= FLAG_THROTTLED;

        match PROCESS_QUOTAS.insert(&pid, &quota) {
            Ok(_) => {
                warn!(&ctx, "Process {} throttled", pid);
                0
            }
            Err(_) => -1,
        }
    } else {
        -1
    }
}

/// Unthrottle process - O(1) enforcement  
pub unsafe fn unthrottle_process(pid: u32) -> i32 {
    if let Some(mut quota) = PROCESS_QUOTAS.get(&pid) {
        quota.flags &= !FLAG_THROTTLED;

        match PROCESS_QUOTAS.insert(&pid, &quota) {
            Ok(_) => {
                info!(&ctx, "Process {} unthrottled", pid);
                0
            }
            Err(_) => -1,
        }
    } else {
        -1
    }
}

/// Get current resource usage - O(1)
pub unsafe fn get_resource_usage(pid: u32) -> Option<ResourceUsage> {
    PROCESS_USAGE.get(&pid).copied()
}

/// Update resource usage - O(1)
pub unsafe fn update_resource_usage(pid: u32, cpu_cycles: u64, mem_bytes: i64) -> i32 {
    let mut usage = match PROCESS_USAGE.get(&pid) {
        Some(u) => *u,
        None => ResourceUsage::default(),
    };

    usage.cpu_cycles = usage.cpu_cycles.saturating_add(cpu_cycles);

    if mem_bytes > 0 {
        usage.memory_bytes = usage.memory_bytes.saturating_add(mem_bytes as u64);
    } else {
        usage.memory_bytes = usage.memory_bytes.saturating_sub((-mem_bytes) as u64);
    }

    usage.last_update_ns = bpf_ktime_get_ns();

    match PROCESS_USAGE.insert(&pid, &usage) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// LSM hook for security checks (Linux Security Modules)
#[lsm]
pub fn sma_quota_check(ctx: LsmContext) -> i32 {
    match try_sma_quota_check(ctx) {
        Ok(ret) => ret,
        Err(_) => 0,
    }
}

fn try_sma_quota_check(ctx: LsmContext) -> Result<i32, ()> {
    let pid = unsafe { bpf_get_current_pid_tgid() } as u32;

    // O(1) quota lookup
    if let Some(quota) = unsafe { PROCESS_QUOTAS.get(&pid) } {
        if quota.flags & FLAG_THROTTLED != 0 {
            return Err(()); // Block operation
        }
    }

    Ok(0) // Allow
}

/// ProbeContext for kprobes
pub struct ProbeContext {
    regs: *mut pt_regs,
}

impl ProbeContext {
    /// Extract syscall argument from pt_regs
    ///
    /// # Safety
    /// Requires valid pt_regs pointer from eBPF probe context
    unsafe fn arg<T: Copy>(&self, n: usize) -> Option<T> {
        if self.regs.is_null() {
            return None;
        }

        // x86_64: arguments are in rdi, rsi, rdx, rcx, r8, r9 (indices 0-5)
        // For syscalls, orig_rax holds the syscall number, then args follow
        // Note: pt_regs field names depend on the architecture
        let arg_ptr = match n {
            0 => core::ptr::addr_of!((*self.regs).di),
            1 => core::ptr::addr_of!((*self.regs).si),
            2 => core::ptr::addr_of!((*self.regs).dx),
            3 => core::ptr::addr_of!((*self.regs).cx),
            4 => core::ptr::addr_of!((*self.regs).r8),
            5 => core::ptr::addr_of!((*self.regs).r9),
            _ => return None,
        };

        Some(core::ptr::read(arg_ptr as *const T))
    }
}

/// LsmContext for LSM hooks
pub struct LsmContext {
    // LSM context
}

/// Get current time in nanoseconds from eBPF helper
///
/// # Safety
/// This is safe to call from eBPF programs as it uses the bpf_ktime_get_ns kernel helper
unsafe fn bpf_ktime_get_ns() -> u64 {
    // Use aya_ebpf::helpers::bpf_ktime_get_ns if available, otherwise implement via inline asm
    #[cfg(target_arch = "bpf")]
    {
        // On BPF target, use the actual eBPF helper
        // Helper number 5 is bpf_ktime_get_ns
        let time_ns: u64;
        core::arch::asm!(
            "call 5",
            out("r0") time_ns,
            options(nomem, nostack, preserves_flags)
        );
        time_ns
    }

    #[cfg(not(target_arch = "bpf"))]
    {
        // For testing on non-BPF targets, return a monotonic counter
        // In production, this should never be reached
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
