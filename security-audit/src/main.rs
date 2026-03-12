//! Security Audit Framework for SMA-OS
//!
//! Phase 3.5: Security audit and penetration testing

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use tracing::{error, info, warn};

mod checks;
mod report;

use checks::{SecurityCheck, CheckResult, CheckSeverity};
use report::SecurityReport;

#[derive(Parser)]
#[command(name = "sma-security-audit")]
#[command(about = "Security audit framework for SMA-OS")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all security checks
    Audit {
        /// Target URL to audit
        #[arg(short, long)]
        target: String,
        /// Output format
        #[arg(short, long, default_value = "json")]
        output: String,
    },
    /// Run specific check
    Check {
        /// Check name
        #[arg(short, long)]
        name: String,
        /// Target URL
        #[arg(short, long)]
        target: String,
    },
    /// Generate compliance report
    Compliance {
        /// Standard (owasp, pci, hipaa)
        #[arg(short, long)]
        standard: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Audit { target, output } => {
            info!("[Security] Starting full audit of {}", target);
            run_full_audit(&target, &output).await?;
        }
        Commands::Check { name, target } => {
            info!("[Security] Running check '{}' on {}", name, target);
            run_single_check(&name, &target).await?;
        }
        Commands::Compliance { standard } => {
            info!("[Security] Generating {} compliance report", standard);
            generate_compliance(&standard).await?;
        }
    }

    Ok(())
}

async fn run_full_audit(target: &str, output: &str) -> anyhow::Result<()> {
    let mut report = SecurityReport::new(target);

    // Run all security checks
    let checks: Vec<Box<dyn SecurityCheck>> = vec![
        Box::new(checks::TlsCheck::new()),
        Box::new(checks::AuthCheck::new()),
        Box::new(checks::InjectionCheck::new()),
        Box::new(checks::SecretsCheck::new()),
        Box::new(checks::HeaderCheck::new()),
        Box::new(checks::RateLimitCheck::new()),
    ];

    for check in checks {
        let result = check.run(target).await;
        report.add_result(result);
    }

    // Output report
    match output {
        "json" => println!("{}", report.to_json()?),
        "html" => println!("{}", report.to_html()?),
        _ => println!("{}", report.to_text()),
    }

    let summary = report.summary();
    info!(
        "[Security] Audit complete: {} passed, {} warnings, {} critical",
        summary.passed, summary.warnings, summary.critical
    );

    Ok(())
}

async fn run_single_check(name: &str, target: &str) -> anyhow::Result<()> {
    let check: Box<dyn SecurityCheck> = match name {
        "tls" => Box::new(checks::TlsCheck::new()),
        "auth" => Box::new(checks::AuthCheck::new()),
        "injection" => Box::new(checks::InjectionCheck::new()),
        "secrets" => Box::new(checks::SecretsCheck::new()),
        "headers" => Box::new(checks::HeaderCheck::new()),
        "rate-limit" => Box::new(checks::RateLimitCheck::new()),
        _ => {
            error!("[Security] Unknown check: {}", name);
            return Ok(());
        }
    };

    let result = check.run(target).await;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn generate_compliance(standard: &str) -> anyhow::Result<()> {
    info!("[Security] {} compliance checklist:", standard);
    
    let checklist = match standard {
        "owasp" => get_owasp_checklist(),
        "pci" => get_pci_checklist(),
        "hipaa" => get_hipaa_checklist(),
        _ => {
            warn!("[Security] Unknown standard: {}", standard);
            return Ok(());
        }
    };

    for item in checklist {
        println!("[{}] {}", item.0, item.1);
    }

    Ok(())
}

fn get_owasp_checklist() -> Vec<(String, String)> {
    vec![
        ("A01:2021".to_string(), "Broken Access Control".to_string()),
        ("A02:2021".to_string(), "Cryptographic Failures".to_string()),
        ("A03:2021".to_string(), "Injection".to_string()),
        ("A04:2021".to_string(), "Insecure Design".to_string()),
        ("A05:2021".to_string(), "Security Misconfiguration".to_string()),
        ("A06:2021".to_string(), "Vulnerable Components".to_string()),
        ("A07:2021".to_string(), "Auth Failures".to_string()),
        ("A08:2021".to_string(), "Data Integrity Failures".to_string()),
        ("A09:2021".to_string(), "Logging Failures".to_string()),
        ("A10:2021".to_string(), "SSRF".to_string()),
    ]
}

fn get_pci_checklist() -> Vec<(String, String)> {
    vec![
        ("Req 1".to_string(), "Install firewall".to_string()),
        ("Req 2".to_string(), "Default passwords".to_string()),
        ("Req 3".to_string(), "Protect stored data".to_string()),
        ("Req 4".to_string(), "Encrypt transmission".to_string()),
    ]
}

fn get_hipaa_checklist() -> Vec<(String, String)> {
    vec![
        ("164.312".to_string(), "Access control".to_string()),
        ("164.312".to_string(), "Audit controls".to_string()),
        ("164.312".to_string(), "Integrity controls".to_string()),
    ]
}
