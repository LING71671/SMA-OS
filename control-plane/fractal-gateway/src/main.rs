use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya::maps::HashMap;
use aya_log::BpfLogger;
use clap::Parser;
use log::{info, warn};
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't
    // use the new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    unsafe {
        libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/fractal-gateway"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/fractal-gateway"
    ))?;

    if let Err(e) = BpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut Xdp = bpf.program_mut("fractal_gateway").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags")?;

    info!("Fractal-Gateway Attached to {}", opt.iface);

    // -----------------------------------------------------
    // Dynamic BPF Map Manipulation (Rule Injection)
    // -----------------------------------------------------
    let mut blocked_ips: HashMap<_, u32, u8> = HashMap::try_from(bpf.map_mut("BLOCKED_IPS").unwrap())?;
    
    // Example: Block IP 192.168.1.100 (in network-byte order)
    let malicious_ip: u32 = u32::from_be_bytes([192, 168, 1, 100]);
    blocked_ips.insert(malicious_ip, 1, 0)?;

    info!("Dynamically appended IP 192.168.1.100 to the eBPF Drop Map via user-space!");
    info!("Waiting for Ctrl-C...");

    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
