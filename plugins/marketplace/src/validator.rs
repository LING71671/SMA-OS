//! Package validator

use crate::MarketplaceError;
use std::path::Path;
use tracing::{error, info, warn};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: vec![],
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

/// Package validator
pub struct PackageValidator;

impl PackageValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate plugin package
    pub fn validate(&self, path: &Path) -> Result<ValidationResult, MarketplaceError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check file exists
        if !path.exists() {
            return Ok(ValidationResult::failure(vec![format!(
                "Package not found: {}",
                path.display()
            )]));
        }

        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() == 0 {
            errors.push("Package file is empty".to_string());
        }

        // Check file extension
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext != "wasm" && ext != "so" {
                warnings.push(format!("Unusual file extension: {}", ext));
            }
        } else {
            warnings.push("No file extension".to_string());
        }

        // TODO: Add more validation
        // - WASM validation
        // - Manifest validation
        // - Signature verification

        if errors.is_empty() {
            Ok(ValidationResult::success().with_warnings(warnings))
        } else {
            Ok(ValidationResult::failure(errors).with_warnings(warnings))
        }
    }
}

impl Default for PackageValidator {
    fn default() -> Self {
        Self::new()
    }
}
