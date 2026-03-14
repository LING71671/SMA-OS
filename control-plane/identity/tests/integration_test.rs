//! Identity Module Integration Tests
//!
//! Tests the complete identity lifecycle including creation, hierarchy,
//! credentials, escalation, and delegation.

use identity::{
    AuditAction, AuditEvent, AuditLogger, Capability, DelegationToken,
    EscalationToken, IdentityContext, IdentityCredential, IdentityError,
    IdentityFilter, IdentityManager, IdentityScope, IdentityStatus, IdentityType,
    Result, ScopeLevel,
};
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_complete_identity_lifecycle() {
    let manager = IdentityManager::new();
    
    // 1. Create a manager identity
    let manager_identity = manager
        .create_identity(
            IdentityType::Manager,
            "test-manager",
            IdentityScope::tenant("test-tenant"),
            None,
        )
        .unwrap();
    
    assert_eq!(manager_identity.identity_type, IdentityType::Manager);
    assert!(manager_identity.is_active());
    assert_eq!(manager_identity.generation, 0);
    
    // 2. Create a worker identity under the manager
    let worker_identity = manager
        .create_child(
            &manager_identity.id,
            "test-worker",
            IdentityType::Worker,
        )
        .unwrap();
    
    assert_eq!(worker_identity.parent_id, Some(manager_identity.id.clone()));
    assert_eq!(worker_identity.generation, 1);
    
    // 3. Issue credentials
    let credential = manager
        .issue_credential(
            &worker_identity.id,
            vec!["task.execute".to_string()],
            3600,
        )
        .unwrap();
    
    assert_eq!(credential.identity_id, worker_identity.id);
    assert!(credential.is_valid());
    assert!(credential.has_scope("task.execute"));
    
    // 4. Add capability and escalate
    let mut updated = worker_identity.clone();
    updated.capabilities.push(Capability::new(
        "admin.execute",
        "manage",
        vec!["run", "stop", "restart"],
    ));
    manager.update_identity(&worker_identity.id, updated).unwrap();
    
    let escalation = manager
        .escalate(
            &worker_identity.id,
            "admin.execute",
            "Need admin access for maintenance",
            300,
        )
        .unwrap();
    
    assert_eq!(escalation.identity_id, worker_identity.id);
    assert_eq!(escalation.capability, "admin.execute");
    assert!(escalation.is_valid());
    
    // 5. Create delegation
    let another_worker = manager
        .create_child(
            &manager_identity.id,
            "another-worker",
            IdentityType::Worker,
        )
        .unwrap();
    
    let delegation = manager
        .delegate(
            &worker_identity.id,
            &another_worker.id,
            vec!["task.read".to_string()],
            3600,
        )
        .unwrap();
    
    assert_eq!(delegation.delegator_id, worker_identity.id);
    assert_eq!(delegation.delegatee_id, another_worker.id);
    
    // 6. Verify hierarchy
    let children = manager.get_children(&manager_identity.id);
    assert_eq!(children.len(), 2);
    assert!(children.contains(&worker_identity.id));
    assert!(children.contains(&another_worker.id));
    
    // 7. List identities with filter
    let filter = IdentityFilter::new()
        .with_type(IdentityType::Worker)
        .with_tenant("test-tenant");
    
    let workers = manager.list_identities(&filter);
    assert_eq!(workers.len(), 2);
    
    // 8. Cascade revoke
    manager.revoke_identity(&manager_identity.id, "Test completion").unwrap();
    
    // Verify all children are revoked
    let worker = manager.get_identity(&worker_identity.id).unwrap();
    assert_eq!(worker.status, IdentityStatus::Revoked);
    
    let another = manager.get_identity(&another_worker.id).unwrap();
    assert_eq!(another.status, IdentityStatus::Revoked);
}

#[tokio::test]
async fn test_privilege_escalation_denial() {
    let manager = IdentityManager::new();
    
    // Create worker without admin capability
    let worker = manager
        .create_identity(
            IdentityType::Worker,
            "limited-worker",
            IdentityScope::tenant("tenant-1"),
            None,
        )
        .unwrap();
    
    // Try to escalate without having the capability
    let result = manager.escalate(
        &worker.id,
        "admin.execute",
        "Should fail",
        300,
    );
    
    assert!(matches!(result, Err(IdentityError::EscalationDenied(_))));
}

#[tokio::test]
async fn test_credential_expiration() {
    let manager = IdentityManager::new();
    
    let identity = manager
        .create_identity(
            IdentityType::Worker,
            "expiring-worker",
            IdentityScope::tenant("tenant-1"),
            None,
        )
        .unwrap();
    
    // Issue credential with very short validity
    let credential = manager
        .issue_credential(
            &identity.id,
            vec!["test".to_string()],
            1, // 1 second
        )
        .unwrap();
    
    // Should be valid immediately
    assert!(credential.is_valid());
    
    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Validate should fail with expired
    let result = manager.validate_credential(&credential.token);
    assert!(matches!(result, Err(IdentityError::Expired)));
}

#[tokio::test]
async fn test_scope_compatibility() {
    let manager = IdentityManager::new();
    
    // Create manager with namespace scope
    let manager_identity = manager
        .create_identity(
            IdentityType::Manager,
            "ns-manager",
            IdentityScope::namespace("tenant-1", "namespace-1"),
            None,
        )
        .unwrap();
    
    // Child should not have broader scope than parent
    let result = manager.create_identity(
        IdentityType::Worker,
        "broad-worker",
        IdentityScope::tenant("tenant-1"), // Broader than namespace
        Some(&manager_identity.id),
    );
    
    assert!(matches!(result, Err(IdentityError::InsufficientPrivilege { .. })));
}

#[tokio::test]
async fn test_system_identity_protection() {
    let manager = IdentityManager::new();
    
    // Get the orchestrator system identity
    let system_identity = manager
        .get_identity("system:orchestrator")
        .unwrap();
    
    // Cannot revoke system identity
    let result = manager.revoke_identity(&system_identity.id, "Should fail");
    assert!(matches!(result, Err(IdentityError::InsufficientPrivilege { .. })));
}

#[tokio::test]
async fn test_audit_logging() {
    let audit = AuditLogger::new();
    
    // Create audit event
    let event = AuditEvent::new(
        "identity-1",
        "WORKER",
        AuditAction::IdentityCreated,
    )
    .with_target("target-1", "identity")
    .with_metadata(serde_json::json!({
        "tenant": "test-tenant"
    }));
    
    // Log the event
    audit.log(event).await;
    
    // Verify logging methods
    audit.log_identity_created(&IdentityContext::new(
        "test-1",
        IdentityType::Worker,
        "test",
        IdentityScope::default(),
    )).await;
    
    audit.log_escalation_granted("identity-1", "admin.execute", "test").await;
    
    // Test passes if no panics
}