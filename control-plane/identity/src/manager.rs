//! Identity Manager Module
//!
//! This module provides the core identity management functionality for SMA-OS.
//! It handles identity lifecycle, hierarchy management, credential issuance,
//! privilege escalation, and delegation with full PostgreSQL persistence.
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         IdentityManager                 │
//! ├─────────────┬─────────────┬─────────────┤
//! │  Identities │ Credentials │  Hierarchy  │
//! │   (DashMap) │  (DashMap)  │   (DashMap) │
//! └─────────────┴─────────────┴─────────────┘
//!              │
//!              ▼
//! ┌─────────────────────────────────────────┐
//! │      PostgreSQL Persistence Layer       │
//! └─────────────────────────────────────────┘
//! ```

use crate::{
    Capability, CredentialStatus, CredentialType, DelegationToken, EscalationToken,
    IdentityContext, IdentityCredential, IdentityError, IdentityFilter, IdentityScope,
    IdentityStatus, IdentityType, Result, ScopedIdentity, SecurityLevel,
};
use arc_swap::ArcSwap;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

/// IdentityManager provides thread-safe identity management with persistence.
///
/// Uses DashMap for concurrent in-memory storage and PostgreSQL for durability.
/// All operations are async and support concurrent access.
#[derive(Debug)]
pub struct IdentityManager {
    /// In-memory identity cache (id -> IdentityContext)
    identities: DashMap<String, IdentityContext>,
    /// Active credentials cache (credential_id -> IdentityCredential)
    credentials: DashMap<String, IdentityCredential>,
    /// Active escalation tokens (token_id -> EscalationToken)
    escalation_tokens: DashMap<String, EscalationToken>,
    /// Active delegation tokens (token_id -> DelegationToken)
    delegation_tokens: DashMap<String, DelegationToken>,
    /// Hierarchy cache (parent_id -> Vec<child_id>)
    hierarchy: DashMap<String, Vec<String>>,
    /// PostgreSQL connection pool
    pg_pool: ArcSwap<Option<PgPool>>,
}

impl IdentityManager {
    /// Creates a new IdentityManager without database connection.
    ///
    /// Use `with_pool` to add persistence.
    pub fn new() -> Self {
        Self {
            identities: DashMap::new(),
            credentials: DashMap::new(),
            escalation_tokens: DashMap::new(),
            delegation_tokens: DashMap::new(),
            hierarchy: DashMap::new(),
            pg_pool: ArcSwap::new(Arc::new(None)),
        }
    }

    /// Creates a new IdentityManager with PostgreSQL persistence.
    ///
    /// # Arguments
    /// * `pg_pool` - PostgreSQL connection pool
    pub fn with_pool(pg_pool: PgPool) -> Self {
        let manager = Self::new();
        manager.pg_pool.store(Arc::new(Some(pg_pool)));
        manager
    }

    /// Sets the PostgreSQL connection pool.
    pub fn set_pool(&self, pg_pool: PgPool) {
        self.pg_pool.store(Arc::new(Some(pg_pool)));
    }

    /// Gets the PostgreSQL pool if configured.
    fn pool(&self) -> Option<PgPool> {
        self.pg_pool.load().as_ref().clone()
    }

    /// Creates a new identity with persistence.
    #[instrument(skip(self))]
    pub async fn create_identity(
        &self,
        identity_type: IdentityType,
        name: String,
        scope: ScopedIdentity,
        parent_id: Option<String>,
    ) -> Result<IdentityContext> {
        let id = format!("{}:{}:{}", identity_type.as_str(), name, Uuid::new_v4());

        // Validate parent if provided
        let generation = if let Some(ref pid) = parent_id {
            let parent = self
                .get_identity(pid)
                .await?
                .ok_or_else(|| IdentityError::NotFound(pid.clone()))?;

            if !parent.can_create_children() {
                return Err(IdentityError::InsufficientPrivilege {
                    required: "can_create_children".to_string(),
                    current: parent.identity_type.as_str().to_string(),
                });
            }

            parent.generation + 1
        } else {
            0
        };

        let mut identity = IdentityContext::new(&id, identity_type, &name, scope);
        identity.generation = generation;
        identity.status = IdentityStatus::Active; // Activate identity on creation
        if let Some(ref pid) = parent_id {
            identity.parent_id = Some(pid.clone());
        }

        // Persist to database
        if let Some(pool) = self.pool() {
            self.persist_identity(&pool, &identity).await?;
        }

        // Update in-memory cache
        self.identities.insert(id.clone(), identity.clone());

        // Update hierarchy
        if let Some(ref pid) = parent_id {
            self.hierarchy
                .entry(pid.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        info!(identity_id = %id, "Created new identity");
        Ok(identity)
    }

    /// Retrieves an identity by ID.
    #[instrument(skip(self))]
    pub async fn get_identity(&self, id: &str) -> Result<Option<IdentityContext>> {
        // Check in-memory cache first
        if let Some(identity) = self.identities.get(id) {
            return Ok(Some(identity.clone()));
        }

        // Fallback to database
        if let Some(pool) = self.pool() {
            match self.load_identity(&pool, id).await {
                Ok(Some(identity)) => {
                    self.identities.insert(id.to_string(), identity.clone());
                    return Ok(Some(identity));
                }
                Ok(None) => return Ok(None),
                Err(e) => {
                    error!(error = %e, "Failed to load identity from database");
                    return Err(e);
                }
            }
        }

        Ok(None)
    }

    /// Updates an existing identity.
    #[instrument(skip(self))]
    pub async fn update_identity(&self, identity: &IdentityContext) -> Result<()> {
        // Verify identity exists
        if !self.identities.contains_key(&identity.id) {
            if self.get_identity(&identity.id).await?.is_none() {
                return Err(IdentityError::NotFound(identity.id.clone()));
            }
        }

        // Persist to database
        if let Some(pool) = self.pool() {
            self.persist_identity(&pool, identity).await?;
        }

        // Update cache
        self.identities.insert(identity.id.clone(), identity.clone());

        info!(identity_id = %identity.id, "Updated identity");
        Ok(())
    }

    /// Revokes an identity and all its children recursively.
    #[instrument(skip(self))]
    pub async fn revoke_identity(&self, id: &str, reason: String) -> Result<()> {
        // Use Box::pin to handle recursion in async
        Box::pin(self._revoke_identity_recursive(id, reason)).await
    }
    
    /// Internal recursive implementation
    async fn _revoke_identity_recursive(&self, id: &str, reason: String) -> Result<()> {
        let mut identity = self
            .get_identity(id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(id.to_string()))?;

        // Update status
        identity.status = IdentityStatus::Revoked;

        // Persist
        if let Some(pool) = self.pool() {
            self.persist_identity(&pool, &identity).await?;
        }

        self.identities.insert(id.to_string(), identity);

        // Revoke all credentials
        self.revoke_all_credentials(id).await;

        // Recursively revoke children
        let children = self.get_children(id).await;
        for child_id in children {
            Box::pin(self._revoke_identity_recursive(&child_id, format!("parent revoked: {}", id))).await?;
        }

        info!(identity_id = %id, reason = %reason, "Revoked identity");
        Ok(())
    }

    /// Suspends an identity temporarily.
    #[instrument(skip(self))]
    pub async fn suspend_identity(&self, id: &str, reason: String) -> Result<()> {
        let mut identity = self
            .get_identity(id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(id.to_string()))?;

        identity.status = IdentityStatus::Suspended;

        if let Some(pool) = self.pool() {
            self.persist_identity(&pool, &identity).await?;
        }

        self.identities.insert(id.to_string(), identity);

        info!(identity_id = %id, reason = %reason, "Suspended identity");
        Ok(())
    }

    /// Activates a suspended identity.
    #[instrument(skip(self))]
    pub async fn activate_identity(&self, id: &str) -> Result<()> {
        let mut identity = self
            .get_identity(id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(id.to_string()))?;

        identity.status = IdentityStatus::Active;

        if let Some(pool) = self.pool() {
            self.persist_identity(&pool, &identity).await?;
        }

        self.identities.insert(id.to_string(), identity);

        info!(identity_id = %id, "Activated identity");
        Ok(())
    }

    /// Issues a new credential for an identity.
    #[instrument(skip(self))]
    pub async fn issue_credential(
        &self,
        identity_id: &str,
        credential_type: CredentialType,
        data: String,
    ) -> Result<IdentityCredential> {
        let identity = self
            .get_identity(identity_id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(identity_id.to_string()))?;

        if !identity.is_active() {
            return Err(IdentityError::Suspended);
        }

        let credential = IdentityCredential::new(identity_id, credential_type, &data)
            .with_expiration(Utc::now() + Duration::hours(24));

        // Persist credential
        if let Some(pool) = self.pool() {
            self.persist_credential(&pool, &credential).await?;
        }

        self.credentials
            .insert(credential.credential_id.clone(), credential.clone());

        info!(identity_id = %identity_id, "Issued new credential");
        Ok(credential)
    }

    /// Validates a credential by ID.
    #[instrument(skip(self))]
    pub async fn validate_credential(&self, credential_id: &str) -> Result<IdentityCredential> {
        // Check in-memory cache
        if let Some(cred) = self.credentials.get(credential_id) {
            if cred.is_valid() {
                return Ok(cred.clone());
            } else {
                return Err(IdentityError::Expired);
            }
        }

        // Fallback to database
        if let Some(pool) = self.pool() {
            if let Some(cred) = self.load_credential(&pool, credential_id).await? {
                if cred.is_valid() {
                    self.credentials
                        .insert(credential_id.to_string(), cred.clone());
                    return Ok(cred);
                } else {
                    return Err(IdentityError::Expired);
                }
            }
        }

        Err(IdentityError::InvalidCredential)
    }

    /// Revokes a specific credential.
    #[instrument(skip(self))]
    pub async fn revoke_credential(&self, credential_id: &str) -> Result<()> {
        self.credentials.remove(credential_id);

        if let Some(pool) = self.pool() {
            sqlx::query("DELETE FROM identity_credentials WHERE credential_id = $1")
                .bind(credential_id)
                .execute(&pool)
                .await
                .map_err(|e| IdentityError::Storage(e.to_string()))?;
        }

        info!(credential_id = %credential_id, "Revoked credential");
        Ok(())
    }

    /// Grants a temporary privilege escalation.
    #[instrument(skip(self))]
    pub async fn escalate(
        &self,
        identity_id: &str,
        authorized_by: &str,
        target_level: SecurityLevel,
        duration: Duration,
        reason: String,
    ) -> Result<EscalationToken> {
        let identity = self
            .get_identity(identity_id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(identity_id.to_string()))?;

        // Check if identity is active
        if !identity.is_active() {
            return Err(IdentityError::Suspended);
        }

        let token =
            EscalationToken::new(identity_id, authorized_by, target_level, duration, &reason);

        if let Some(pool) = self.pool() {
            self.persist_escalation_token(&pool, &token).await?;
        }

        self.escalation_tokens
            .insert(token.token_id.clone(), token.clone());

        info!(identity_id = %identity_id, "Granted escalation");
        Ok(token)
    }

    /// Revokes an escalation token.
    #[instrument(skip(self))]
    pub async fn de_escalate(&self, token_id: &str) -> Result<()> {
        self.escalation_tokens.remove(token_id);

        if let Some(pool) = self.pool() {
            sqlx::query("DELETE FROM escalation_tokens WHERE token_id = $1")
                .bind(token_id)
                .execute(&pool)
                .await
                .map_err(|e| IdentityError::Storage(e.to_string()))?;
        }

        info!(token_id = %token_id, "Revoked escalation");
        Ok(())
    }

    /// Validates an escalation token.
    #[instrument(skip(self))]
    pub async fn validate_escalation(&self, token_id: &str) -> Result<EscalationToken> {
        if let Some(token) = self.escalation_tokens.get(token_id) {
            if token.is_valid() {
                return Ok(token.clone());
            } else {
                return Err(IdentityError::Expired);
            }
        }

        if let Some(pool) = self.pool() {
            if let Some(token) = self.load_escalation_token(&pool, token_id).await? {
                if token.is_valid() {
                    self.escalation_tokens
                        .insert(token_id.to_string(), token.clone());
                    return Ok(token);
                }
            }
        }

        Err(IdentityError::NotFound(token_id.to_string()))
    }

    /// Creates a delegation from one identity to another.
    #[instrument(skip(self))]
    pub async fn delegate(
        &self,
        delegator_id: &str,
        delegate_id: &str,
        scope: IdentityScope,
        duration: Duration,
        reason: String,
    ) -> Result<DelegationToken> {
        // Verify both identities exist
        let delegator = self
            .get_identity(delegator_id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(delegator_id.to_string()))?;

        let _delegatee = self
            .get_identity(delegate_id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(delegate_id.to_string()))?;

        // Check hierarchy relationship
        if !self.is_ancestor(delegator_id, delegate_id).await
            && !self.is_descendant(delegator_id, delegate_id).await
        {
            return Err(IdentityError::NotInHierarchy {
                ancestor: delegator_id.to_string(),
                descendant: delegate_id.to_string(),
            });
        }

        // Check if delegator is active
        if !delegator.is_active() {
            return Err(IdentityError::Suspended);
        }

        let token = DelegationToken::new(delegator_id, delegate_id, scope, duration, &reason);

        if let Some(pool) = self.pool() {
            self.persist_delegation_token(&pool, &token).await?;
        }

        self.delegation_tokens
            .insert(token.token_id.clone(), token.clone());

        info!(delegator = %delegator_id, delegatee = %delegate_id, "Created delegation");
        Ok(token)
    }

    /// Revokes a delegation token.
    #[instrument(skip(self))]
    pub async fn revoke_delegation(&self, token_id: &str) -> Result<()> {
        self.delegation_tokens.remove(token_id);

        if let Some(pool) = self.pool() {
            sqlx::query("DELETE FROM delegation_tokens WHERE token_id = $1")
                .bind(token_id)
                .execute(&pool)
                .await
                .map_err(|e| IdentityError::Storage(e.to_string()))?;
        }

        info!(token_id = %token_id, "Revoked delegation");
        Ok(())
    }

    /// Validates a delegation token.
    #[instrument(skip(self))]
    pub async fn validate_delegation(&self, token_id: &str) -> Result<DelegationToken> {
        if let Some(token) = self.delegation_tokens.get(token_id) {
            if token.is_valid() {
                return Ok(token.clone());
            } else {
                return Err(IdentityError::Expired);
            }
        }

        if let Some(pool) = self.pool() {
            if let Some(token) = self.load_delegation_token(&pool, token_id).await? {
                if token.is_valid() {
                    self.delegation_tokens
                        .insert(token_id.to_string(), token.clone());
                    return Ok(token);
                }
            }
        }

        Err(IdentityError::NotFound(token_id.to_string()))
    }

    /// Gets all children of an identity.
    #[instrument(skip(self))]
    pub async fn get_children(&self, parent_id: &str) -> Vec<String> {
        self.hierarchy
            .get(parent_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Gets the parent of an identity.
    #[instrument(skip(self))]
    pub async fn get_parent(&self, child_id: &str) -> Result<Option<String>> {
        let identity = self
            .get_identity(child_id)
            .await?
            .ok_or_else(|| IdentityError::NotFound(child_id.to_string()))?;

        Ok(identity.parent_id)
    }

    /// Lists all identities matching the filter.
    #[instrument(skip(self))]
    pub async fn list_identities(&self, filter: &IdentityFilter) -> Result<Vec<IdentityContext>> {
        let mut results = Vec::new();

        // Query from database if available
        if let Some(pool) = self.pool() {
            let identities = self.query_identities(&pool, filter).await?;
            for identity in identities {
                self.identities.insert(identity.id.clone(), identity.clone());
                results.push(identity);
            }
        } else {
            // In-memory filtering
            for entry in self.identities.iter() {
                if Self::matches_filter(entry.value(), filter) {
                    results.push(entry.value().clone());
                }
            }
        }

        Ok(results)
    }

    /// Checks if ancestor is an ancestor of descendant.
    async fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool {
        let mut current = descendant.to_string();

        loop {
            let identity = match self.get_identity(&current).await {
                Ok(Some(i)) => i,
                _ => break,
            };

            if let Some(parent_id) = identity.parent_id {
                if parent_id == ancestor {
                    return true;
                }
                current = parent_id;
            } else {
                break;
            }
        }

        false
    }

    /// Checks if descendant is a descendant of ancestor.
    async fn is_descendant(&self, descendant: &str, ancestor: &str) -> bool {
        self.is_ancestor(ancestor, descendant).await
    }

    /// Revokes all credentials for an identity.
    async fn revoke_all_credentials(&self, identity_id: &str) {
        let ids_to_remove: Vec<String> = self
            .credentials
            .iter()
            .filter(|entry| entry.value().identity_id == identity_id)
            .map(|entry| entry.key().clone())
            .collect();

        for id in ids_to_remove {
            self.credentials.remove(&id);
        }

        if let Some(pool) = self.pool() {
            let _ = sqlx::query("DELETE FROM identity_credentials WHERE identity_id = $1")
                .bind(identity_id)
                .execute(&pool)
                .await;
        }
    }

    /// Checks if an identity matches the filter.
    fn matches_filter(identity: &IdentityContext, filter: &IdentityFilter) -> bool {
        if let Some(ref identity_type) = filter.identity_type {
            if identity.identity_type != *identity_type {
                return false;
            }
        }

        if let Some(ref tenant_id) = filter.tenant_id {
            if !identity.can_access_tenant(tenant_id) {
                return false;
            }
        }

        if let Some(ref namespace) = filter.namespace {
            if !identity.can_access_namespace(
                filter.tenant_id.as_deref().unwrap_or(""),
                namespace,
            ) {
                return false;
            }
        }

        if let Some(ref status) = filter.status {
            if identity.status != *status {
                return false;
            }
        }

        if let Some(ref parent_id) = filter.parent_id {
            if identity.parent_id.as_ref() != Some(parent_id) {
                return false;
            }
        }

        if let Some(min_level) = filter.min_security_level {
            if identity.security_level < min_level {
                return false;
            }
        }

        if let Some(max_level) = filter.max_security_level {
            if identity.security_level > max_level {
                return false;
            }
        }

        true
    }

    // Database persistence methods

    async fn persist_identity(&self, pool: &PgPool, identity: &IdentityContext) -> Result<()> {
        let capabilities_json = serde_json::to_value(&identity.capabilities)?;
        let claims_json = serde_json::to_value(&identity.claims)?;
        let scope_json = serde_json::to_value(&identity.scope)?;
        let metadata_json = serde_json::to_value(&identity.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO identities (
                id, identity_type, name, scope, capabilities, claims,
                security_level, parent_id, generation, created_at, updated_at,
                expires_at, last_authenticated_at, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id) DO UPDATE SET
                identity_type = EXCLUDED.identity_type,
                name = EXCLUDED.name,
                scope = EXCLUDED.scope,
                capabilities = EXCLUDED.capabilities,
                claims = EXCLUDED.claims,
                security_level = EXCLUDED.security_level,
                parent_id = EXCLUDED.parent_id,
                generation = EXCLUDED.generation,
                expires_at = EXCLUDED.expires_at,
                last_authenticated_at = EXCLUDED.last_authenticated_at,
                status = EXCLUDED.status,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
        )
        .bind(&identity.id)
        .bind(identity.identity_type.as_str())
        .bind(&identity.name)
        .bind(scope_json)
        .bind(capabilities_json)
        .bind(claims_json)
        .bind(identity.security_level.as_u8() as i32)
        .bind(&identity.parent_id)
        .bind(identity.generation)
        .bind(identity.created_at)
        .bind(identity.updated_at)
        .bind(identity.expires_at)
        .bind(identity.last_authenticated_at)
        .bind(format!("{:?}", identity.status))
        .bind(metadata_json)
        .execute(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn load_identity(&self, pool: &PgPool, id: &str) -> Result<Option<IdentityContext>> {
        let row = sqlx::query(
            r#"
            SELECT id, identity_type, name, scope, capabilities, claims,
                   security_level, parent_id, generation, created_at, updated_at,
                   expires_at, last_authenticated_at, status, metadata
            FROM identities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        match row {
            Some(row) => {
                let identity = self.row_to_identity(&row)?;
                Ok(Some(identity))
            }
            None => Ok(None),
        }
    }

    async fn query_identities(
        &self,
        pool: &PgPool,
        filter: &IdentityFilter,
    ) -> Result<Vec<IdentityContext>> {
        let mut query = String::from(
            "SELECT id, identity_type, name, scope, capabilities, claims, security_level, parent_id, generation, created_at, updated_at, expires_at, last_authenticated_at, status, metadata FROM identities WHERE 1=1",
        );

        if filter.identity_type.is_some() {
            query.push_str(" AND identity_type = $1");
        }
        if filter.status.is_some() {
            query.push_str(" AND status = $2");
        }
        if filter.parent_id.is_some() {
            query.push_str(" AND parent_id = $3");
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(ref identity_type) = filter.identity_type {
            sql_query = sql_query.bind(identity_type.as_str());
        }
        if let Some(ref status) = filter.status {
            sql_query = sql_query.bind(format!("{:?}", status));
        }
        if let Some(ref parent_id) = filter.parent_id {
            sql_query = sql_query.bind(parent_id);
        }

        let rows = sql_query
            .fetch_all(pool)
            .await
            .map_err(|e| IdentityError::Storage(e.to_string()))?;

        let mut identities = Vec::new();
        for row in rows {
            identities.push(self.row_to_identity(&row)?);
        }

        Ok(identities)
    }

    fn row_to_identity(
        &self,
        row: &sqlx::postgres::PgRow,
    ) -> Result<IdentityContext> {
        use sqlx::Row;

        let id: String = row.get("id");
        let identity_type_str: String = row.get("identity_type");
        let name: String = row.get("name");
        let scope_json: serde_json::Value = row.get("scope");
        let capabilities_json: serde_json::Value = row.get("capabilities");
        let claims_json: serde_json::Value = row.get("claims");
        let security_level: i32 = row.get("security_level");
        let parent_id: Option<String> = row.get("parent_id");
        let generation: i32 = row.get("generation");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");
        let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
        let last_authenticated_at: Option<DateTime<Utc>> = row.get("last_authenticated_at");
        let status_str: String = row.get("status");
        let metadata_json: serde_json::Value = row.get("metadata");

        let identity_type = match identity_type_str.as_str() {
            "system" => IdentityType::System,
            "manager" => IdentityType::Manager,
            "worker" => IdentityType::Worker,
            "service" => IdentityType::Service,
            "ephemeral" => IdentityType::Ephemeral,
            _ => IdentityType::Worker,
        };

        let status = match status_str.as_str() {
            "ACTIVE" | "Active" => IdentityStatus::Active,
            "SUSPENDED" | "Suspended" => IdentityStatus::Suspended,
            "REVOKED" | "Revoked" => IdentityStatus::Revoked,
            "EXPIRED" | "Expired" => IdentityStatus::Expired,
            "PENDING" | "Pending" => IdentityStatus::Pending,
            _ => IdentityStatus::Active,
        };

        let scope: ScopedIdentity = serde_json::from_value(scope_json)?;
        let capabilities: Vec<Capability> = serde_json::from_value(capabilities_json)?;
        let claims: std::collections::HashMap<String, String> =
            serde_json::from_value(claims_json)?;
        let metadata: std::collections::HashMap<String, String> =
            serde_json::from_value(metadata_json)?;

        Ok(IdentityContext {
            id,
            identity_type,
            name,
            scope,
            capabilities,
            claims,
            security_level: match security_level {
                    0 => SecurityLevel::Minimal,
                    1 => SecurityLevel::Low,
                    2 => SecurityLevel::Medium,
                    3 => SecurityLevel::High,
                    4 => SecurityLevel::Critical,
                    _ => SecurityLevel::Medium, // default
                },
            parent_id,
            generation,
            created_at,
            updated_at,
            expires_at,
            last_authenticated_at,
            status,
            metadata,
        })
    }

    async fn persist_credential(
        &self,
        pool: &PgPool,
        credential: &IdentityCredential,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO identity_credentials (
                credential_id, identity_id, credential_type, data, salt,
                status, created_at, last_used_at, expires_at, max_uses,
                use_count, scopes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (credential_id) DO UPDATE SET
                status = EXCLUDED.status,
                last_used_at = EXCLUDED.last_used_at,
                use_count = EXCLUDED.use_count
            "#,
        )
        .bind(&credential.credential_id)
        .bind(&credential.identity_id)
        .bind(format!("{:?}", credential.credential_type))
        .bind(&credential.data)
        .bind(&credential.salt)
        .bind(format!("{:?}", credential.status))
        .bind(credential.created_at)
        .bind(credential.last_used_at)
        .bind(credential.expires_at)
        .bind(credential.max_uses.map(|v| v as i64))
        .bind(credential.use_count as i64)
        .bind(&credential.scopes)
        .execute(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn load_credential(
        &self,
        pool: &PgPool,
        credential_id: &str,
    ) -> Result<Option<IdentityCredential>> {
        let row = sqlx::query(
            r#"
            SELECT credential_id, identity_id, credential_type, data, salt,
                   status, created_at, last_used_at, expires_at, max_uses,
                   use_count, scopes
            FROM identity_credentials
            WHERE credential_id = $1
            "#,
        )
        .bind(credential_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        match row {
            Some(row) => {
                let credential_type_str: String = row.get("credential_type");
                let status_str: String = row.get("status");

                let credential_type = match credential_type_str.as_str() {
                    "PASSWORD" | "Password" => CredentialType::Password,
                    "PUBLIC_KEY" | "PublicKey" => CredentialType::PublicKey,
                    "API_KEY" | "ApiKey" => CredentialType::ApiKey,
                    "OAUTH_TOKEN" | "OAuthToken" => CredentialType::OAuthToken,
                    "JWT_TOKEN" | "JwtToken" => CredentialType::JwtToken,
                    "HARDWARE_TOKEN" | "HardwareToken" => CredentialType::HardwareToken,
                    "BIOMETRIC" | "Biometric" => CredentialType::Biometric,
                    "CERTIFICATE" | "Certificate" => CredentialType::Certificate,
                    _ => CredentialType::ApiKey,
                };

                let status = match status_str.as_str() {
                    "ACTIVE" | "Active" => CredentialStatus::Active,
                    "SUSPENDED" | "Suspended" => CredentialStatus::Suspended,
                    "EXPIRED" | "Expired" => CredentialStatus::Expired,
                    "REVOKED" | "Revoked" => CredentialStatus::Revoked,
                    "COMPROMISED" | "Compromised" => CredentialStatus::Compromised,
                    _ => CredentialStatus::Active,
                };

                Ok(Some(IdentityCredential {
                    credential_id: row.get("credential_id"),
                    identity_id: row.get("identity_id"),
                    credential_type,
                    data: row.get("data"),
                    salt: row.get("salt"),
                    status,
                    created_at: row.get("created_at"),
                    last_used_at: row.get("last_used_at"),
                    expires_at: row.get("expires_at"),
                    max_uses: row.get::<Option<i64>, _>("max_uses").map(|v| v as u64),
                    use_count: row.get::<i64, _>("use_count") as u64,
                    scopes: row.get("scopes"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn persist_escalation_token(
        &self,
        pool: &PgPool,
        token: &EscalationToken,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO escalation_tokens (
                token_id, identity_id, authorized_by, target_level, granted_capabilities,
                created_at, expires_at, reason, signature, used, used_at, single_use
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (token_id) DO UPDATE SET
                used = EXCLUDED.used,
                used_at = EXCLUDED.used_at
            "#,
        )
        .bind(&token.token_id)
        .bind(&token.identity_id)
        .bind(&token.authorized_by)
        .bind(token.target_level.as_u8() as i32)
        .bind(&token.granted_capabilities)
        .bind(token.created_at)
        .bind(token.expires_at)
        .bind(&token.reason)
        .bind(&token.signature)
        .bind(token.used)
        .bind(token.used_at)
        .bind(token.single_use)
        .execute(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn load_escalation_token(
        &self,
        pool: &PgPool,
        token_id: &str,
    ) -> Result<Option<EscalationToken>> {
        let row = sqlx::query(
            r#"
            SELECT token_id, identity_id, authorized_by, target_level, granted_capabilities,
                   created_at, expires_at, reason, signature, used, used_at, single_use
            FROM escalation_tokens
            WHERE token_id = $1
            "#,
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        match row {
            Some(row) => {
                let target_level: i32 = row.get("target_level");

                Ok(Some(EscalationToken {
                    token_id: row.get("token_id"),
                    identity_id: row.get("identity_id"),
                    authorized_by: row.get("authorized_by"),
                    target_level: match target_level {
                        0 => SecurityLevel::Minimal,
                        1 => SecurityLevel::Low,
                        2 => SecurityLevel::Medium,
                        3 => SecurityLevel::High,
                        4 => SecurityLevel::Critical,
                        _ => SecurityLevel::Medium,
                    },
                    granted_capabilities: row.get("granted_capabilities"),
                    created_at: row.get("created_at"),
                    expires_at: row.get("expires_at"),
                    reason: row.get("reason"),
                    signature: row.get("signature"),
                    used: row.get("used"),
                    used_at: row.get("used_at"),
                    single_use: row.get("single_use"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn persist_delegation_token(
        &self,
        pool: &PgPool,
        token: &DelegationToken,
    ) -> Result<()> {
        let scope_json = serde_json::to_value(&token.scope)?;

        sqlx::query(
            r#"
            INSERT INTO delegation_tokens (
                token_id, delegator_id, delegate_id, scope, delegated_capabilities,
                resource_restrictions, created_at, expires_at, reason, signature,
                active, revoked_at, revocation_reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (token_id) DO UPDATE SET
                active = EXCLUDED.active,
                revoked_at = EXCLUDED.revoked_at,
                revocation_reason = EXCLUDED.revocation_reason
            "#,
        )
        .bind(&token.token_id)
        .bind(&token.delegator_id)
        .bind(&token.delegate_id)
        .bind(scope_json)
        .bind(&token.delegated_capabilities)
        .bind(&token.resource_restrictions)
        .bind(token.created_at)
        .bind(token.expires_at)
        .bind(&token.reason)
        .bind(&token.signature)
        .bind(token.active)
        .bind(token.revoked_at)
        .bind(&token.revocation_reason)
        .execute(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn load_delegation_token(
        &self,
        pool: &PgPool,
        token_id: &str,
    ) -> Result<Option<DelegationToken>> {
        let row = sqlx::query(
            r#"
            SELECT token_id, delegator_id, delegate_id, scope, delegated_capabilities,
                   resource_restrictions, created_at, expires_at, reason, signature,
                   active, revoked_at, revocation_reason
            FROM delegation_tokens
            WHERE token_id = $1
            "#,
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| IdentityError::Storage(e.to_string()))?;

        match row {
            Some(row) => {
                let scope_json: serde_json::Value = row.get("scope");
                let scope: IdentityScope = serde_json::from_value(scope_json)?;

                Ok(Some(DelegationToken {
                    token_id: row.get("token_id"),
                    delegator_id: row.get("delegator_id"),
                    delegate_id: row.get("delegate_id"),
                    scope,
                    delegated_capabilities: row.get("delegated_capabilities"),
                    resource_restrictions: row.get("resource_restrictions"),
                    created_at: row.get("created_at"),
                    expires_at: row.get("expires_at"),
                    reason: row.get("reason"),
                    signature: row.get("signature"),
                    active: row.get("active"),
                    revoked_at: row.get("revoked_at"),
                    revocation_reason: row.get("revocation_reason"),
                }))
            }
            None => Ok(None),
        }
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, ScopedIdentity};

    #[tokio::test]
    async fn test_create_identity() {
        let manager = IdentityManager::new();
        let identity = manager
            .create_identity(
                IdentityType::Worker,
                "test-worker".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                None,
            )
            .await
            .unwrap();

        assert_eq!(identity.identity_type, IdentityType::Worker);
        assert_eq!(identity.name, "test-worker");
        assert_eq!(identity.generation, 0);
    }

    #[tokio::test]
    async fn test_hierarchy() {
        let manager = IdentityManager::new();

        let parent = manager
            .create_identity(
                IdentityType::Manager,
                "parent".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                None,
            )
            .await
            .unwrap();

        let child = manager
            .create_identity(
                IdentityType::Worker,
                "child".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                Some(parent.id.clone()),
            )
            .await
            .unwrap();

        assert_eq!(child.generation, 1);
        assert_eq!(child.parent_id, Some(parent.id.clone()));

        let children = manager.get_children(&parent.id).await;
        assert!(children.contains(&child.id));
    }

    #[tokio::test]
    async fn test_credential_lifecycle() {
        let manager = IdentityManager::new();

        let identity = manager
            .create_identity(
                IdentityType::Worker,
                "test".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                None,
            )
            .await
            .unwrap();

        let credential = manager
            .issue_credential(&identity.id, CredentialType::ApiKey, "secret123".to_string())
            .await
            .unwrap();

        let validated = manager.validate_credential(&credential.credential_id).await.unwrap();
        assert_eq!(validated.identity_id, identity.id);

        manager.revoke_credential(&credential.credential_id).await.unwrap();
        assert!(manager.validate_credential(&credential.credential_id).await.is_err());
    }

    #[tokio::test]
    async fn test_filter() {
        let manager = IdentityManager::new();

        manager
            .create_identity(
                IdentityType::Worker,
                "worker-1".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                None,
            )
            .await
            .unwrap();

        manager
            .create_identity(
                IdentityType::Manager,
                "manager-1".to_string(),
                ScopedIdentity::tenant("tenant-1"),
                None,
            )
            .await
            .unwrap();

        let filter = IdentityFilter::new().with_type(IdentityType::Worker);
        let results = manager.list_identities(&filter).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].identity_type, IdentityType::Worker);
    }
}
