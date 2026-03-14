//! SMA-OS Identity Management Module
//!
//! This module provides identity scheduling capabilities for the SMA-OS control plane.
//! It supports multi-tenant identity management with hierarchical relationships,
//! privilege escalation, and delegation patterns.
//!
//! ## Key Features
//! - Identity lifecycle management (create, update, revoke)
//! - Hierarchical identity relationships (parent-child)
//! - Dynamic privilege escalation and de-escalation
//! - Identity delegation with scope restrictions
//! - Comprehensive audit logging
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Identity Manager            │
//! ├─────────────┬─────────────┬─────────┤
//! │ Identities  │ Credentials │ Hierarchy│
//! └─────────────┴─────────────┴─────────┘
//! │
//! ▼
//! ┌─────────────────────────────────────┐
//! │        Audit Logger                 │
//! └─────────────────────────────────────┘
//! ```

pub mod types;
pub mod manager;
pub mod audit;

// Re-export all types from submodules
pub use types::{
    Capability, CredentialStatus, CredentialType, DelegationToken, EscalationToken,
    IdentityContext, IdentityCredential, IdentityFilter, IdentityScope, IdentityStatus,
    IdentityType, ScopeLevel, ScopedIdentity, SecurityLevel,
};
pub use manager::IdentityManager;
pub use audit::{AuditAction, AuditEvent, AuditLogger};

use thiserror::Error;

/// Result type for identity operations
pub type Result<T> = std::result::Result<T, IdentityError>;

/// Error types for identity operations
#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identity not found: {0}")]
    NotFound(String),
    
    #[error("Identity already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Insufficient privilege: required {required}, current {current}")]
    InsufficientPrivilege { required: String, current: String },
    
    #[error("Identity expired")]
    Expired,
    
    #[error("Identity suspended")]
    Suspended,
    
    #[error("Invalid credential")]
    InvalidCredential,
    
    #[error("Escalation denied: {0}")]
    EscalationDenied(String),
    
    #[error("Not in hierarchy: ancestor {ancestor}, descendant {descendant}")]
    NotInHierarchy { ancestor: String, descendant: String },
    
    #[error("Lock error")]
    LockError,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
}
