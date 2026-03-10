//! JSON Reporter
//!
//! Generates test reports in JSON format.

use serde::Serialize;

#[derive(Serialize)]
pub struct TestReport {
    pub scenario_name: String,
    pub status: String,
    pub duration_secs: f64,
    pub errors: Vec<String>,
    pub timestamp: String,
}

impl TestReport {
    pub fn new(
        scenario_name: &str,
        success: bool,
        duration_secs: f64,
        errors: Vec<String>,
    ) -> Self {
        Self {
            scenario_name: scenario_name.to_string(),
            status: if success {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            },
            duration_secs,
            errors,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or_default()
    }
}
