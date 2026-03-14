use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{
    Identifier, OperatorId, PlatformId, PolicyId,
    PublicApiKeyId, PublicApiKeyValue, ServiceAccountId,
    TenantId, UserId,
};

// ───────────────────── DefaultRole ─────────────────────

/// Default role assigned to users within an operator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum DefaultRole {
    Owner,
    Manager,
    General,
    Store,
}

// ───────────────────── User ────────────────────────────

/// Represents an authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub tenants: Vec<TenantId>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub email_verified: Option<DateTime<Utc>>,
    pub image: Option<String>,
    pub role: DefaultRole,
    pub metadata: Option<std::collections::HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ───────────────────── Operator ────────────────────────

/// Represents a tenant operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub id: TenantId,
    pub name: String,
    pub operator_name: Identifier,
    pub platform_id: TenantId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ───────────────────── TenantHierarchy ─────────────────

/// Resolved tenant hierarchy (Host → Platform → Operator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantHierarchy {
    pub host_id: TenantId,
    pub platform_id: Option<PlatformId>,
    pub operator_id: Option<OperatorId>,
}

// ───────────────────── ServiceAccount ──────────────────

/// Machine user for API access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub id: ServiceAccountId,
    pub tenant_id: TenantId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

// ───────────────────── PublicApiKey ─────────────────────

/// API key associated with a service account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicApiKey {
    pub id: PublicApiKeyId,
    pub tenant_id: TenantId,
    pub service_account_id: ServiceAccountId,
    pub name: String,
    pub value: PublicApiKeyValue,
    pub created_at: DateTime<Utc>,
}

// ───────────────────── Policy ──────────────────────────

/// Represents an authorization policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub tenant_id: Option<TenantId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ───────────────────── UserPolicy ──────────────────────

/// Mapping between a user, a policy, and a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPolicy {
    pub user_id: UserId,
    pub policy_id: PolicyId,
    pub tenant_id: TenantId,
    pub resource_scope: Option<String>,
}

// ───────────────────── OAuth ───────────────────────────

/// Simplified OAuth token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub provider: String,
    pub access_token: String,
}

/// OAuth token with detailed information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenDetail {
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl OAuthTokenDetail {
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }
}

// ───────────────────── OAuth2 Client ───────────────────

/// Result of creating an OAuth2 client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ClientCreated {
    pub client_id: String,
    pub client_secret: String,
}

// ─────────────────── NewOperatorOwnerMethod ────────────

/// How the owner of a new operator is determined.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum NewOperatorOwnerMethod {
    Inherit,
    Create,
}

// ─────────────── PolicyActionRequest ───────────────────

/// Action entry for policy registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActionRequest {
    pub action_id: String,
    pub effect: String,
}

// ───────────── EvaluatePoliciesBatchOutcome ─────────────

/// Result of evaluating a single action in a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePoliciesBatchOutcome {
    pub action: String,
    pub allowed: bool,
    pub error: Option<String>,
}

// ───────────────────── UserQuery ────────────────────────

/// Trait for querying user information.
#[async_trait::async_trait]
pub trait UserQuery: std::fmt::Debug + Send + Sync {
    async fn find_by_id(
        &self,
        id: &UserId,
    ) -> super::error::AuthResult<Option<User>>;

    async fn find_by_tenant(
        &self,
        tenant_id: &TenantId,
    ) -> super::error::AuthResult<Vec<User>>;
}
