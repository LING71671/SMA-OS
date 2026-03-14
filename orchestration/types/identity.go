// Package types provides shared types for the SMA-OS orchestration layer.
//
// This package contains identity management types that mirror the Rust
// implementation in the control plane, enabling cross-language compatibility.
//
// Key types:
//   - IdentityContext: Core identity structure
//   - IdentityType: Enum for identity categories
//   - IdentityScope: Access scope definitions
//   - Capability: Fine-grained permission declarations
//
// Example usage:
//
//	identity := types.NewIdentityContext(
//	    "worker-1",
//	    types.IdentityTypeWorker,
//	    "data-processor",
//	    types.NewTenantScope("tenant-a"),
//	).WithSecurityLevel(75).
//	    WithClaim("role", "admin")
package types

import (
	"fmt"
	"time"

	"github.com/google/uuid"
)

// IdentityType represents the category of an identity.
type IdentityType string

const (
	// IdentityTypeSystem is for system-level identities (orchestrator, state-engine)
	IdentityTypeSystem IdentityType = "SYSTEM"

	// IdentityTypeManager is for manager-level identities (DAG manager, scheduler)
	IdentityTypeManager IdentityType = "MANAGER"

	// IdentityTypeWorker is for worker-level identities
	IdentityTypeWorker IdentityType = "WORKER"

	// IdentityTypeService is for service account identities (plugins, external services)
	IdentityTypeService IdentityType = "SERVICE"

	// IdentityTypeEphemeral is for short-lived identities
	IdentityTypeEphemeral IdentityType = "EPHEMERAL"
)

// String returns the string representation of the identity type.
func (t IdentityType) String() string {
	return string(t)
}

// CanCreateChildren returns true if this identity type can create child identities.
func (t IdentityType) CanCreateChildren() bool {
	return t == IdentityTypeSystem || t == IdentityTypeManager
}

// DefaultSecurityLevel returns the default security level for this identity type.
func (t IdentityType) DefaultSecurityLevel() int {
	switch t {
	case IdentityTypeSystem:
		return 100
	case IdentityTypeManager:
		return 80
	case IdentityTypeWorker:
		return 50
	case IdentityTypeService:
		return 40
	case IdentityTypeEphemeral:
		return 30
	default:
		return 50
	}
}

// ScopeLevel represents the granularity of access scope.
type ScopeLevel string

const (
	// ScopeGlobal allows access to all resources
	ScopeGlobal ScopeLevel = "GLOBAL"

	// ScopeTenant limits access to a specific tenant
	ScopeTenant ScopeLevel = "TENANT"

	// ScopeNamespace limits access to a specific namespace within a tenant
	ScopeNamespace ScopeLevel = "NAMESPACE"

	// ScopeResource limits access to a specific resource
	ScopeResource ScopeLevel = "RESOURCE"
)

// IdentityScope defines the scope of access for an identity.
type IdentityScope struct {
	Level      ScopeLevel `json:"level"`
	TenantID   *string    `json:"tenant_id,omitempty"`
	Namespace  *string    `json:"namespace,omitempty"`
	ResourceID *string    `json:"resource_id,omitempty"`
}

// NewGlobalScope creates a global scope.
func NewGlobalScope() IdentityScope {
	return IdentityScope{
		Level: ScopeGlobal,
	}
}

// NewTenantScope creates a tenant-level scope.
func NewTenantScope(tenantID string) IdentityScope {
	return IdentityScope{
		Level:    ScopeTenant,
		TenantID: &tenantID,
	}
}

// NewNamespaceScope creates a namespace-level scope.
func NewNamespaceScope(tenantID, namespace string) IdentityScope {
	return IdentityScope{
		Level:     ScopeNamespace,
		TenantID:  &tenantID,
		Namespace: &namespace,
	}
}

// NewResourceScope creates a resource-level scope.
func NewResourceScope(tenantID, namespace, resourceID string) IdentityScope {
	return IdentityScope{
		Level:      ScopeResource,
		TenantID:   &tenantID,
		Namespace:  &namespace,
		ResourceID: &resourceID,
	}
}

// CanAccessTenant checks if this scope can access the given tenant.
func (s IdentityScope) CanAccessTenant(tenantID string) bool {
	if s.Level == ScopeGlobal {
		return true
	}
	return s.TenantID != nil && *s.TenantID == tenantID
}

// CanAccessNamespace checks if this scope can access the given namespace.
func (s IdentityScope) CanAccessNamespace(tenantID, namespace string) bool {
	if s.Level == ScopeGlobal {
		return true
	}
	if s.Level == ScopeTenant {
		return s.TenantID != nil && *s.TenantID == tenantID
	}
	return s.TenantID != nil && *s.TenantID == tenantID &&
		s.Namespace != nil && *s.Namespace == namespace
}

// CanAccessResource checks if this scope can access the given resource.
func (s IdentityScope) CanAccessResource(tenantID, namespace, resourceID string) bool {
	switch s.Level {
	case ScopeGlobal:
		return true
	case ScopeTenant:
		return s.TenantID != nil && *s.TenantID == tenantID
	case ScopeNamespace:
		return s.TenantID != nil && *s.TenantID == tenantID &&
			s.Namespace != nil && *s.Namespace == namespace
	case ScopeResource:
		return s.TenantID != nil && *s.TenantID == tenantID &&
			s.Namespace != nil && *s.Namespace == namespace &&
			s.ResourceID != nil && *s.ResourceID == resourceID
	}
	return false
}

// Capability represents a permission that an identity can possess.
type Capability struct {
	Name            string   `json:"name"`
	Category        string   `json:"category"`
	Actions         []string `json:"actions"`
	ResourcePattern *string  `json:"resource_pattern,omitempty"`
}

// NewCapability creates a new capability.
func NewCapability(name, category string, actions []string) Capability {
	return Capability{
		Name:     name,
		Category: category,
		Actions:  actions,
	}
}

// WithResourcePattern sets the resource pattern.
func (c Capability) WithResourcePattern(pattern string) Capability {
	c.ResourcePattern = &pattern
	return c
}

// AllowsAction checks if this capability allows the given action.
func (c Capability) AllowsAction(action string) bool {
	for _, a := range c.Actions {
		if a == action {
			return true
		}
	}
	return false
}

// IdentityStatus represents the current status of an identity.
type IdentityStatus string

const (
	// IdentityStatusActive means the identity is active and can be used
	IdentityStatusActive IdentityStatus = "ACTIVE"

	// IdentityStatusSuspended means the identity is temporarily suspended
	IdentityStatusSuspended IdentityStatus = "SUSPENDED"

	// IdentityStatusRevoked means the identity has been permanently revoked
	IdentityStatusRevoked IdentityStatus = "REVOKED"

	// IdentityStatusExpired means the identity has expired
	IdentityStatusExpired IdentityStatus = "EXPIRED"
)

// String returns the string representation of the status.
func (s IdentityStatus) String() string {
	return string(s)
}

// IdentityContext is the core identity structure.
type IdentityContext struct {
	ID            string            `json:"id"`
	Type          IdentityType      `json:"type"`
	Name          string            `json:"name"`
	Scope         IdentityScope     `json:"scope"`
	Capabilities  []Capability      `json:"capabilities"`
	Claims        map[string]string `json:"claims"`
	SecurityLevel int               `json:"security_level"`
	ParentID      *string           `json:"parent_id,omitempty"`
	Generation    int               `json:"generation"`
	CreatedAt     time.Time         `json:"created_at"`
	ExpiresAt     *time.Time        `json:"expires_at,omitempty"`
	Status        IdentityStatus    `json:"status"`
}

// NewIdentityContext creates a new identity context.
func NewIdentityContext(
	id string,
	identityType IdentityType,
	name string,
	scope IdentityScope,
) *IdentityContext {
	now := time.Now()
	expires := now.Add(24 * time.Hour)

	return &IdentityContext{
		ID:            id,
		Type:          identityType,
		Name:          name,
		Scope:         scope,
		Capabilities:  []Capability{},
		Claims:        make(map[string]string),
		SecurityLevel: identityType.DefaultSecurityLevel(),
		ParentID:      nil,
		Generation:    0,
		CreatedAt:     now,
		ExpiresAt:     &expires,
		Status:        IdentityStatusActive,
	}
}

// WithParent sets the parent identity.
func (i *IdentityContext) WithParent(parentID string, generation int) *IdentityContext {
	i.ParentID = &parentID
	i.Generation = generation
	return i
}

// WithSecurityLevel sets the security level.
func (i *IdentityContext) WithSecurityLevel(level int) *IdentityContext {
	i.SecurityLevel = level
	return i
}

// WithCapability adds a capability.
func (i *IdentityContext) WithCapability(cap Capability) *IdentityContext {
	i.Capabilities = append(i.Capabilities, cap)
	return i
}

// WithClaim adds a claim.
func (i *IdentityContext) WithClaim(key, value string) *IdentityContext {
	i.Claims[key] = value
	return i
}

// WithExpiration sets the expiration time.
func (i *IdentityContext) WithExpiration(expiresAt time.Time) *IdentityContext {
	i.ExpiresAt = &expiresAt
	return i
}

// HasCapability checks if the identity has the given capability.
func (i *IdentityContext) HasCapability(capabilityName string) bool {
	for _, c := range i.Capabilities {
		if c.Name == capabilityName {
			return true
		}
	}
	return false
}

// Can checks if the identity can perform the given action.
func (i *IdentityContext) Can(capability, action string) bool {
	for _, c := range i.Capabilities {
		if c.Name == capability && c.AllowsAction(action) {
			return true
		}
	}
	return false
}

// CanAccessTenant checks if the identity can access the given tenant.
func (i *IdentityContext) CanAccessTenant(tenantID string) bool {
	return i.Scope.CanAccessTenant(tenantID)
}

// CanAccessNamespace checks if the identity can access the given namespace.
func (i *IdentityContext) CanAccessNamespace(tenantID, namespace string) bool {
	return i.Scope.CanAccessNamespace(tenantID, namespace)
}

// IsActive checks if the identity is active.
func (i *IdentityContext) IsActive() bool {
	if i.Status != IdentityStatusActive {
		return false
	}

	if i.ExpiresAt != nil && i.ExpiresAt.Before(time.Now()) {
		return false
	}

	return true
}

// CanCreateChildren returns true if this identity can create children.
func (i *IdentityContext) CanCreateChildren() bool {
	return i.Type.CanCreateChildren()
}

// IdentityCredential represents an authentication credential.
type IdentityCredential struct {
	IdentityID string    `json:"identity_id"`
	Token      string    `json:"token"`
	IssuedAt   time.Time `json:"issued_at"`
	ExpiresAt  time.Time `json:"expires_at"`
	Scopes     []string  `json:"scopes"`
}

// NewIdentityCredential creates a new credential.
func NewIdentityCredential(identityID, token string, validity time.Duration) *IdentityCredential {
	now := time.Now()
	return &IdentityCredential{
		IdentityID: identityID,
		Token:      token,
		IssuedAt:   now,
		ExpiresAt:  now.Add(validity),
		Scopes:     []string{},
	}
}

// WithScope adds a scope.
func (c *IdentityCredential) WithScope(scope string) *IdentityCredential {
	c.Scopes = append(c.Scopes, scope)
	return c
}

// IsValid checks if the credential is valid.
func (c *IdentityCredential) IsValid() bool {
	return c.ExpiresAt.After(time.Now())
}

// HasScope checks if the credential has the given scope.
func (c *IdentityCredential) HasScope(scope string) bool {
	for _, s := range c.Scopes {
		if s == scope {
			return true
		}
	}
	return false
}

// EscalationToken represents a temporary privilege escalation.
type EscalationToken struct {
	TokenID    string    `json:"token_id"`
	IdentityID string    `json:"identity_id"`
	Capability string    `json:"capability"`
	GrantedAt  time.Time `json:"granted_at"`
	ExpiresAt  time.Time `json:"expires_at"`
	Reason     string    `json:"reason"`
}

// NewEscalationToken creates a new escalation token.
func NewEscalationToken(identityID, capability, reason string, duration time.Duration) *EscalationToken {
	now := time.Now()
	return &EscalationToken{
		TokenID:    uuid.New().String(),
		IdentityID: identityID,
		Capability: capability,
		GrantedAt:  now,
		ExpiresAt:  now.Add(duration),
		Reason:     reason,
	}
}

// IsValid checks if the token is valid.
func (e *EscalationToken) IsValid() bool {
	return e.ExpiresAt.After(time.Now())
}

// DelegationToken represents a delegation from one identity to another.
type DelegationToken struct {
	TokenID     string    `json:"token_id"`
	DelegatorID string    `json:"delegator_id"`
	DelegateeID string    `json:"delegatee_id"`
	Scopes      []string  `json:"scopes"`
	GrantedAt   time.Time `json:"granted_at"`
	ExpiresAt   time.Time `json:"expires_at"`
}

// NewDelegationToken creates a new delegation token.
func NewDelegationToken(delegatorID, delegateeID string, scopes []string, duration time.Duration) *DelegationToken {
	now := time.Now()
	return &DelegationToken{
		TokenID:     uuid.New().String(),
		DelegatorID: delegatorID,
		DelegateeID: delegateeID,
		Scopes:      scopes,
		GrantedAt:   now,
		ExpiresAt:   now.Add(duration),
	}
}

// IsValid checks if the token is valid.
func (d *DelegationToken) IsValid() bool {
	return d.ExpiresAt.After(time.Now())
}

// HasScope checks if the token has the given scope.
func (d *DelegationToken) HasScope(scope string) bool {
	for _, s := range d.Scopes {
		if s == scope {
			return true
		}
	}
	return false
}

// IdentityFilter is used for querying identities.
type IdentityFilter struct {
	IdentityType     *IdentityType   `json:"identity_type,omitempty"`
	TenantID         *string         `json:"tenant_id,omitempty"`
	Namespace        *string         `json:"namespace,omitempty"`
	Status           *IdentityStatus `json:"status,omitempty"`
	ParentID         *string         `json:"parent_id,omitempty"`
	MinSecurityLevel *int            `json:"min_security_level,omitempty"`
	MaxSecurityLevel *int            `json:"max_security_level,omitempty"`
}

// NewIdentityFilter creates a new filter.
func NewIdentityFilter() *IdentityFilter {
	return &IdentityFilter{}
}

// WithType sets the identity type filter.
func (f *IdentityFilter) WithType(identityType IdentityType) *IdentityFilter {
	f.IdentityType = &identityType
	return f
}

// WithTenant sets the tenant ID filter.
func (f *IdentityFilter) WithTenant(tenantID string) *IdentityFilter {
	f.TenantID = &tenantID
	return f
}

// WithNamespace sets the namespace filter.
func (f *IdentityFilter) WithNamespace(namespace string) *IdentityFilter {
	f.Namespace = &namespace
	return f
}

// WithStatus sets the status filter.
func (f *IdentityFilter) WithStatus(status IdentityStatus) *IdentityFilter {
	f.Status = &status
	return f
}

// WithParent sets the parent ID filter.
func (f *IdentityFilter) WithParent(parentID string) *IdentityFilter {
	f.ParentID = &parentID
	return f
}

// WithSecurityRange sets the security level range filter.
func (f *IdentityFilter) WithSecurityRange(min, max *int) *IdentityFilter {
	f.MinSecurityLevel = min
	f.MaxSecurityLevel = max
	return f
}

// IdentityManager provides identity management operations.
type IdentityManager struct {
	identities       map[string]*IdentityContext
	credentials      map[string]*IdentityCredential
	escalationTokens map[string]*EscalationToken
	delegationTokens map[string]*DelegationToken
	hierarchy        map[string][]string // parent -> children
}

// NewIdentityManager creates a new identity manager.
func NewIdentityManager() *IdentityManager {
	return &IdentityManager{
		identities:       make(map[string]*IdentityContext),
		credentials:      make(map[string]*IdentityCredential),
		escalationTokens: make(map[string]*EscalationToken),
		delegationTokens: make(map[string]*DelegationToken),
		hierarchy:        make(map[string][]string),
	}
}

// CreateIdentity creates a new identity.
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

	identity := NewIdentityContext(id, identityType, name, scope)
	identity.Generation = generation

	if parentID != nil {
		identity.ParentID = parentID
	}

	m.identities[id] = identity

	if parentID != nil {
		m.hierarchy[*parentID] = append(m.hierarchy[*parentID], id)
	}

	return identity, nil
}

// GetIdentity retrieves an identity by ID.
func (m *IdentityManager) GetIdentity(id string) (*IdentityContext, error) {
	if identity, ok := m.identities[id]; ok {
		return identity, nil
	}
	return nil, fmt.Errorf("identity not found: %s", id)
}

// IssueCredential issues a new credential.
func (m *IdentityManager) IssueCredential(
	identityID string,
	scopes []string,
	validity time.Duration,
) (*IdentityCredential, error) {
	identity, err := m.GetIdentity(identityID)
	if err != nil {
		return nil, err
	}

	if !identity.IsActive() {
		return nil, fmt.Errorf("identity not active: %s", identityID)
	}

	token := uuid.New().String()
	credential := NewIdentityCredential(identityID, token, validity)

	for _, scope := range scopes {
		credential.WithScope(scope)
	}

	m.credentials[token] = credential

	return credential, nil
}

// ValidateCredential validates a credential token.
func (m *IdentityManager) ValidateCredential(token string) (*IdentityCredential, error) {
	credential, ok := m.credentials[token]
	if !ok {
		return nil, fmt.Errorf("invalid credential")
	}

	if !credential.IsValid() {
		return nil, fmt.Errorf("credential expired")
	}

	return credential, nil
}

// Escalate grants a temporary privilege escalation.
func (m *IdentityManager) Escalate(
	identityID string,
	capability string,
	reason string,
	duration time.Duration,
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
		return nil, fmt.Errorf("escalation denied: identity does not have capability %s", capability)
	}

	token := NewEscalationToken(identityID, capability, reason, duration)
	m.escalationTokens[token.TokenID] = token

	return token, nil
}

// DeEscalate revokes an escalation token.
func (m *IdentityManager) DeEscalate(tokenID string) error {
	delete(m.escalationTokens, tokenID)
	return nil
}

// Delegate creates a delegation from one identity to another.
func (m *IdentityManager) Delegate(
	delegatorID string,
	delegateeID string,
	scopes []string,
	duration time.Duration,
) (*DelegationToken, error) {
	// Check hierarchy relationship
	if !m.isAncestor(delegatorID, delegateeID) && !m.isDescendant(delegatorID, delegateeID) {
		return nil, fmt.Errorf("not in hierarchy: %s and %s", delegatorID, delegateeID)
	}

	token := NewDelegationToken(delegatorID, delegateeID, scopes, duration)
	m.delegationTokens[token.TokenID] = token

	return token, nil
}

// RevokeDelegation revokes a delegation token.
func (m *IdentityManager) RevokeDelegation(tokenID string) error {
	delete(m.delegationTokens, tokenID)
	return nil
}

// RevokeIdentity revokes an identity and all its children.
func (m *IdentityManager) RevokeIdentity(identityID string, reason string) error {
	identity, err := m.GetIdentity(identityID)
	if err != nil {
		return err
	}

	identity.Status = IdentityStatusRevoked

	// Recursively revoke children
	children := m.GetChildren(identityID)
	for _, childID := range children {
		m.RevokeIdentity(childID, fmt.Sprintf("parent revoked: %s", identityID))
	}

	// Revoke all credentials
	m.revokeAllCredentials(identityID)

	return nil
}

// GetChildren returns the child identities of the given identity.
func (m *IdentityManager) GetChildren(parentID string) []string {
	return m.hierarchy[parentID]
}

// ListIdentities returns all identities matching the filter.
func (m *IdentityManager) ListIdentities(filter *IdentityFilter) []*IdentityContext {
	var result []*IdentityContext

	for _, identity := range m.identities {
		if m.matchesFilter(identity, filter) {
			result = append(result, identity)
		}
	}

	return result
}

// matchesFilter checks if an identity matches the filter.
func (m *IdentityManager) matchesFilter(identity *IdentityContext, filter *IdentityFilter) bool {
	if filter == nil {
		return true
	}

	if filter.IdentityType != nil && identity.Type != *filter.IdentityType {
		return false
	}

	if filter.TenantID != nil {
		if !identity.CanAccessTenant(*filter.TenantID) {
			return false
		}
	}

	if filter.Namespace != nil {
		if identity.Scope.Namespace == nil || *identity.Scope.Namespace != *filter.Namespace {
			return false
		}
	}

	if filter.Status != nil && identity.Status != *filter.Status {
		return false
	}

	if filter.ParentID != nil {
		if identity.ParentID == nil || *identity.ParentID != *filter.ParentID {
			return false
		}
	}

	if filter.MinSecurityLevel != nil && identity.SecurityLevel < *filter.MinSecurityLevel {
		return false
	}

	if filter.MaxSecurityLevel != nil && identity.SecurityLevel > *filter.MaxSecurityLevel {
		return false
	}

	return true
}

func (m *IdentityManager) isAncestor(ancestor, descendant string) bool {
	current := descendant
	for {
		identity, ok := m.identities[current]
		if !ok || identity.ParentID == nil {
			break
		}
		if *identity.ParentID == ancestor {
			return true
		}
		current = *identity.ParentID
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
