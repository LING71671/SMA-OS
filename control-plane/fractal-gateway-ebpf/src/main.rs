//! Fractal Gateway eBPF - XDP Packet Filtering
//!
//! This is the eBPF program that runs in kernel space for nanosecond-level
//! network filtering and IP blocking.

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::HashMap,
    programs::XdpContext,
};
use aya_log_ebpf::info;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::Ipv4Hdr,
};

/// Map to store blocked IP addresses (max 1024 entries)
#[map]
static BLOCKED_IPS: HashMap<u32, u8> = HashMap::with_max_entries(1024, 0);

/// XDP program entry point
#[xdp]
pub fn fractal_gateway(ctx: XdpContext) -> u32 {
    match try_fractal_gateway(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_fractal_gateway(ctx: XdpContext) -> Result<u32, ()> {
    info!(&ctx, "Fractal Gateway: packet intercepted");

    let ethhdr: *const EthHdr = unsafe { ptr_at(&ctx, 0)? };
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };
    let source = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    // Check if source IP is in blocked list
    if unsafe { BLOCKED_IPS.get(&source) }.is_some() {
        info!(&ctx, "FRACTAL SHIELD: dropping packet from blocked IP");
        return Ok(xdp_action::XDP_DROP);
    }

    Ok(xdp_action::XDP_PASS)
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = core::mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }
    Ok((start + offset) as *const T)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // eBPF programs cannot panic, this is a placeholder
    loop {}
}
