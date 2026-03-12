//! Security audit reports

use serde::{Deserialize, Serialize};

/// Security report
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityReport {
    pub target: String,
    pub timestamp: String,
    pub results: Vec<crate::checks::CheckResult>,
}

impl SecurityReport {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: crate::checks::CheckResult) {
        self.results.push(result);
    }

    pub fn summary(&self) -> ReportSummary {
        let mut passed = 0;
        let mut warnings = 0;
        let mut critical = 0;

        for result in &self.results {
            if result.passed {
                passed += 1;
            } else {
                match result.severity {
                    crate::checks::CheckSeverity::Low | crate::checks::CheckSeverity::Medium => {
                        warnings += 1
                    }
                    crate::checks::CheckSeverity::High | crate::checks::CheckSeverity::Critical => {
                        critical += 1
                    }
                    _ => {}
                }
            }
        }

        ReportSummary {
            total: self.results.len(),
            passed,
            warnings,
            critical,
        }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_html(&self) -> String {
        let mut html = format!(
            "<html><body><h1>SMA-OS Security Report</h1><p>Target: {}</p><p>Time: {}</p><table>",
            self.target, self.timestamp
        );

        for result in &self.results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                result.name, status, result.message
            ));
        }

        html.push_str("</table></body></html>");
        html
    }

    pub fn to_text(&self) -> String {
        let mut text = format!(
            "SMA-OS Security Report\nTarget: {}\nTime: {}\n\n",
            self.target, self.timestamp
        );

        for result in &self.results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            text.push_str(&format!(
                "[{}] {}: {}\n",
                status, result.name, result.message
            ));
        }

        text
    }
}

/// Report summary
#[derive(Debug)]
pub struct ReportSummary {
    pub total: usize,
    pub passed: usize,
    pub warnings: usize,
    pub critical: usize,
}
