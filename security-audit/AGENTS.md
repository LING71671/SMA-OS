# Security Audit Framework Guide

**Location**: `security-audit/`
**Domain**: Automated security scanning and penetration testing
**Language**: Rust
**Score**: 12/25 (standalone tool, security-focused)

## Overview

Automated security audit framework for SMA-OS infrastructure. Performs vulnerability scanning, configuration auditing, and compliance reporting against OWASP, PCI-DSS, and HIPAA standards.

## Structure

```
security-audit/
├── src/
│   ├── main.rs           # CLI entry point (clap)
│   ├── checks/           # Security check implementations
│   │   ├── mod.rs
│   │   ├── injection.rs    # SQL, command injection tests
│   │   ├── auth.rs         # Authentication bypass checks
│   │   ├── tls.rs          # TLS/SSL configuration audit
│   │   └── secrets.rs      # Secret leakage detection
│   └── report.rs         # Report generation (JSON, HTML)
├── Cargo.toml          # clap, reqwest, rustls
└── configs/
    └── owasp-top10.toml  # OWASP check definitions
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| CLI args | `main.rs:20-50` | clap derive macro for subcommands |
| Audit runner | `main.rs:60-90` | Full audit orchestration |
| Check trait | `checks/mod.rs` | SecurityCheck interface |
| Report gen | `report.rs` | SecurityReport, output formatting |
| Injection tests | `checks/injection.rs` | SQLi, command injection |
| Auth checks | `checks/auth.rs` | JWT, session validation |

## Conventions (This Module)

### Check Implementation
```rust
pub trait SecurityCheck {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn severity(&self) -> CheckSeverity;
    async fn execute(&self, target: &str) -> Result<CheckResult, AuditError>;
}
```

### Severity Levels
```rust
pub enum CheckSeverity {
    Critical,  // Immediate action required
    High,      // Significant risk
    Medium,    // Moderate concern
    Low,       // Minor issue
    Info,      // Informational
}
```

### Report Structure
```rust
pub struct SecurityReport {
    pub target: String,
    pub timestamp: DateTime<Utc>,
    pub findings: Vec<Finding>,
    pub summary: ScanSummary,
    pub compliance: ComplianceStatus,
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Skip TLS verification in production
let client = reqwest::Client::builder()
    .danger_accept_invalid_certs(true)  // SECURITY RISK
    .build()?;

// ALWAYS: Verify certificates
let client = reqwest::Client::builder()
    .add_root_certificate(cert)
    .build()?;
```

### Error Handling
```rust
// WRONG: Silent failure on check error
match check.execute(target).await {
    Ok(result) => results.push(result),
    Err(_) => continue,  // Lost security finding!
}

// CORRECT: Log and report all outcomes
match check.execute(target).await {
    Ok(result) => results.push(result),
    Err(e) => {
        warn!("Check {} failed: {}", check.name(), e);
        results.push(Finding::error(check.name(), e));
    }
}
```

## Unique Styles

### CLI Structure
```rust
#[derive(Parser)]
#[command(name = "sma-security-audit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Audit { target: String, output: String },
    Check { name: String, target: String },
    Compliance { standard: String },
}
```

### Async Checks
```rust
async fn run_full_audit(target: &str, output: &str) -> Result<(), AuditError> {
    let checks: Vec<Box<dyn SecurityCheck>> = vec![
        Box::new(injection::SqlInjectionCheck),
        Box::new(auth::JwtValidationCheck),
        Box::new(tls::TlsConfigCheck),
    ];
    
    let results = futures::future::join_all(
        checks.iter().map(|c| c.execute(target))
    ).await;
    
    SecurityReport::from(results).save(output).await
}
```

## Commands

```bash
# Build
cd security-audit && cargo build --release

# Run full audit
cargo run --release -- audit --target https://api.sma-os.local --output report.json

# Run specific check
cargo run --release -- check --name tls-config --target https://api.sma-os.local

# Generate compliance report
cargo run --release -- compliance --standard owasp

# Test
cargo test -- --test-threads=1  # Sequential for network tests
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | CLI parsing |
| reqwest | HTTP auditing |
| rustls | TLS/SSL analysis |
| serde | Report serialization |
| chrono | Timestamps |
| tracing | Structured logging |

## Usage Notes

- **Never run against production without approval**
- **Respect rate limits**: Built-in 100ms delay between requests
- **Concurrent safety**: Default 4 concurrent checks
- **Output formats**: JSON (machine-readable), HTML (human-readable), SARIF (GitHub integration)

## Compliance Standards

| Standard | Coverage |
|----------|----------|
| OWASP Top 10 2021 | Full |
| PCI-DSS 4.0 | Partial (network, crypto) |
| HIPAA | Partial (audit logging) |
