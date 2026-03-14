//! Identity Types for SMA-OS Control Plane
//!
//! This module defines the core identity types used for authentication,
//! authorization, and identity lifecycle management across the SMA-OS platform.
//!
//! ## Identity Hierarchy
//!
//! ```text
//! System (Global)
//!   └── Manager (Tenant)
//!         └── Worker (Namespace)
//!               └── Service (Resource)
//!                     └── Ephemeral (Temporary)
//! ```
//!
//! ## Security Levels
//!
//! - `Critical`: System-level identities with unrestricted access
//! - `High`: Manager identities with tenant-wide privileges
//! - `Medium`: Worker identities with namespace-scoped access
//! - `Low`: Service identities with resource-limited access
//! - `Minimal`: Ephemeral identities with minimal permissions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Identity type classification
///
/// Defines the hierarchical role of an identity within the SMA-OS system.
/// Each type has specific privileges and scope restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentityType {
    /// System-level identity with global scope
    /// Used for core infrastructure components and bootstrap operations
    System,

    /// Manager identity with tenant-wide authority
    /// Orchestrates resources and manages workers within a tenant
    Manager,

    /// Worker identity for task execution
    /// Executes tasks within a specific namespace scope
    Worker,

    /// Service identity for long-running services
    /// Represents persistent services with resource-level access
    Service,

    /// Ephemeral identity for temporary operations
    /// Short-lived identities with minimal permissions
    Ephemeral,
}

impl IdentityType {
    /// Returns the string representation of the identity type
    pub fn as_str(&self) -> &str {
        match self {
            IdentityType::System => "system",
            IdentityType::Manager => "manager",
            IdentityType::Worker => "worker",
            IdentityType::Service => "service",
            IdentityType::Ephemeral => "ephemeral",
        }
    }

    /// Returns the default security level for this identity type
    pub fn default_security_level(&self) -> SecurityLevel {
        match self {
            IdentityType::System => SecurityLevel::Critical,
            IdentityType::Manager => SecurityLevel::High,
            IdentityType::Worker => SecurityLevel::Medium,
            IdentityType::Service => SecurityLevel::Low,
            IdentityType::Ephemeral => SecurityLevel::Minimal,
        }
    }

    /// Returns true if this identity type can create identities of the given type
    pub fn can_create(&self, other: IdentityType) -> bool {
        match (self, other) {
            (IdentityType::System, _) => true,
            (IdentityType::Manager, IdentityType::Worker) => true,
            (IdentityType::Manager, IdentityType::Service) => true,
            (IdentityType::Manager, IdentityType::Ephemeral) => true,
            (IdentityType::Worker, IdentityType::Service) => true,
            (IdentityType::Worker, IdentityType::Ephemeral) => true,
            (IdentityType::Service, IdentityType::Ephemeral) => true,
            _ => false,
        }
    }

    /// Returns true if this identity type can escalate to the target level
    pub fn can_escalate_to(&self, target: SecurityLevel) -> bool {
        let current = self.default_security_level();
        current.can_escalate_to(target)
    }

    /// Returns true if this identity type can create child identities
    pub fn can_create_children(&self) -> bool {
        matches!(self, IdentityType::System | IdentityType::Manager)
    }
}

/// Identity scope levels for hierarchical access control
///
/// Defines the boundary within which an identity operates.
/// Higher levels encompass lower levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentityScope {
    /// Global scope - unrestricted access across all tenants
    Global,

    /// Tenant scope - limited to a specific tenant
    Tenant,

    /// Namespace scope - limited to a specific namespace within a tenant
    Namespace,

    /// Resource scope - limited to specific resources
    Resource,
}

impl IdentityScope {
    /// Returns true if this scope encompasses the other scope
    pub fn contains(&self, other: IdentityScope) -> bool {
        match (self, other) {
            (IdentityScope::Global, _) => true,
            (IdentityScope::Tenant, IdentityScope::Tenant) => true,
            (IdentityScope::Tenant, IdentityScope::Namespace) => true,
            (IdentityScope::Tenant, IdentityScope::Resource) => true,
            (IdentityScope::Namespace, IdentityScope::Namespace) => true,
            (IdentityScope::Namespace, IdentityScope::Resource) => true,
            (IdentityScope::Resource, IdentityScope::Resource) => true,
            _ => false,
        }
    }

    /// Returns the parent scope, if any
    pub fn parent(&self) -> Option<IdentityScope> {
        match self {
            IdentityScope::Global => None,
            IdentityScope::Tenant => Some(IdentityScope::Global),
            IdentityScope::Namespace => Some(IdentityScope::Tenant),
            IdentityScope::Resource => Some(IdentityScope::Namespace),
        }
    }
}

/// Scope Level Enumeration (legacy name for compatibility)
pub type ScopeLevel = IdentityScope;

/// Identity Scope with context
///
/// Defines the scope of access for an identity with specific context.
/// The scope is hierarchical: Global > Tenant > Namespace > Resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedIdentity {
    /// The level of scope
    pub level: IdentityScope,

    /// Tenant ID (if level >= Tenant)
    pub tenant_id: Option<String>,

    /// Namespace (if level >= Namespace)
    pub namespace: Option<String>,

    /// Resource ID (if level == Resource)
    pub resource_id: Option<String>,
}

impl Default for ScopedIdentity {
    fn default() -> Self {
        Self {
            level: IdentityScope::Global,
            tenant_id: None,
            namespace: None,
            resource_id: None,
        }
    }
}

impl ScopedIdentity {
    /// Creates a tenant-level scope
    pub fn tenant(tenant_id: impl Into<String>) -> Self {
        Self {
            level: IdentityScope::Tenant,
            tenant_id: Some(tenant_id.into()),
            namespace: None,
            resource_id: None,
        }
    }

    /// Creates a namespace-level scope
    pub fn namespace(tenant_id: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            level: IdentityScope::Namespace,
            tenant_id: Some(tenant_id.into()),
            namespace: Some(namespace.into()),
            resource_id: None,
        }
    }

    /// Creates a resource-level scope
    pub fn resource(
        tenant_id: impl Into<String>,
        namespace: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            level: IdentityScope::Resource,
            tenant_id: Some(tenant_id.into()),
            namespace: Some(namespace.into()),
            resource_id: Some(resource_id.into()),
        }
    }

    /// Checks if this scope can access the given tenant
    pub fn can_access_tenant(&self, tenant_id: &str) -> bool {
        match self.level {
            IdentityScope::Global => true,
            _ => self.tenant_id.as_ref() == Some(&tenant_id.to_string()),
        }
    }

    /// Checks if this scope can access the given namespace
    pub fn can_access_namespace(&self, tenant_id: &str, namespace: &str) -> bool {
        match self.level {
            IdentityScope::Global => true,
            IdentityScope::Tenant => self.tenant_id.as_ref() == Some(&tenant_id.to_string()),
            _ => {
                self.tenant_id.as_ref() == Some(&tenant_id.to_string())
                    && self.namespace.as_ref() == Some(&namespace.to_string())
            }
        }
    }

    /// Checks if this scope can access the given resource
    pub fn can_access_resource(&self, tenant_id: &str, namespace: &str, resource_id: &str) -> bool {
        match self.level {
            IdentityScope::Global => true,
            IdentityScope::Tenant => self.tenant_id.as_ref() == Some(&tenant_id.to_string()),
            IdentityScope::Namespace => {
                self.tenant_id.as_ref() == Some(&tenant_id.to_string())
                    && self.namespace.as_ref() == Some(&namespace.to_string())
            }
            IdentityScope::Resource => {
                self.tenant_id.as_ref() == Some(&tenant_id.to_string())
                    && self.namespace.as_ref() == Some(&namespace.to_string())
                    && self.resource_id.as_ref() == Some(&resource_id.to_string())
            }
        }
    }
}

/// Security classification levels
///
/// Defines the sensitivity and privilege level of an identity.
/// Used for access control and audit logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityLevel {
    /// Minimal security - ephemeral or guest access
    Minimal = 0,

    /// Low security - service accounts with limited access
    Low = 1,

    /// Medium security - standard worker identities
    Medium = 2,

    /// High security - manager identities with elevated privileges
    High = 3,

    /// Critical security - system identities with unrestricted access
    Critical = 4,
}

impl SecurityLevel {
    /// Returns true if this level can escalate to the target level
    pub fn can_escalate_to(&self, target: SecurityLevel) -> bool {
        *self < target
    }

    /// Returns true if this level can de-escalate to the target level
    pub fn can_de_escalate_to(&self, target: SecurityLevel) -> bool {
        *self > target
    }

    /// Returns the numeric value of the security level
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Identity status for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentityStatus {
    /// Identity is active and can be used
    Active,

    /// Identity is temporarily suspended
    Suspended,

    /// Identity has expired
    Expired,

    /// Identity has been revoked
    Revoked,

    /// Identity is pending activation
    Pending,
}

/// Capability Declaration
///
/// Represents a capability that an identity can possess.
/// Capabilities are used for fine-grained access control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "task.execute")
    pub name: String,

    /// Category (e.g., "execute", "storage", "network", "manage")
    pub category: String,

    /// Allowed actions for this capability
    pub actions: Vec<String>,

    /// Optional resource pattern (regex)
    pub resource_pattern: Option<String>,
}

impl Capability {
    /// Creates a new capability
    pub fn new(name: impl Into<String>, category: impl Into<String>, actions: Vec<String>) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            actions,
            resource_pattern: None,
        }
    }

    /// Sets the resource pattern
    pub fn with_resource_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.resource_pattern = Some(pattern.into());
        self
    }

    /// Checks if this capability allows the given action
    pub fn allows_action(&self, action: &str) -> bool {
        self.actions.contains(&action.to_string())
    }
}

/// Core identity context structure
///
/// The primary identity representation used throughout SMA-OS.
/// Contains all metadata needed for authentication, authorization,
/// and identity lifecycle management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityContext {
    /// Unique identity identifier (UUID v4)
    pub id: String,

    /// Human-readable identity name
    pub name: String,

    /// Identity type classification
    pub identity_type: IdentityType,

    /// Access scope with context
    pub scope: ScopedIdentity,

    /// Granted capabilities
    pub capabilities: Vec<Capability>,

    /// Additional claims (e.g., from JWT)
    pub claims: HashMap<String, String>,

    /// Security classification level
    pub security_level: SecurityLevel,

    /// Current status
    pub status: IdentityStatus,

    /// Parent identity ID (for hierarchical relationships)
    pub parent_id: Option<String>,

    /// Generation in hierarchy (0 = root)
    pub generation: i32,

    /// Identity creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// Expiration timestamp (if applicable)
    pub expires_at: Option<DateTime<Utc>>,

    /// Last authentication timestamp
    pub last_authenticated_at: Option<DateTime<Utc>>,

    /// Metadata for extensibility
    pub metadata: HashMap<String, String>,
}

impl IdentityContext {
    /// Creates a new identity context with default values
    pub fn new(
        id: impl Into<String>,
        identity_type: IdentityType,
        name: impl Into<String>,
        scope: ScopedIdentity,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            identity_type,
            scope,
            capabilities: Vec::new(),
            claims: HashMap::new(),
            security_level: identity_type.default_security_level(),
            status: IdentityStatus::Pending,
            parent_id: None,
            generation: 0,
            created_at: now,
            updated_at: now,
            expires_at: Some(now + chrono::Duration::hours(24)),
            last_authenticated_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the parent identity and returns self for chaining
    pub fn with_parent(mut self, parent_id: impl Into<String>, generation: i32) -> Self {
        self.parent_id = Some(parent_id.into());
        self.generation = generation;
        self
    }

    /// Sets the security level and returns self for chaining
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }

    /// Adds a capability and returns self for chaining
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Adds a claim and returns self for chaining
    pub fn with_claim(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.claims.insert(key.into(), value.into());
        self
    }

    /// Sets the expiration time and returns self for chaining
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Checks if the identity has a specific capability
    pub fn has_capability(&self, capability_name: &str) -> bool {
        self.capabilities.iter().any(|c| c.name == capability_name)
    }

    /// Checks if the identity can perform the given action on the given resource
    pub fn can(&self, capability: &str, action: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.name == capability && c.allows_action(action))
    }

    /// Checks if the identity can access the given tenant
    pub fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.scope.can_access_tenant(tenant_id)
    }

    /// Checks if the identity can access the given namespace
    pub fn can_access_namespace(&self, tenant_id: &str, namespace: &str) -> bool {
        self.scope.can_access_namespace(tenant_id, namespace)
    }

    /// Checks if the identity is active
    pub fn is_active(&self) -> bool {
        if self.status != IdentityStatus::Active {
            return false;
        }

        if let Some(expires) = self.expires_at {
            if expires < Utc::now() {
                return false;
            }
        }

        true
    }

    /// Returns true if this identity can create children
    pub fn can_create_children(&self) -> bool {
        self.identity_type.can_create_children()
    }

    /// Updates the modification timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Identity credential for authentication
///
/// Represents the proof of identity used for authentication.
/// Supports multiple credential types for flexibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCredential {
    /// Unique credential identifier
    pub credential_id: String,

    /// Associated identity ID
    pub identity_id: String,

    /// Credential type
    pub credential_type: CredentialType,

    /// Encrypted credential data (e.g., hashed password, public key)
    pub data: String,

    /// Salt for credential data
    pub salt: Option<String>,

    /// Credential status
    pub status: CredentialStatus,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last used timestamp
    pub last_used_at: Option<DateTime<Utc>>,

    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,

    /// Maximum number of uses (if limited)
    pub max_uses: Option<u64>,

    /// Current use count
    pub use_count: u64,

    /// Authorized scopes
    pub scopes: Vec<String>,
}

/// Credential type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CredentialType {
    /// Password-based credential
    Password,

    /// Public key credential (SSH, TLS)
    PublicKey,

    /// API key credential
    ApiKey,

    /// OAuth token credential
    OAuthToken,

    /// JWT token credential
    JwtToken,

    /// Hardware token credential
    HardwareToken,

    /// Biometric credential
    Biometric,

    /// Certificate-based credential
    Certificate,
}

/// Credential status for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CredentialStatus {
    /// Credential is active and can be used
    Active,

    /// Credential is temporarily suspended
    Suspended,

    /// Credential has expired
    Expired,

    /// Credential has been revoked
    Revoked,

    /// Credential has been compromised
    Compromised,
}

impl IdentityCredential {
    /// Creates a new credential
    pub fn new(
        identity_id: impl Into<String>,
        credential_type: CredentialType,
        data: impl Into<String>,
    ) -> Self {
        Self {
            credential_id: Uuid::new_v4().to_string(),
            identity_id: identity_id.into(),
            credential_type,
            data: data.into(),
            salt: None,
            status: CredentialStatus::Active,
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
            max_uses: None,
            use_count: 0,
            scopes: Vec::new(),
        }
    }

    /// Sets the salt and returns self for chaining
    pub fn with_salt(mut self, salt: impl Into<String>) -> Self {
        self.salt = Some(salt.into());
        self
    }

    /// Sets the expiration and returns self for chaining
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Sets the maximum uses and returns self for chaining
    pub fn with_max_uses(mut self, max_uses: u64) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Adds a scope and returns self for chaining
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Records a use of this credential
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used_at = Some(Utc::now());
    }

    /// Checks if the credential is valid for use
    pub fn is_valid(&self) -> bool {
        if self.status != CredentialStatus::Active {
            return false;
        }

        if let Some(expires) = self.expires_at {
            if Utc::now() > expires {
                return false;
            }
        }

        if let Some(max) = self.max_uses {
            if self.use_count >= max {
                return false;
            }
        }

        true
    }

    /// Checks if the credential has the given scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
    }
}

/// Escalation token for temporary privilege elevation
///
/// Represents a time-limited grant of elevated privileges.
/// Used for just-in-time access and break-glass scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationToken {
    /// Unique token identifier
    pub token_id: String,

    /// Identity being escalated
    pub identity_id: String,

    /// Identity that authorized the escalation
    pub authorized_by: String,

    /// Target security level
    pub target_level: SecurityLevel,

    /// Granted capabilities
    pub granted_capabilities: Vec<String>,

    /// Token creation timestamp
    pub created_at: DateTime<Utc>,

    /// Token expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Reason for escalation
    pub reason: String,

    /// Token signature for verification
    pub signature: String,

    /// Whether the token has been used
    pub used: bool,

    /// Timestamp when token was used
    pub used_at: Option<DateTime<Utc>>,

    /// Single-use flag
    pub single_use: bool,
}

impl EscalationToken {
    /// Creates a new escalation token
    pub fn new(
        identity_id: impl Into<String>,
        authorized_by: impl Into<String>,
        target_level: SecurityLevel,
        duration: chrono::Duration,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            token_id: Uuid::new_v4().to_string(),
            identity_id: identity_id.into(),
            authorized_by: authorized_by.into(),
            target_level,
            granted_capabilities: Vec::new(),
            created_at: now,
            expires_at: now + duration,
            reason: reason.into(),
            signature: String::new(),
            used: false,
            used_at: None,
            single_use: true,
        }
    }

    /// Adds a granted capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.granted_capabilities.push(capability.into());
        self
    }

    /// Sets the signature
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    /// Sets single-use flag
    pub fn with_single_use(mut self, single_use: bool) -> Self {
        self.single_use = single_use;
        self
    }

    /// Checks if the token is valid and not expired
    pub fn is_valid(&self) -> bool {
        if self.used && self.single_use {
            return false;
        }
        Utc::now() < self.expires_at
    }

    /// Marks the token as used
    pub fn mark_used(&mut self) {
        self.used = true;
        self.used_at = Some(Utc::now());
    }
}

/// Delegation token for identity impersonation
///
/// Allows one identity to act on behalf of another
/// with restricted scope and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationToken {
    /// Unique token identifier
    pub token_id: String,

    /// Delegator identity (who is delegating)
    pub delegator_id: String,

    /// Delegate identity (who receives the delegation)
    pub delegate_id: String,

    /// Scope of delegation
    pub scope: IdentityScope,

    /// Delegated capabilities
    pub delegated_capabilities: Vec<String>,

    /// Resource restrictions (if any)
    pub resource_restrictions: Vec<String>,

    /// Token creation timestamp
    pub created_at: DateTime<Utc>,

    /// Token expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Delegation reason
    pub reason: String,

    /// Token signature for verification
    pub signature: String,

    /// Whether the delegation is active
    pub active: bool,

    /// Timestamp when delegation was revoked
    pub revoked_at: Option<DateTime<Utc>>,

    /// Revocation reason
    pub revocation_reason: Option<String>,
}

impl DelegationToken {
    /// Creates a new delegation token
    pub fn new(
        delegator_id: impl Into<String>,
        delegate_id: impl Into<String>,
        scope: IdentityScope,
        duration: chrono::Duration,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            token_id: Uuid::new_v4().to_string(),
            delegator_id: delegator_id.into(),
            delegate_id: delegate_id.into(),
            scope,
            delegated_capabilities: Vec::new(),
            resource_restrictions: Vec::new(),
            created_at: now,
            expires_at: now + duration,
            reason: reason.into(),
            signature: String::new(),
            active: true,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Adds a delegated capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.delegated_capabilities.push(capability.into());
        self
    }

    /// Adds a resource restriction
    pub fn with_resource_restriction(mut self, resource: impl Into<String>) -> Self {
        self.resource_restrictions.push(resource.into());
        self
    }

    /// Sets the signature
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    /// Checks if the delegation is valid and active
    pub fn is_valid(&self) -> bool {
        if !self.active {
            return false;
        }
        if self.revoked_at.is_some() {
            return false;
        }
        Utc::now() < self.expires_at
    }

    /// Revokes the delegation
    pub fn revoke(&mut self, reason: impl Into<String>) {
        self.active = false;
        self.revoked_at = Some(Utc::now());
        self.revocation_reason = Some(reason.into());
    }
}

/// Identity relationship for hierarchy tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRelationship {
    /// Relationship ID
    pub relationship_id: String,

    /// Parent identity ID
    pub parent_id: String,

    /// Child identity ID
    pub child_id: String,

    /// Relationship type
    pub relationship_type: RelationshipType,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Relationship type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    /// Parent-child hierarchical relationship
    ParentChild,

    /// Manager-worker relationship
    ManagerWorker,

    /// Service dependency relationship
    ServiceDependency,

    /// Temporary delegation relationship
    TemporaryDelegation,

    /// Group membership relationship
    GroupMembership,
}

/// Identity audit event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAuditEvent {
    /// Event ID
    pub event_id: String,

    /// Identity ID associated with the event
    pub identity_id: String,

    /// Event type
    pub event_type: AuditEventType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Actor identity ID (who performed the action)
    pub actor_id: String,

    /// Event details
    pub details: serde_json::Value,

    /// Source IP address
    pub source_ip: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Success/failure status
    pub success: bool,

    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Audit event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    /// Identity created
    IdentityCreated,

    /// Identity updated
    IdentityUpdated,

    /// Identity deleted
    IdentityDeleted,

    /// Identity activated
    IdentityActivated,

    /// Identity suspended
    IdentitySuspended,

    /// Identity revoked
    IdentityRevoked,

    /// Credential created
    CredentialCreated,

    /// Credential updated
    CredentialUpdated,

    /// Credential revoked
    CredentialRevoked,

    /// Authentication attempt
    AuthenticationAttempt,

    /// Authentication success
    AuthenticationSuccess,

    /// Authentication failure
    AuthenticationFailure,

    /// Escalation granted
    EscalationGranted,

    /// Escalation revoked
    EscalationRevoked,

    /// Delegation created
    DelegationCreated,

    /// Delegation revoked
    DelegationRevoked,

    /// Permission check
    PermissionCheck,

    /// Access granted
    AccessGranted,

    /// Access denied
    AccessDenied,
}

/// Identity Filter
///
/// Used for querying and filtering identities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityFilter {
    /// Filter by type
    pub identity_type: Option<IdentityType>,

    /// Filter by tenant ID
    pub tenant_id: Option<String>,

    /// Filter by namespace
    pub namespace: Option<String>,

    /// Filter by status
    pub status: Option<IdentityStatus>,

    /// Filter by parent ID
    pub parent_id: Option<String>,

    /// Filter by security level (minimum)
    pub min_security_level: Option<SecurityLevel>,

    /// Filter by security level (maximum)
    pub max_security_level: Option<SecurityLevel>,
}

impl IdentityFilter {
    /// Creates a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the identity type filter
    pub fn with_type(mut self, identity_type: IdentityType) -> Self {
        self.identity_type = Some(identity_type);
        self
    }

    /// Sets the tenant ID filter
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Sets the namespace filter
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Sets the status filter
    pub fn with_status(mut self, status: IdentityStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the parent ID filter
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Sets the security level range filter
    pub fn with_security_range(
        mut self,
        min: Option<SecurityLevel>,
        max: Option<SecurityLevel>,
    ) -> Self {
        self.min_security_level = min;
        self.max_security_level = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_type_as_str() {
        assert_eq!(IdentityType::System.as_str(), "system");
        assert_eq!(IdentityType::Manager.as_str(), "manager");
        assert_eq!(IdentityType::Worker.as_str(), "worker");
        assert_eq!(IdentityType::Service.as_str(), "service");
        assert_eq!(IdentityType::Ephemeral.as_str(), "ephemeral");
    }

    #[test]
    fn test_identity_type_default_security_level() {
        assert_eq!(
            IdentityType::System.default_security_level(),
            SecurityLevel::Critical
        );
        assert_eq!(
            IdentityType::Manager.default_security_level(),
            SecurityLevel::High
        );
        assert_eq!(
            IdentityType::Worker.default_security_level(),
            SecurityLevel::Medium
        );
        assert_eq!(
            IdentityType::Service.default_security_level(),
            SecurityLevel::Low
        );
        assert_eq!(
            IdentityType::Ephemeral.default_security_level(),
            SecurityLevel::Minimal
        );
    }

    #[test]
    fn test_identity_type_can_create() {
        assert!(IdentityType::System.can_create(IdentityType::Manager));
        assert!(IdentityType::System.can_create(IdentityType::Worker));
        assert!(IdentityType::Manager.can_create(IdentityType::Worker));
        assert!(IdentityType::Manager.can_create(IdentityType::Service));
        assert!(!IdentityType::Worker.can_create(IdentityType::Manager));
        assert!(!IdentityType::Service.can_create(IdentityType::Manager));
    }

    #[test]
    fn test_identity_scope_contains() {
        assert!(IdentityScope::Global.contains(IdentityScope::Tenant));
        assert!(IdentityScope::Global.contains(IdentityScope::Namespace));
        assert!(IdentityScope::Tenant.contains(IdentityScope::Namespace));
        assert!(IdentityScope::Tenant.contains(IdentityScope::Resource));
        assert!(IdentityScope::Namespace.contains(IdentityScope::Resource));
        assert!(!IdentityScope::Resource.contains(IdentityScope::Namespace));
        assert!(!IdentityScope::Namespace.contains(IdentityScope::Tenant));
    }

    #[test]
    fn test_security_level_escalation() {
        assert!(SecurityLevel::Low.can_escalate_to(SecurityLevel::High));
        assert!(SecurityLevel::Medium.can_escalate_to(SecurityLevel::Critical));
        assert!(!SecurityLevel::High.can_escalate_to(SecurityLevel::Medium));
        assert!(!SecurityLevel::Critical.can_escalate_to(SecurityLevel::High));
    }

    #[test]
    fn test_scoped_identity_access_control() {
        let global = ScopedIdentity::default();
        assert!(global.can_access_tenant("any"));

        let tenant = ScopedIdentity::tenant("tenant-1");
        assert!(tenant.can_access_tenant("tenant-1"));
        assert!(!tenant.can_access_tenant("tenant-2"));

        let ns = ScopedIdentity::namespace("tenant-1", "ns-1");
        assert!(ns.can_access_namespace("tenant-1", "ns-1"));
        assert!(!ns.can_access_namespace("tenant-1", "ns-2"));
    }

    #[test]
    fn test_identity_context_builder() {
        let identity = IdentityContext::new(
            "test-1",
            IdentityType::Worker,
            "test-worker",
            ScopedIdentity::tenant("tenant-1"),
        )
        .with_security_level(SecurityLevel::High)
        .with_claim("role", "admin")
        .with_capability(Capability::new(
            "task.execute",
            "execute",
            vec!["run".to_string()],
        ));

        assert_eq!(identity.security_level, SecurityLevel::High);
        assert!(identity.has_capability("task.execute"));
        assert!(identity.can("task.execute", "run"));
        assert_eq!(identity.claims.get("role"), Some(&"admin".to_string()));
    }

    #[test]
    fn test_credential_validation() {
        let cred = IdentityCredential::new("id-1", CredentialType::ApiKey, "token-1")
            .with_expiration(Utc::now() + chrono::Duration::hours(1));
        assert!(cred.is_valid());

        let expired = IdentityCredential::new("id-1", CredentialType::ApiKey, "token-1")
            .with_expiration(Utc::now() - chrono::Duration::hours(1));
        assert!(!expired.is_valid());
    }

    #[test]
    fn test_escalation_token_validity() {
        let token = EscalationToken::new(
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
            SecurityLevel::High,
            chrono::Duration::hours(1),
            "Emergency access",
        );

        assert!(token.is_valid());
        assert!(!token.used);
    }

    #[test]
    fn test_delegation_token_validity() {
        let token = DelegationToken::new(
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
            IdentityScope::Namespace,
            chrono::Duration::hours(1),
            "Temporary delegation",
        );

        assert!(token.is_valid());
        assert!(token.active);
    }
}
