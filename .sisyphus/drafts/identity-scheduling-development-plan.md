# SMA-OS 身份调度系统开发方案

**版本**: 1.0  
**日期**: 2026-03-14  
**状态**: 技术设计完成，待实现

---

## 执行摘要

本文档详细描述SMA-OS身份调度系统的完整实现方案，包括：
- 完整的代码结构设计
- 详细的API接口定义
- 具体的数据库schema变更
- 完整的测试策略
- 详细的实施时间表

**预期成果**：
- 6个实施阶段
- 8周开发周期
- 35+文件修改
- 1900+新增代码行

---

## 架构概览

### 身份调度系统架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        身份调度系统架构                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐          │
│  │   User/      │────▶│  Identity    │────▶│   Policy     │          │
│  │   Agent      │     │   Manager    │     │   Engine     │          │
│  └──────────────┘     └──────┬───────┘     └──────┬───────┘          │
│                              │                     │                    │
│                              ▼                     ▼                    │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐          │
│  │   Audit      │◀────│  Credential  │◀────│   Hierarchy  │          │
│  │   Logger     │     │   Store      │     │   Manager    │          │
│  └──────────────┘     └──────────────┘     └──────────────┘          │
│                                                                         │
│  ═══════════════════════════════════════════════════════════════════   │
│                         集成层                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │Orchestra- │ │Execution  │ │ Memory   │ │  Plugin  │ │Control   │   │
│  │  tion     │ │  Layer   │ │  Bus    │ │  System  │ │  Plane   │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 核心组件

| 组件 | 职责 | 技术选型 |
|------|------|----------|
| IdentityManager | 身份生命周期管理 | Rust + DashMap |
| CredentialStore | 凭证存储与验证 | Redis + PostgreSQL |
| PolicyEngine | 权限策略评估 | Rust |
| HierarchyManager | 身份层级关系 | PostgreSQL |
| AuditLogger | 审计日志记录 | PostgreSQL + ClickHouse |

---

## Phase 0: 基础设施（第1周）

### 0.1 核心数据结构设计

#### 0.1.1 IdentityContext（跨语言共享类型）

**Go版本** - `orchestration/types/identity.go`

```go
package types

import (
    "time"
)

// IdentityType 身份类型枚举
type IdentityType string

const (
    IdentityTypeSystem       IdentityType = "SYSTEM"       // 系统身份
    IdentityTypeManager     IdentityType = "MANAGER"      // 管理身份
    IdentityTypeWorker     IdentityType = "WORKER"       // 工作身份
    IdentityTypeService    IdentityType = "SERVICE"      // 服务账号
    IdentityTypeEphemeral  IdentityType = "EPHEMERAL"    // 临时身份
)

// IdentityScope 权限范围
type IdentityScope struct {
    Level    ScopeLevel    // Global/Tenant/Namespace/Resource
    TenantID string        // 租户ID
    Namespace string       // 命名空间
    ResourceID string      // 资源ID
}

type ScopeLevel string

const (
    ScopeGlobal    ScopeLevel = "GLOBAL"
    ScopeTenant    ScopeLevel = "TENANT"
    ScopeNamespace ScopeLevel = "NAMESPACE"
    ScopeResource  ScopeLevel = "RESOURCE"
)

// Capability 能力声明
type Capability struct {
    Name        string            `json:"name"`
    Category    string            `json:"category"`     // execute/storage/network/manage
    Actions     []string          `json:"actions"`     // 操作列表
    ResourcePatt string           `json:"resource_pattern"` // 资源匹配模式
}

// IdentityContext 身份上下文（跨模块传递）
type IdentityContext struct {
    // 身份标识
    ID         string       `json:"id"`
    Type       IdentityType `json:"type"`
    Name       string       `json:"name"`
    
    // 权限范围
    Scope      IdentityScope `json:"scope"`
    
    // 能力声明
    Capabilities []Capability `json:"capabilities"`
    
    // 额外声明
    Claims map[string]string `json:"claims"`
    
    // 安全等级 (1-100)
    SecurityLevel int `json:"security_level"`
    
    // 血缘关系
    ParentID   *string `json:"parent_id,omitempty"`
    Generation int    `json:"generation"` // 世代号
    
    // 时间戳
    CreatedAt  time.Time `json:"created_at"`
    ExpiresAt  *time.Time `json:"expires_at,omitempty"`
    
    // 状态
    Status IdentityStatus `json:"status"`
}

type IdentityStatus string

const (
    IdentityStatusActive     IdentityStatus = "ACTIVE"
    IdentityStatusSuspended  IdentityStatus = "SUSPENDED"
    IdentityStatusRevoked    IdentityStatus = "REVOKED"
    IdentityStatusExpired    IdentityStatus = "EXPIRED"
)

// IdentityCredential 身份凭证（用于验证）
type IdentityCredential struct {
    IdentityID   string    `json:"identity_id"`
    Token        string    `json:"token"`         // JWT或签名token
    IssuedAt     time.Time `json:"issued_at"`
    ExpiresAt    time.Time `json:"expires_at"`
    Scopes       []string  `json:"scopes"`       // 授权范围
}

// EscalationToken 特权升级令牌
type EscalationToken struct {
    TokenID       string    `json:"token_id"`
    IdentityID    string    `json:"identity_id"`
    Capability    string    `json:"capability"`
    GrantedAt     time.Time `json:"granted_at"`
    ExpiresAt     time.Time `json:"expires_at"`
    Reason        string    `json:"reason"`
}

// DelegationToken 委托令牌
type DelegationToken struct {
    TokenID       string    `json:"token_id"`
    DelegatorID    string    `json:"delegator_id"`
    DelegateeID    string    `json:"delegatee_id"`
    Scopes        []string  `json:"scopes"`
    GrantedAt     time.Time `json:"granted_at"`
    ExpiresAt     time.Time `json:"expires_at"`
}
```

#### 0.1.2 Rust版本 - `control-plane/identity/src/types.rs`

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

// 身份类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityType {
    System,
    Manager,
    Worker,
    Service,
    Ephemeral,
}

// 权限范围级别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeLevel {
    Global,
    Tenant,
    Namespace,
    Resource,
}

// 权限范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityScope {
    pub level: ScopeLevel,
    pub tenant_id: Option<String>,
    pub namespace: Option<String>,
    pub resource_id: Option<String>,
}

// 能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub category: String,
    pub actions: Vec<String>,
    pub resource_pattern: Option<String>,
}

// 身份上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityContext {
    pub id: String,
    pub identity_type: IdentityType,
    pub name: String,
    pub scope: IdentityScope,
    pub capabilities: Vec<Capability>,
    pub claims: HashMap<String, String>,
    pub security_level: u8,
    pub parent_id: Option<String>,
    pub generation: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: IdentityStatus,
}

// 身份状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityStatus {
    Active,
    Suspended,
    Revoked,
    Expired,
}

// 身份凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCredential {
    pub identity_id: String,
    pub token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

// 特权升级令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationToken {
    pub token_id: String,
    pub identity_id: String,
    pub capability: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reason: String,
}

// 委托令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationToken {
    pub token_id: String,
    pub delegator_id: String,
    pub delegatee_id: String,
    pub scopes: Vec<String>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

#### 0.1.3 Protobuf定义 - `sma-proto/identity.proto`

```protobuf
syntax = "proto3";

package sma.identity;

option go_package = "sma-os/sma-proto/identity";

// 身份类型
enum IdentityType {
    IDENTITY_TYPE_UNSPECIFIED = 0;
    IDENTITY_TYPE_SYSTEM = 1;
    IDENTITY_TYPE_MANAGER = 2;
    IDENTITY_TYPE_WORKER = 3;
    IDENTITY_TYPE_SERVICE = 4;
    IDENTITY_TYPE_EPHEMERAL = 5;
}

// 权限范围级别
enum ScopeLevel {
    SCOPE_LEVEL_UNSPECIFIED = 0;
    SCOPE_LEVEL_GLOBAL = 1;
    SCOPE_LEVEL_TENANT = 2;
    SCOPE_LEVEL_NAMESPACE = 3;
    SCOPE_LEVEL_RESOURCE = 4;
}

// 权限范围
message IdentityScope {
    ScopeLevel level = 1;
    string tenant_id = 2;
    string namespace = 3;
    string resource_id = 4;
}

// 能力声明
message Capability {
    string name = 1;
    string category = 2;
    repeated string actions = 3;
    string resource_pattern = 4;
}

// 身份上下文
message IdentityContext {
    string id = 1;
    IdentityType type = 2;
    string name = 3;
    IdentityScope scope = 4;
    repeated Capability capabilities = 5;
    map<string, string> claims = 6;
    uint32 security_level = 7;
    string parent_id = 8;
    int32 generation = 9;
    string created_at = 10;
    string expires_at = 11;
    IdentityStatus status = 12;
}

// 身份状态
enum IdentityStatus {
    IDENTITY_STATUS_UNSPECIFIED = 0;
    IDENTITY_STATUS_ACTIVE = 1;
    IDENTITY_STATUS_SUSPENDED = 2;
    IDENTITY_STATUS_REVOKED = 3;
    IDENTITY_STATUS_EXPIRED = 4;
}

// 身份凭证
message IdentityCredential {
    string identity_id = 1;
    string token = 2;
    string issued_at = 3;
    string expires_at = 4;
    repeated string scopes = 5;
}

// 身份服务
service IdentityService {
    // 身份管理
    rpc CreateIdentity(CreateIdentityRequest) returns (IdentityContext);
    rpc GetIdentity(GetIdentityRequest) returns (IdentityContext);
    rpc UpdateIdentity(UpdateIdentityRequest) returns (IdentityContext);
    rpc DeleteIdentity(DeleteIdentityRequest) returns (DeleteIdentityResponse);
    rpc ListIdentities(ListIdentitiesRequest) returns (ListIdentitiesResponse);
    
    // 凭证管理
    rpc IssueCredential(IssueCredentialRequest) returns (IdentityCredential);
    rpc ValidateCredential(ValidateCredentialRequest) returns (ValidateCredentialResponse);
    rpc RevokeCredential(RevokeCredentialRequest) returns (RevokeCredentialResponse);
    
    // 特权操作
    rpc Escalate(EscalationRequest) returns (EscalationToken);
    rpc DeEscalate(DeEscalateRequest) returns (DeEscalateResponse);
    
    // 委托
    rpc Delegate(DelegationRequest) returns (DelegationToken);
    rpc RevokeDelegation(RevokeDelegationRequest) returns (RevokeDelegationResponse);
    
    // 层级管理
    rpc CreateChild(CreateChildRequest) returns (IdentityContext);
    rpc GetChildren(GetChildrenRequest) returns (GetChildrenResponse);
    rpc CascadeRevoke(CascadeRevokeRequest) returns (CascadeRevokeResponse);
}
```

---

### 0.2 身份管理器核心实现

#### 0.2.1 Rust实现 - `control-plane/identity/src/manager.rs`

```rust
use crate::types::*;
use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use chrono::Utc;
use uuid::Uuid;
use thiserror::Error;

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
}

pub struct IdentityManager {
    identities: Arc<DashMap<String, IdentityContext>>,
    credentials: Arc<DashMap<String, IdentityCredential>>,
    escalation_tokens: Arc<DashMap<String, EscalationToken>>,
    delegation_tokens: Arc<DashMap<String, DelegationToken>>,
    hierarchy: Arc<DashMap<String, Vec<String>>>, // parent -> children
    credential_store: Arc<RwLock<CredentialStore>>,
}

impl IdentityManager {
    pub fn new() -> Self {
        Self {
            identities: Arc::new(DashMap::new()),
            credentials: Arc::new(DashMap::new()),
            escalation_tokens: Arc::new(DashMap::new()),
            delegation_tokens: Arc::new(DashMap::new()),
            hierarchy: Arc::new(DashMap::new()),
            credential_store: Arc::new(RwLock::new(CredentialStore::new())),
        }
    }
    
    /// 创建系统身份（启动时调用）
    pub fn create_system_identity(&self, name: &str) -> Result<IdentityContext, IdentityError> {
        let id = format!("system:{}", name);
        
        if self.identities.contains_key(&id) {
            return Err(IdentityError::AlreadyExists(id));
        }
        
        let identity = IdentityContext {
            id: id.clone(),
            identity_type: IdentityType::System,
            name: name.to_string(),
            scope: IdentityScope {
                level: ScopeLevel::Global,
                tenant_id: None,
                namespace: None,
                resource_id: None,
            },
            capabilities: self.get_system_capabilities(name),
            claims: std::collections::HashMap::new(),
            security_level: 100,
            parent_id: None,
            generation: 0,
            created_at: Utc::now(),
            expires_at: None,
            status: IdentityStatus::Active,
        };
        
        self.identities.insert(id, identity.clone());
        info!("[Identity] System identity created: {}", identity.id);
        
        Ok(identity)
    }
    
    /// 创建普通身份
    pub fn create_identity(
        &self,
        identity_type: IdentityType,
        name: &str,
        scope: IdentityScope,
        parent_id: Option<&str>,
    ) -> Result<IdentityContext, IdentityError> {
        if let Some(pid) = parent_id {
            self.get_identity(pid)?;
        }
        
        let id = format!("{}:{}:{}", 
            identity_type.as_str(), 
            name, 
            Uuid::new_v4().to_string()[..8].to_string()
        );
        
        let generation = if let Some(pid) = parent_id {
            self.get_generation(pid) + 1
        } else {
            0
        };
        
        let identity = IdentityContext {
            id: id.clone(),
            identity_type: identity_type.clone(),
            name: name.to_string(),
            scope,
            capabilities: vec![],
            claims: std::collections::HashMap::new(),
            security_level: 50,
            parent_id: parent_id.map(|s| s.to_string()),
            generation,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
            status: IdentityStatus::Active,
        };
        
        self.identities.insert(id.clone(), identity.clone());
        
        if let Some(pid) = parent_id {
            self.hierarchy.entry(pid.to_string())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        info!("[Identity] Identity created: {} (parent: {:?})", id, parent_id);
        
        Ok(identity)
    }
    
    /// 获取身份
    pub fn get_identity(&self, id: &str) -> Result<IdentityContext, IdentityError> {
        self.identities
            .get(id)
            .map(|i| i.clone())
            .ok_or(IdentityError::NotFound(id.to_string()))
    }
    
    /// 撤销身份
    pub fn revoke_identity(&self, id: &str, reason: &str) -> Result<(), IdentityError> {
        let mut identity = self.get_identity(id)?;
        
        let children = self.get_children(id);
        for child_id in children {
            self.revoke_identity(&child_id, &format!("parent revoked: {}", id))?;
        }
        
        identity.status = IdentityStatus::Revoked;
        self.identities.insert(id.to_string(), identity);
        self.revoke_all_credentials(id);
        
        info!("[Identity] Identity revoked: {} (reason: {})", id, reason);
        
        Ok(())
    }
    
    /// 特权升级
    pub fn escalate(
        &self,
        identity_id: &str,
        capability: &str,
        reason: &str,
        duration_secs: u32,
    ) -> Result<EscalationToken, IdentityError> {
        let identity = self.get_identity(identity_id)?;
        
        let has_capability = identity.capabilities.iter()
            .any(|c| c.name == capability);
        
        if !has_capability {
            return Err(IdentityError::EscalationDenied(
                format!("identity {} does not have capability {}", identity_id, capability)
            ));
        }
        
        let token_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let token = EscalationToken {
            token_id: token_id.clone(),
            identity_id: identity_id.to_string(),
            capability: capability.to_string(),
            granted_at: now,
            expires_at: now + chrono::Duration::seconds(duration_secs as i64),
            reason: reason.to_string(),
        };
        
        self.escalation_tokens.insert(token_id, token.clone());
        
        info!("[Identity] Escalation granted: {} -> {} (reason: {})", 
            identity_id, capability, reason);
        
        Ok(token)
    }
    
    /// 委托身份
    pub fn delegate(
        &self,
        delegator_id: &str,
        delegatee_id: &str,
        scopes: Vec<String>,
        duration_secs: u32,
    ) -> Result<DelegationToken, IdentityError> {
        if !self.is_ancestor(delegator_id, delegatee_id) && 
           !self.is_descendant(delegator_id, delegatee_id) {
            return Err(IdentityError::NotInHierarchy {
                ancestor: delegator_id.to_string(),
                descendant: delegatee_id.to_string(),
            });
        }
        
        let token_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let token = DelegationToken {
            token_id: token_id.clone(),
            delegator_id: delegator_id.to_string(),
            delegatee_id: delegatee_id.to_string(),
            scopes,
            granted_at: now,
            expires_at: now + chrono::Duration::seconds(duration_secs as i64),
        };
        
        self.delegation_tokens.insert(token_id, token.clone());
        
        info!("[Identity] Delegation: {} -> {} (scopes: {:?})", 
            delegator_id, delegatee_id, token.scopes);
        
        Ok(token)
    }
    
    /// 获取子身份列表
    pub fn get_children(&self, parent_id: &str) -> Vec<String> {
        self.hierarchy
            .get(parent_id)
            .map(|c| c.clone())
            .unwrap_or_default()
    }
    
    /// 获取世代号
    fn get_generation(&self, id: &str) -> i32 {
        self.identities.get(id)
            .map(|i| i.generation)
            .unwrap_or(0)
    }
    
    /// 检查祖先关系
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> bool {
        let mut current = descendant;
        while let Some(parent_id) = self.identities.get(current)
            .and_then(|i| i.parent_id.clone()) 
        {
            if parent_id == ancestor {
                return true;
            }
            current = &parent_id;
        }
        false
    }
    
    fn is_descendant(&self, descendant: &str, ancestor: &str) -> bool {
        self.is_ancestor(ancestor, descendant)
    }
    
    fn revoke_all_credentials(&self, identity_id: &str) {
        let tokens_to_revoke: Vec<String> = self.credentials
            .iter()
            .filter(|c| c.identity_id == identity_id)
            .map(|c| c.token.clone())
            .collect();
        
        for token in tokens_to_revoke {
            self.credentials.remove(&token);
        }
    }
    
    fn get_system_capabilities(&self, name: &str) -> Vec<Capability> {
        match name {
            "orchestrator" => vec![
                Capability {
                    name: "task.create".to_string(),
                    category: "manage".to_string(),
                    actions: vec!["create".to_string(), "read".to_string(), "update".to_string()],
                    resource_pattern: None,
                },
            ],
            "state-engine" => vec![
                Capability {
                    name: "event.append".to_string(),
                    category: "execute".to_string(),
                    actions: vec!["append".to_string()],
                    resource_pattern: None,
                },
            ],
            _ => vec![],
        }
    }
}

// 辅助 trait
impl IdentityType {
    pub fn as_str(&self) -> &str {
        match self {
            IdentityType::System => "system",
            IdentityType::Manager => "manager",
            IdentityType::Worker => "worker",
            IdentityType::Service => "service",
            IdentityType::Ephemeral => "ephemeral",
        }
    }
}
```

#### 0.2.2 Go实现 - `orchestration/types/identity.go`

```go
package types

import (
    "time"
    "fmt"
    "github.com/google/uuid"
)

// IdentityManager 身份管理器
type IdentityManager struct {
    identities       map[string]*IdentityContext
    credentials     map[string]*IdentityCredential
    escalationTokens map[string]*EscalationToken
    delegationTokens map[string]*DelegationToken
    hierarchy       map[string][]string
}

func NewIdentityManager() *IdentityManager {
    return &IdentityManager{
        identities:        make(map[string]*IdentityContext),
        credentials:       make(map[string]*IdentityCredential),
        escalationTokens:  make(map[string]*EscalationToken),
        delegationTokens:  make(map[string]*DelegationToken),
        hierarchy:         make(map[string][]string),
    }
}

func (m *IdentityManager) CreateIdentity(
    identityType IdentityType,
    name string,
    scope IdentityScope,
    parentID *string,
) (*IdentityContext, error) {
    id := fmt.Sprintf("%s:%s:%s", identityType, name, uuid.New().String()[:8])
    
    generation := 0
    if parentID != nil {
        if parent, ok := m.identities[*parentID]; ok {
            generation = parent.Generation + 1
        }
    }
    
    identity := &IdentityContext{
        ID:            id,
        Type:          identityType,
        Name:          name,
        Scope:         scope,
        Capabilities:  []Capability{},
        Claims:        make(map[string]string),
        SecurityLevel: 50,
        ParentID:      parentID,
        Generation:    generation,
        CreatedAt:     time.Now(),
        Status:        IdentityStatusActive,
    }
    
    exp := time.Now().Add(24 * time.Hour)
    identity.ExpiresAt = &exp
    
    m.identities[id] = identity
    
    if parentID != nil {
        m.hierarchy[*parentID] = append(m.hierarchy[*parentID], id)
    }
    
    return identity, nil
}

func (m *IdentityManager) GetIdentity(id string) (*IdentityContext, error) {
    if identity, ok := m.identities[id]; ok {
        return identity, nil
    }
    return nil, fmt.Errorf("identity not found: %s", id)
}

func (m *IdentityManager) IssueCredential(
    identityID string,
    scopes []string,
    validitySecs int32,
) (*IdentityCredential, error) {
    identity, err := m.GetIdentity(identityID)
    if err != nil {
        return nil, err
    }
    
    if identity.Status != IdentityStatusActive {
        return nil, fmt.Errorf("identity suspended")
    }
    
    token := uuid.New().String()
    now := time.Now()
    
    credential := &IdentityCredential{
        IdentityID: identityID,
        Token:      token,
        IssuedAt:   now,
        ExpiresAt:  now.Add(time.Duration(validitySecs) * time.Second),
        Scopes:     scopes,
    }
    
    m.credentials[token] = credential
    return credential, nil
}

func (m *IdentityManager) ValidateCredential(token string) (*IdentityCredential, error) {
    cred, ok := m.credentials[token]
    if !ok {
        return nil, fmt.Errorf("invalid credential")
    }
    
    if time.Now().After(cred.ExpiresAt) {
        return nil, fmt.Errorf("credential expired")
    }
    
    return cred, nil
}

func (m *IdentityManager) Escalate(
    identityID string,
    capability string,
    reason string,
    durationSecs int32,
) (*EscalationToken, error) {
    identity, err := m.GetIdentity(identityID)
    if err != nil {
        return nil, err
    }
    
    hasCapability := false
    for _, cap := range identity.Capabilities {
        if cap.Name == capability {
            hasCapability = true
            break
        }
    }
    
    if !hasCapability {
        return nil, fmt.Errorf("escalation denied")
    }
    
    tokenID := uuid.New().String()
    now := time.Now()
    
    token := &EscalationToken{
        TokenID:    tokenID,
        IdentityID: identityID,
        Capability: capability,
        GrantedAt:  now,
        ExpiresAt:  now.Add(time.Duration(durationSecs) * time.Second),
        Reason:     reason,
    }
    
    m.escalationTokens[tokenID] = token
    return token, nil
}

func (m *IdentityManager) Delegate(
    delegatorID string,
    delegateeID string,
    scopes []string,
    durationSecs int32,
) (*DelegationToken, error) {
    if !m.isAncestor(delegatorID, delegateeID) && 
       !m.isDescendant(delegatorID, delegateeID) {
        return nil, fmt.Errorf("not in hierarchy")
    }
    
    tokenID := uuid.New().String()
    now := time.Now()
    
    token := &DelegationToken{
        TokenID:     tokenID,
        DelegatorID: delegatorID,
        DelegateeID: delegateeID,
        Scopes:      scopes,
        GrantedAt:   now,
        ExpiresAt:   now.Add(time.Duration(durationSecs) * time.Second),
    }
    
    m.delegationTokens[tokenID] = token
    return token, nil
}

func (m *IdentityManager) RevokeIdentity(identityID, reason string) error {
    identity, err := m.GetIdentity(identityID)
    if err != nil {
        return err
    }
    
    children := m.GetChildren(identityID)
    for _, childID := range children {
        m.RevokeIdentity(childID, "parent revoked: "+identityID)
    }
    
    identity.Status = IdentityStatusRevoked
    m.revokeAllCredentials(identityID)
    
    return nil
}

func (m *IdentityManager) GetChildren(parentID string) []string {
    return m.hierarchy[parentID]
}

func (m *IdentityManager) isAncestor(ancestor, descendant string) bool {
    current := descendant
    for {
        parentID, ok := m.identities[current]
        if !ok || parentID.ParentID == nil {
            break
        }
        if *parentID.ParentID == ancestor {
            return true
        }
        current = *parentID.ParentID
    }
    return false
}

func (m *IdentityManager) isDescendant(descendant, ancestor string) bool {
    return m.isAncestor(ancestor, descendant)
}

func (m *IdentityManager) revokeAllCredentials(identityID string) {
    for token, cred := range m.credentials {
        if cred.IdentityID == identityID {
            delete(m.credentials, token)
        }
    }
}
```

---

### 0.3 审计日志设计

#### 0.3.1 PostgreSQL表结构

```sql
-- 身份审计日志表
CREATE TABLE IF NOT EXISTS identity_audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    identity_id     VARCHAR(255) NOT NULL,
    identity_type   VARCHAR(50) NOT NULL,
    action          VARCHAR(100) NOT NULL,
    target_id       VARCHAR(255),
    target_type     VARCHAR(50),
    success         BOOLEAN NOT NULL,
    error_message   TEXT,
    ip_address      INET,
    user_agent      VARCHAR(500),
    request_id      UUID,
    metadata        JSONB,
    
    INDEX idx_identity_audit_identity_id (identity_id),
    INDEX idx_identity_audit_timestamp (timestamp),
    INDEX idx_identity_audit_action (action)
);
```

---

### 0.4 Feature Flag系统

```go
// orchestration/config/feature_flags.go

type FeatureFlag string

const (
    FlagIdentityScheduling      FeatureFlag = "identity_scheduling"
    FlagIdentityEscalation      FeatureFlag = "identity_escalation"
    FlagIdentityDelegation      FeatureFlag = "identity_delegation"
    FlagIdentityAudit           FeatureFlag = "identity_audit"
    FlagIdentityStrict          FeatureFlag = "identity_strict_mode"
)

type FeatureFlags struct {
    flags map[FeatureFlag]bool
    mutex sync.RWMutex
}

var globalFlags = &FeatureFlags{
    flags: map[FeatureFlag]bool{
        FlagIdentityScheduling: false,  // 默认关闭
        FlagIdentityEscalation: false,
        FlagIdentityDelegation: false,
        FlagIdentityAudit:      true,   // 审计默认开启
        FlagIdentityStrict:     false,
    },
}

func (f *FeatureFlags) IsEnabled(flag FeatureFlag) bool {
    f.mutex.RLock()
    defer f.mutex.RUnlock()
    return f.flags[flag]
}

func IsIdentitySchedulingEnabled() bool {
    return globalFlags.IsEnabled(FlagIdentityScheduling)
}
```

---

## Phase 1: Control Plane增强（第1-2周）

### 1.1 Fractal Gateway身份集成

```rust
// control-plane/fractal-gateway/src/security.rs

pub struct SecurityPolicy {
    pub agent_id: String,
    pub tenant_id: String,              // 新增
    pub namespace: String,               // 新增
    pub identity_scope: IdentityScope,  // 新增
    pub seccomp_profile: SeccompProfile,
    pub dynamic_quotas: DynamicQuotas,
}

impl SecurityPolicy {
    pub fn new(agent_id: &str, tenant_id: &str, namespace: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            tenant_id: tenant_id.to_string(),
            namespace: namespace.to_string(),
            identity_scope: IdentityScope::Namespace(tenant_id.to_string(), namespace.to_string()),
            seccomp_profile: SeccompProfile::default(),
            dynamic_quotas: DynamicQuotas::default(),
        }
    }
    
    pub fn matches_identity(&self, identity: &IdentityContext) -> bool {
        match &self.identity_scope {
            IdentityScope::All => true,
            IdentityScope::Tenant(t) => identity.scope.tenant_id.as_ref() == Some(t),
            IdentityScope::Namespace(t, n) => {
                identity.scope.tenant_id.as_ref() == Some(t) &&
                identity.scope.namespace.as_ref() == Some(n)
            }
            IdentityScope::Agent(a) => identity.id == *a,
        }
    }
}
```

---

## Phase 2: Orchestration核心（第2-4周）

### 2.1 TaskNode扩展

```go
// orchestration/manager/main.go

type TaskNode struct {
    ID           string
    ActionName   string
    Dependencies []string
    Status       TaskStatus
    Payload      string
    // 新增 身份字段
    OwnerID              string
    PrincipalID          string
    AuthorizationContext  string
    IdentityClaims       map[string]string
    SecurityLevel        int
    RequiredCapabilities []string
}

type TaskResult struct {
    TaskID    string
    Status    TaskStatus
    Error     error
    StartTime time.Time
    EndTime   time.Time
    RetryCnt  int
    // 新增
    ExecutedBy   string
    WorkerID     string
}
```

### 2.2 Worker调度扩展

```go
type WorkerNode struct {
    ID        string
    Type      WorkerType
    NodeHost  string
    Available bool
    Health    WorkerHealth
    // 新增
    AuthorizedPrincipals []string
    AllowedTenants       []string
    SecurityLevel        int
    MinSecurityLevel     int
    Capabilities         []string
}

func (s *FractalClusterScheduler) AssignTask(
    taskID string,
    identity *types.IdentityContext,
    previousHost string,
) (string, error) {
    // 1. 验证身份授权
    if err := s.validateIdentityForTask(identity, task); err != nil {
        return "", err
    }
    
    // 2. 选择Worker
    workerID := s.selectWorker(identity, task, previousHost)
    
    // 3. 标记占用
    s.Workers[workerID].Available = false
    
    return workerID, nil
}

func (s *FractalClusterScheduler) validateIdentityForTask(
    identity *types.IdentityContext,
    task *TaskNode,
) error {
    if task.SecurityLevel > identity.SecurityLevel {
        return fmt.Errorf("insufficient security level")
    }
    
    for _, reqCap := range task.RequiredCapabilities {
        hasCap := false
        for _, cap := range identity.Capabilities {
            if cap == reqCap {
                hasCap = true
                break
            }
        }
        if !hasCap {
            return fmt.Errorf("missing capability: %s", reqCap)
        }
    }
    
    return nil
}
```

---

## Phase 3: Execution Layer（第4-5周）

### 3.1 Sandbox Daemon扩展

```rust
// execution-layer/sandbox-daemon/src/microvm.rs

pub struct FirecrackerVM {
    pub vm_id: String,
    // 新增 身份绑定
    pub identity_id: Option<String>,
    pub agent_id: Option<String>,
    pub tenant_id: Option<String>,
    pub namespace: Option<String>,
    
    pub socket_path: String,
    pub config: VmConfig,
    pub state: VmState,
    pub pid: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
}

impl FirecrackerVM {
    pub fn bind_identity(
        &mut self, 
        identity_id: String,
        tenant_id: String,
        namespace: String,
    ) -> Result<(), VmError> {
        self.identity_id = Some(identity_id);
        self.tenant_id = Some(tenant_id);
        self.namespace = Some(namespace);
        self.assigned_at = Some(Utc::now());
        
        info!("[VM] {} bound to identity {} (tenant: {})", 
            self.vm_id, identity_id, tenant_id);
        
        Ok(())
    }
}
```

### 3.2 gRPC协议变更

```protobuf
// sma-proto/sandbox.proto

message AssignRequest {
    string tenant_id = 1;
    string namespace = 2;
    string action_name = 3;
    string payload_json = 4;
    // 新增
    string identity_id = 5;
    string principal_id = 6;
    string credential_token = 7;
    repeated string scopes = 8;
}
```

---

## Phase 4: Memory Bus（第5-6周）

### 4.1 Ingestion扩展

```go
// memory-bus/ingestion/main.go

func ProcessInputWithIdentity(
    userInput string,
    identity types.IdentityContext,
    cacheManager *cache.CacheManager,
) (*ParsedIntent, error) {
    // 验证身份
    if identity.SecurityLevel < 10 {
        return nil, fmt.Errorf("insufficient security level")
    }
    
    // 身份增强缓存key
    cacheKey := generateCacheKeyWithIdentity(identity, userInput)
    
    // 身份增强prompt
    prompt := fmt.Sprintf(
        "[Context: caller=%s, tenant=%s, role=%s] %s",
        identity.ID,
        identity.Scope.TenantID,
        identity.Claims["role"],
        userInput,
    )
    
    return invokeLLM(prompt)
}

func generateCacheKeyWithIdentity(identity types.IdentityContext, input string) string {
    composite := fmt.Sprintf("%s:%s:%s", 
        identity.Scope.TenantID,
        identity.ID,
        input,
    )
    hash := sha256.Sum256([]byte(composite))
    return fmt.Sprintf("intent:%x", hash)
}
```

---

## Phase 5: Plugin系统（第6-7周）

### 5.1 权限模型扩展

```rust
// plugins/core/src/manifest.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityScope {
    All,                    // 所有人可访问
    Tenant(String),         // 特定tenant
    Identity(Vec<String>),  // 特定身份列表
    Role(String),           // 角色
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub resource: String,
    pub actions: Vec<String>,
    pub identity_scope: IdentityScope,  // 新增
}
```

---

## Phase 6: 测试与发布（第7-8周）

### 6.1 测试策略

```rust
// control-plane/identity/src/manager_test.rs

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_create_and_get_identity() {
        let manager = IdentityManager::new();
        
        let identity = manager.create_identity(
            IdentityType::Worker,
            "test-worker",
            IdentityScope {
                level: ScopeLevel::Tenant,
                tenant_id: Some("tenant-1".to_string()),
                namespace: None,
                resource_id: None,
            },
            None,
        ).unwrap();
        
        let retrieved = manager.get_identity(&identity.id).unwrap();
        
        assert_eq!(identity.id, retrieved.id);
    }
    
    #[tokio::test]
    async fn test_escalation() {
        let manager = IdentityManager::new();
        
        let identity = manager.create_identity(
            IdentityType::Worker,
            "privileged-worker",
            IdentityScope::default(),
            None,
        ).unwrap();
        
        // 授予能力
        let mut updated = identity.clone();
        updated.capabilities = vec![Capability {
            name: "admin.execute".to_string(),
            category: "manage".to_string(),
            actions: vec!["execute".to_string()],
            resource_pattern: None,
        }];
        manager.update_identity(&identity.id, updated).unwrap();
        
        let token = manager.escalate(
            &identity.id,
            "admin.execute",
            "need admin for task",
            300,
        ).unwrap();
        
        assert!(token.token_id.len() > 0);
    }
    
    #[tokio::test]
    async fn test_hierarchy_cascade_revoke() {
        let manager = IdentityManager::new();
        
        let parent = manager.create_identity(
            IdentityType::Manager,
            "parent",
            IdentityScope::default(),
            None,
        ).unwrap();
        
        let _child = manager.create_child(
            &parent.id,
            "child",
            IdentityType::Worker,
        ).unwrap();
        
        manager.revoke_identity(&parent.id, "test").unwrap();
        
        let children = manager.get_children(&parent.id);
        for child_id in children {
            let child = manager.get_identity(&child_id).unwrap();
            assert_eq!(child.status, IdentityStatus::Revoked);
        }
    }
}
```

---

## 实施时间表

| 周次 | 阶段 | 任务 |
|------|------|------|
| 第1周 | 基础设施 | IdentityContext定义、IdentityManager核心、审计日志、Feature Flag |
| 第2周 | Control Plane | SecurityPolicy扩展、eBPF quota增强、State Engine集成 |
| 第3周 | Orchestration-Mgr | TaskNode扩展、双格式JSON解析、DAG执行流修改 |
| 第4周 | Orchestration-Scheduler | WorkerNode扩展、重写AssignTask、身份过滤 |
| 第5周 | Execution Layer | FirecrackerVM修改、gRPC协议更新、WarmPool重构 |
| 第6周 | Memory Bus & Plugin | Ingestion API扩展、缓存key变更、Plugin权限模型 |
| 第7周 | 测试 | 单元测试、集成测试、E2E测试 |
| 第8周 | 发布 | 性能调优、文档更新、渐进开启 |

---

## 成功标准

### 功能验收
- 任务提交必须携带身份
- Worker分配必须验证身份
- VM创建必须绑定身份
- Plugin执行必须检查权限
- 完整审计日志

### 性能验收
- 身份创建 <5ms
- 凭证验证 <1ms
- 调度身份过滤 <10ms

### 兼容性验收
- 旧API向后兼容
- 零停机迁移
- Feature Flag可控

---

**文档版本**: 1.0  
**下次更新**: 开始实现时
