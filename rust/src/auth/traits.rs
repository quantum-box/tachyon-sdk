use std::fmt::Debug;

use super::domain::*;
use super::error::AuthResult;
use super::inputs::*;
use super::types::TenantId;

/// Core authentication and authorization interface.
///
/// Implementations of this trait provide policy checks,
/// user management, OAuth token handling, and other auth
/// operations. The SDK ships a REST-based implementation
/// (`SdkAuthApp`) that delegates to a running Tachyon API
/// server.
#[async_trait::async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait AuthApp: Debug + Send + Sync + 'static {
    /// Check whether the executor is allowed to perform
    /// the given action.
    async fn check_policy<'a>(
        &self,
        input: &CheckPolicyInput<'a>,
    ) -> AuthResult<()>;

    /// Evaluate multiple actions in a single batch.
    async fn evaluate_policies_batch<'a>(
        &self,
        input: &EvaluatePoliciesBatchInput<'a>,
    ) -> AuthResult<Vec<EvaluatePoliciesBatchOutcome>>;

    /// Resolve the tenant hierarchy for a given tenant.
    async fn get_tenant_hierarchy<'a>(
        &self,
        tenant_id: &'a TenantId,
    ) -> AuthResult<TenantHierarchy>;

    /// Look up a user ID by external provider user ID.
    async fn get_user_id_by_user_provider_id<'a>(
        &self,
        input: &GetUserIdByUserProviderIdInput<'a>,
    ) -> AuthResult<Option<String>>;

    /// Delete an operator.
    async fn delete_operator<'a>(
        &self,
        input: &DeleteOperatorInput<'a>,
    ) -> AuthResult<()>;

    /// Find an operator by its identifier (alias).
    async fn get_operator_by_identifier<'a>(
        &self,
        input: &GetOperatorByIdentifierInput<'a>,
    ) -> AuthResult<Option<Operator>>;

    /// Find an operator by its ID.
    async fn get_operator_by_id<'a>(
        &self,
        input: &GetOperatorByIdInput<'a>,
    ) -> AuthResult<Option<Operator>>;

    /// Create a new operator under a platform.
    async fn create_operator<'a>(
        &self,
        input: &CreateOperatorInput<'a>,
    ) -> AuthResult<Operator>;

    /// List authorized OAuth tokens for the executor.
    async fn oauth_tokens<'a>(
        &self,
        input: &OAuthTokenInput<'a>,
    ) -> AuthResult<Vec<OAuthToken>>;

    /// Get a specific OAuth token by provider name.
    async fn get_oauth_token_by_provider<'a>(
        &self,
        input: &GetOAuthTokenByProviderInput<'a>,
    ) -> AuthResult<Option<OAuthTokenDetail>>;

    /// Save (create or update) an OAuth token.
    async fn save_oauth_token<'a>(
        &self,
        input: &SaveOAuthTokenInput<'a>,
    ) -> AuthResult<()>;

    /// Delete an OAuth token for a given provider.
    async fn delete_oauth_token<'a>(
        &self,
        input: &DeleteOAuthTokenInput<'a>,
    ) -> AuthResult<()>;

    /// Create a service account in a tenant.
    async fn create_service_account<'a>(
        &self,
        input: &CreateServiceAccountInput<'a>,
    ) -> AuthResult<ServiceAccount>;

    /// Update a service account's name.
    async fn update_service_account<'a>(
        &self,
        input: &UpdateServiceAccountInput<'a>,
    ) -> AuthResult<ServiceAccount>;

    /// Look up a service account by name.
    async fn get_service_account_by_name<'a>(
        &self,
        input: &GetServiceAccountByNameInput<'a>,
    ) -> AuthResult<Option<ServiceAccount>>;

    /// Delete a service account.
    async fn delete_service_account<'a>(
        &self,
        input: &DeleteServiceAccountInput<'a>,
    ) -> AuthResult<()>;

    /// Create a public API key.
    async fn create_public_api_key<'a>(
        &self,
        input: &CreatePublicApiKeyInput<'a>,
    ) -> AuthResult<PublicApiKey>;

    /// List all API keys for a service account.
    async fn find_all_public_api_key<'a>(
        &self,
        input: &FindAllPublicApiKeyInput<'a>,
    ) -> AuthResult<Vec<PublicApiKey>>;

    /// Attach a policy to a user in a tenant.
    async fn attach_user_policy<'a>(
        &self,
        input: &AttachUserPolicyInput<'a>,
    ) -> AuthResult<()>;

    /// Detach a policy from a user in a tenant.
    async fn detach_user_policy<'a>(
        &self,
        input: &DetachUserPolicyInput<'a>,
    ) -> AuthResult<()>;

    /// Check resource-level permissions.
    async fn check_policy_for_resource<'a>(
        &self,
        input: &CheckPolicyForResourceInput<'a>,
    ) -> AuthResult<()>;

    /// Attach a policy with resource scope.
    async fn attach_user_policy_with_scope<'a>(
        &self,
        input: &AttachUserPolicyWithScopeInput<'a>,
    ) -> AuthResult<()>;

    /// Detach a policy with resource scope.
    async fn detach_user_policy_with_scope<'a>(
        &self,
        input: &DetachUserPolicyWithScopeInput<'a>,
    ) -> AuthResult<()>;

    /// Add a user to a tenant.
    async fn add_user_to_tenant<'a>(
        &self,
        input: &AddUserToTenantInput<'a>,
    ) -> AuthResult<()>;

    /// Get a user by ID.
    async fn get_user_by_id<'a>(
        &self,
        input: &GetUserByIdInput<'a>,
    ) -> AuthResult<Option<User>>;

    /// Find all users in a tenant.
    async fn find_users_by_tenant<'a>(
        &self,
        input: &FindUsersByTenantInput<'a>,
    ) -> AuthResult<Vec<User>>;

    /// Get a policy by ID.
    async fn get_policy_by_id<'a>(
        &self,
        input: &GetPolicyByIdInput<'a>,
    ) -> AuthResult<Option<Policy>>;

    /// Register (create) a custom policy with actions.
    async fn register_policy<'a>(
        &self,
        input: &RegisterPolicyInput<'a>,
    ) -> AuthResult<Policy>;

    /// Find a custom policy by name within a tenant.
    async fn find_policy_by_name<'a>(
        &self,
        input: &FindPolicyByNameInput<'a>,
    ) -> AuthResult<Option<Policy>>;

    /// Attach a policy to a service account.
    async fn attach_sa_policy<'a>(
        &self,
        input: &AttachSaPolicyInput<'a>,
    ) -> AuthResult<()>;

    /// Create an OAuth2 client.
    async fn create_oauth2_client<'a>(
        &self,
        input: &CreateOAuth2ClientInput<'a>,
    ) -> AuthResult<OAuth2ClientCreated>;

    /// Find an OAuth2 client by display name.
    async fn find_oauth2_client_by_name<'a>(
        &self,
        input: &FindOAuth2ClientByNameInput<'a>,
    ) -> AuthResult<Option<String>>;
}
