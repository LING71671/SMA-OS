//! Audit Logging Module
//!
//! Provides comprehensive audit logging for identity-related operations.
//! Supports both PostgreSQL for operational data and ClickHouse for analytics.

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit Action Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    IdentityCreated,
    IdentityUpdated,
    IdentityRevoked,
    IdentitySuspended,
    IdentityActivated,
    CredentialIssued,
    CredentialValidated,
    CredentialRevoked,
    EscalationGranted,
    EscalationRevoked,
    DelegationGranted,
    DelegationRevoked,
    ChildCreated,
    CascadeRevoke,
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::IdentityCreated => "IDENTITY_CREATED",
            AuditAction::IdentityUpdated => "IDENTITY_UPDATED",
            AuditAction::IdentityRevoked => "IDENTITY_REVOKED",
            AuditAction::IdentitySuspended => "IDENTITY_SUSPENDED",
            AuditAction::IdentityActivated => "IDENTITY_ACTIVATED",
            AuditAction::CredentialIssued => "CREDENTIAL_ISSUED",
            AuditAction::CredentialValidated => "CREDENTIAL_VALIDATED",
            AuditAction::CredentialRevoked => "CREDENTIAL_REVOKED",
            AuditAction::EscalationGranted => "ESCALATION_GRANTED",
            AuditAction::EscalationRevoked => "ESCALATION_REVOKED",
            AuditAction::DelegationGranted => "DELEGATION_GRANTED",
            AuditAction::DelegationRevoked => "DELEGATION_REVOKED",
            AuditAction::ChildCreated => "CHILD_CREATED",
            AuditAction::CascadeRevoke => "CASCADE_REVOKE",
        }
    }
}

/// Audit Event Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: Uuid,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Identity that performed the action
    pub identity_id: String,
    
    /// Type of identity
    pub identity_type: String,
    
    /// Action performed
    pub action: AuditAction,
    
    /// Target ID (if applicable)
    pub target_id: Option<String>,
    
    /// Target type
    pub target_type: Option<String>,
    
    /// Whether the action succeeded
    pub success: bool,
    
    /// Error message (if failed)
    pub error_message: Option<String>,
    
    /// IP address
    pub ip_address: Option<String>,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Request ID for tracing
    pub request_id: Option<Uuid>,
    
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl AuditEvent {
    /// Creates a new audit event
    pub fn new(
        identity_id: impl Into<String>,
        identity_type: impl Into<String>,
        action: AuditAction,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            identity_id: identity_id.into(),
            identity_type: identity_type.into(),
            action,
            target_id: None,
            target_type: None,
            success: true,
            error_message: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            metadata: None,
        }
    }
    
    /// Sets the target
    pub fn with_target(mut self, target_id: impl Into<String>, target_type: impl Into<String>) -> Self {
        self.target_id = Some(target_id.into());
        self.target_type = Some(target_type.into());
        self
    }
    
    /// Marks the event as failed
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }
    
    /// Sets client information
    pub fn with_client_info(
        mut self,
        ip: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        self.ip_address = Some(ip.into());
        self.user_agent = Some(user_agent.into());
        self
    }
    
    /// Sets the request ID
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    /// Sets metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Audit Logger
pub struct AuditLogger {
    // PostgreSQL connection pool
    // ClickHouse connection
}

impl AuditLogger {
    /// Creates a new audit logger
    pub fn new() -> Self {
        Self {}
    }
    
    /// Logs an event
    pub async fn log(&self, event: AuditEvent) {
        // Log to console
        tracing::info!(
            "[Audit] {}: {} performed {:?} on {:?} = {}",
            event.event_id,
            event.identity_id,
            event.action,
            event.target_id,
            if event.success { "SUCCESS" } else { "FAILED" }
        );
        
        // TODO: Write to PostgreSQL
        // TODO: Batch write to ClickHouse for analytics
    }
    
    /// Logs identity creation
    pub async fn log_identity_created(&self, identity: &IdentityContext) {
        let event = AuditEvent::new(
            &identity.id,
            identity.identity_type.as_str(),
            AuditAction::IdentityCreated,
        )
        .with_target(&identity.id, "identity");
        
        self.log(event).await;
    }
    
    /// Logs identity revocation
    pub async fn log_identity_revoked(&self, identity_id: &str, reason: &str) {
        let event = AuditEvent::new(
            identity_id,
            "UNKNOWN",
            AuditAction::IdentityRevoked,
        )
        .with_target(identity_id, "identity")
        .with_metadata(serde_json::json!({
            "reason": reason
        }));
        
        self.log(event).await;
    }
    
    /// Logs escalation
    pub async fn log_escalation_granted(
        &self,
        identity_id: &str,
        capability: &str,
        reason: &str,
    ) {
        let event = AuditEvent::new(
            identity_id,
            "WORKER",
            AuditAction::EscalationGranted,
        )
        .with_target(capability, "capability")
        .with_metadata(serde_json::json!({
            "reason": reason
        }));
        
        self.log(event).await;
    }
    
    /// Logs delegation
    pub async fn log_delegation_granted(
        &self,
        delegator_id: &str,
        delegatee_id: &str,
        scopes: &[String],
    ) {
        let event = AuditEvent::new(
            delegator_id,
            "MANAGER",
            AuditAction::DelegationGranted,
        )
        .with_target(delegatee_id, "identity")
        .with_metadata(serde_json::json!({
            "scopes": scopes
        }));
        
        self.log(event).await;
    }
    
    /// Logs credential issuance
    pub async fn log_credential_issued(&self, identity_id: &str, scopes: &[String]) {
        let event = AuditEvent::new(
            identity_id,
            "UNKNOWN",
            AuditAction::CredentialIssued,
        )
        .with_metadata(serde_json::json!({
            "scopes": scopes
        }));
        
        self.log(event).await;
    }
    
    /// Logs cascade revoke
    pub async fn log_cascade_revoke(
        &self,
        parent_id: &str,
        revoked_count: usize,
        revoked_ids: &[String],
    ) {
        let event = AuditEvent::new(
            parent_id,
            "MANAGER",
            AuditAction::CascadeRevoke,
        )
        .with_target(parent_id, "identity")
        .with_metadata(serde_json::json!({
            "revoked_count": revoked_count,
            "revoked_ids": revoked_ids
        }));
        
        self.log(event).await;
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}