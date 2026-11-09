use chrono::{DateTime, Utc};

use super::domain::{
    DefaultRole, EvaluatePoliciesBatchOutcome, NewOperatorOwnerMethod, OAuth2ClientCreated,
    OAuthToken, OAuthTokenDetail, Operator, Policy, PublicApiKey, ServiceAccount, TenantHierarchy,
    User,
};
use super::error::{AuthError, AuthResult};
use super::inputs::*;
use super::traits::AuthApp;
use super::types::{
    Identifier, PlatformId, PublicApiKeyId, PublicApiKeyValue, ServiceAccountId, TenantId, UserId,
};
use crate::apis::{
    self, auth_api_keys_api, auth_o_auth2_clients_api, auth_o_auth_tokens_api, auth_operators_api,
    auth_policies_api, auth_service_accounts_api, auth_user_policies_api, auth_users_api,
    configuration::Configuration,
};
use crate::models;

/// REST-backed [`AuthApp`] implementation using generated API bindings.
#[derive(Debug, Clone)]
pub struct SdkAuthApp {
    configuration: Configuration,
}

impl SdkAuthApp {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
}

fn not_supported(operation: &str, classification: &str) -> AuthError {
    AuthError::Internal(format!(
        "{operation} is not supported by SdkAuthApp yet ({classification})"
    ))
}

fn map_api_error<T>(operation: &str, error: apis::Error<T>) -> AuthError {
    match error {
        apis::Error::Reqwest(error) => AuthError::Http(error),
        apis::Error::Serde(error) => AuthError::Internal(format!("{operation}: {error}")),
        apis::Error::Io(error) => AuthError::Internal(format!("{operation}: {error}")),
        apis::Error::ResponseError(response) => {
            let message = if response.content.is_empty() {
                format!("{operation}: status {}", response.status)
            } else {
                format!(
                    "{operation}: status {}: {}",
                    response.status, response.content
                )
            };

            match response.status.as_u16() {
                400 => AuthError::BadRequest(message),
                401 => AuthError::Unauthorized(message),
                403 => AuthError::Forbidden(message),
                404 => AuthError::NotFound(message),
                _ => AuthError::Internal(message),
            }
        }
    }
}

fn parse_datetime(value: &str, field: &str) -> AuthResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|error| AuthError::Internal(format!("{field}: {error}")))
}

fn parse_default_role(value: &str) -> DefaultRole {
    match value.to_ascii_lowercase().as_str() {
        "owner" => DefaultRole::Owner,
        "manager" => DefaultRole::Manager,
        "store" => DefaultRole::Store,
        _ => DefaultRole::General,
    }
}

fn operator_from_response(response: models::OperatorResponse) -> Operator {
    let now = Utc::now();
    Operator {
        id: TenantId::new(response.id),
        name: response.name,
        operator_name: Identifier::new(response.operator_name),
        platform_id: PlatformId::new(response.platform_id),
        created_at: now,
        updated_at: now,
    }
}

fn service_account_from_response(
    response: models::ServiceAccountResponse,
) -> AuthResult<ServiceAccount> {
    Ok(ServiceAccount {
        id: ServiceAccountId::new(response.id),
        tenant_id: TenantId::new(response.tenant_id),
        name: response.name,
        created_at: parse_datetime(&response.created_at, "service_account.created_at")?,
    })
}

fn public_api_key_from_response(
    response: models::ApiKeyResponse,
    tenant_id: &TenantId,
) -> AuthResult<PublicApiKey> {
    Ok(PublicApiKey {
        id: PublicApiKeyId::new(response.id),
        tenant_id: tenant_id.clone(),
        service_account_id: ServiceAccountId::new(response.service_account_id),
        name: response.name,
        value: PublicApiKeyValue::new(response.value),
        created_at: parse_datetime(&response.created_at, "api_key.created_at")?,
    })
}

fn user_from_response(response: models::UserResponse) -> User {
    let now = Utc::now();
    User {
        id: UserId::new(response.id.clone()),
        username: response.id,
        tenants: response.tenants.into_iter().map(TenantId::new).collect(),
        email: response.email.flatten(),
        name: response.name.flatten(),
        email_verified: None,
        image: None,
        role: parse_default_role(&response.role),
        metadata: None,
        created_at: now,
        updated_at: now,
    }
}

fn policy_from_response(response: models::PolicyResponse) -> AuthResult<Policy> {
    Ok(Policy {
        id: response.id.into(),
        name: response.name,
        description: response.description.flatten(),
        is_system: response.is_system,
        tenant_id: response.tenant_id.flatten().map(TenantId::new),
        created_at: parse_datetime(&response.created_at, "policy.created_at")?,
        updated_at: parse_datetime(&response.updated_at, "policy.updated_at")?,
    })
}

fn oauth_token_detail_from_response(
    response: models::OAuthTokenDetailResponse,
) -> AuthResult<OAuthTokenDetail> {
    Ok(OAuthTokenDetail {
        provider: response.provider,
        provider_user_id: response.provider_user_id,
        access_token: response.access_token,
        refresh_token: response.refresh_token.flatten(),
        expires_at: parse_datetime(&response.expires_at, "oauth.expires_at")?,
    })
}

#[async_trait::async_trait]
impl AuthApp for SdkAuthApp {
    async fn check_policy<'a>(&self, input: &CheckPolicyInput<'a>) -> AuthResult<()> {
        let response = auth_policies_api::evaluate_policies_batch(
            &self.configuration,
            models::EvaluatePoliciesBatchRequest::new(vec![input.action.to_string()]),
        )
        .await
        .map_err(|error| map_api_error("evaluate_policies_batch", error))?;

        match response.results.first() {
            Some(outcome) if outcome.allowed => Ok(()),
            Some(outcome) => Err(AuthError::Forbidden(
                outcome
                    .error
                    .clone()
                    .flatten()
                    .unwrap_or_else(|| format!("Policy denied {}", input.action)),
            )),
            None => Err(AuthError::Internal(format!(
                "No policy evaluation result for {}",
                input.action
            ))),
        }
    }

    async fn evaluate_policies_batch<'a>(
        &self,
        input: &EvaluatePoliciesBatchInput<'a>,
    ) -> AuthResult<Vec<EvaluatePoliciesBatchOutcome>> {
        let response = auth_policies_api::evaluate_policies_batch(
            &self.configuration,
            models::EvaluatePoliciesBatchRequest::new(
                input
                    .actions
                    .iter()
                    .map(|action| action.to_string())
                    .collect(),
            ),
        )
        .await
        .map_err(|error| map_api_error("evaluate_policies_batch", error))?;

        Ok(response
            .results
            .into_iter()
            .map(|outcome| EvaluatePoliciesBatchOutcome {
                action: outcome.action,
                allowed: outcome.allowed,
                error: outcome.error.flatten(),
            })
            .collect())
    }

    async fn get_tenant_hierarchy<'a>(
        &self,
        _tenant_id: &'a TenantId,
    ) -> AuthResult<TenantHierarchy> {
        Err(not_supported(
            "get_tenant_hierarchy",
            "B server endpoint addition",
        ))
    }

    async fn get_user_id_by_user_provider_id<'a>(
        &self,
        _input: &GetUserIdByUserProviderIdInput<'a>,
    ) -> AuthResult<Option<String>> {
        Err(not_supported(
            "get_user_id_by_user_provider_id",
            "B server endpoint addition",
        ))
    }

    async fn delete_operator<'a>(&self, _input: &DeleteOperatorInput<'a>) -> AuthResult<()> {
        Err(not_supported(
            "delete_operator",
            "B server endpoint addition",
        ))
    }

    async fn get_operator_by_identifier<'a>(
        &self,
        input: &GetOperatorByIdentifierInput<'a>,
    ) -> AuthResult<Option<Operator>> {
        let response = auth_operators_api::get_operator_by_alias(
            &self.configuration,
            input.platform_id.as_str(),
            input.operator_identifier.value(),
        )
        .await
        .map_err(|error| map_api_error("get_operator_by_alias", error))?;

        Ok(Some(operator_from_response(response)))
    }

    async fn get_operator_by_id<'a>(
        &self,
        input: &GetOperatorByIdInput<'a>,
    ) -> AuthResult<Option<Operator>> {
        let response =
            auth_operators_api::get_operator_by_id(&self.configuration, input.operator_id.as_str())
                .await
                .map_err(|error| map_api_error("get_operator_by_id", error))?;

        Ok(Some(operator_from_response(response)))
    }

    async fn create_operator<'a>(&self, input: &CreateOperatorInput<'a>) -> AuthResult<Operator> {
        let owner_method = match input.new_operator_owner_method {
            NewOperatorOwnerMethod::Inherit => "inherit",
            NewOperatorOwnerMethod::Create => "create",
        };
        let mut request = models::CreateOperatorRequest::new(
            input.new_operator_owner_id.to_string(),
            owner_method.to_string(),
            input.operator_name.to_string(),
            input.platform_id.to_string(),
        );
        request.operator_alias = Some(Some(input.operator_alias.to_string()));
        request.new_operator_owner_password =
            Some(input.new_operator_owner_password.map(str::to_string));

        let response = auth_operators_api::create_operator(&self.configuration, request)
            .await
            .map_err(|error| map_api_error("create_operator", error))?;

        Ok(operator_from_response(*response.operator))
    }

    async fn oauth_tokens<'a>(&self, _input: &OAuthTokenInput<'a>) -> AuthResult<Vec<OAuthToken>> {
        let response = auth_o_auth_tokens_api::list_oauth_tokens(&self.configuration)
            .await
            .map_err(|error| map_api_error("list_oauth_tokens", error))?;

        Ok(response
            .tokens
            .into_iter()
            .map(|token| OAuthToken {
                provider: token.provider,
                access_token: token.access_token,
            })
            .collect())
    }

    async fn get_oauth_token_by_provider<'a>(
        &self,
        input: &GetOAuthTokenByProviderInput<'a>,
    ) -> AuthResult<Option<OAuthTokenDetail>> {
        let response = auth_o_auth_tokens_api::get_oauth_token_by_provider(
            &self.configuration,
            input.provider,
        )
        .await
        .map_err(|error| map_api_error("get_oauth_token_by_provider", error))?;

        oauth_token_detail_from_response(response).map(Some)
    }

    async fn save_oauth_token<'a>(&self, input: &SaveOAuthTokenInput<'a>) -> AuthResult<()> {
        let mut request = models::SaveOAuthTokenRequest::new(
            input.access_token.to_string(),
            input.expires_in,
            input.provider.to_string(),
            input.provider_user_id.to_string(),
        );
        request.refresh_token = Some(input.refresh_token.map(str::to_string));

        auth_o_auth_tokens_api::save_oauth_token(&self.configuration, request)
            .await
            .map_err(|error| map_api_error("save_oauth_token", error))?;

        Ok(())
    }

    async fn delete_oauth_token<'a>(&self, input: &DeleteOAuthTokenInput<'a>) -> AuthResult<()> {
        auth_o_auth_tokens_api::delete_oauth_token(&self.configuration, input.provider)
            .await
            .map_err(|error| map_api_error("delete_oauth_token", error))?;

        Ok(())
    }

    async fn create_service_account<'a>(
        &self,
        input: &CreateServiceAccountInput<'a>,
    ) -> AuthResult<ServiceAccount> {
        let response = auth_service_accounts_api::create_service_account(
            &self.configuration,
            models::CreateServiceAccountRequest::new(
                input.name.to_string(),
                input.tenant_id.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("create_service_account", error))?;

        service_account_from_response(response)
    }

    async fn update_service_account<'a>(
        &self,
        input: &UpdateServiceAccountInput<'a>,
    ) -> AuthResult<ServiceAccount> {
        let mut request = models::UpdateServiceAccountRequest::new();
        request.name = Some(Some(input.name.to_string()));

        let response = auth_service_accounts_api::update_service_account(
            &self.configuration,
            input.service_account_id.as_str(),
            request,
        )
        .await
        .map_err(|error| map_api_error("update_service_account", error))?;

        service_account_from_response(response)
    }

    async fn get_service_account_by_name<'a>(
        &self,
        input: &GetServiceAccountByNameInput<'a>,
    ) -> AuthResult<Option<ServiceAccount>> {
        let response = auth_service_accounts_api::list_service_accounts(
            &self.configuration,
            input.tenant_id.as_str(),
        )
        .await
        .map_err(|error| map_api_error("list_service_accounts", error))?;

        response
            .service_accounts
            .into_iter()
            .find(|service_account| service_account.name == input.name)
            .map(service_account_from_response)
            .transpose()
    }

    async fn delete_service_account<'a>(
        &self,
        input: &DeleteServiceAccountInput<'a>,
    ) -> AuthResult<()> {
        auth_service_accounts_api::delete_service_account(
            &self.configuration,
            input.service_account_id.as_str(),
        )
        .await
        .map_err(|error| map_api_error("delete_service_account", error))?;

        Ok(())
    }

    async fn create_public_api_key<'a>(
        &self,
        input: &CreatePublicApiKeyInput<'a>,
    ) -> AuthResult<PublicApiKey> {
        let response = auth_api_keys_api::create_api_key(
            &self.configuration,
            input.service_account_id.as_str(),
            models::CreateApiKeyRequest::new(input.name.to_string(), input.operator_id.to_string()),
        )
        .await
        .map_err(|error| map_api_error("create_api_key", error))?;

        public_api_key_from_response(response, input.operator_id)
    }

    async fn find_all_public_api_key<'a>(
        &self,
        input: &FindAllPublicApiKeyInput<'a>,
    ) -> AuthResult<Vec<PublicApiKey>> {
        let response = auth_api_keys_api::list_api_keys(
            &self.configuration,
            input.service_account_id.as_str(),
            input.operator_id.as_str(),
        )
        .await
        .map_err(|error| map_api_error("list_api_keys", error))?;

        response
            .api_keys
            .into_iter()
            .map(|api_key| public_api_key_from_response(api_key, input.operator_id))
            .collect()
    }

    async fn attach_user_policy<'a>(&self, input: &AttachUserPolicyInput<'a>) -> AuthResult<()> {
        auth_user_policies_api::attach_user_policy(
            &self.configuration,
            models::AttachUserPolicyRequest::new(
                input.policy_id.to_string(),
                input.tenant_id.to_string(),
                input.user_id.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("attach_user_policy", error))?;

        Ok(())
    }

    async fn detach_user_policy<'a>(&self, input: &DetachUserPolicyInput<'a>) -> AuthResult<()> {
        auth_user_policies_api::detach_user_policy(
            &self.configuration,
            models::DetachUserPolicyRequest::new(
                input.policy_id.to_string(),
                input.tenant_id.to_string(),
                input.user_id.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("detach_user_policy", error))?;

        Ok(())
    }

    async fn check_policy_for_resource<'a>(
        &self,
        input: &CheckPolicyForResourceInput<'a>,
    ) -> AuthResult<()> {
        let response = auth_policies_api::check_policy_for_resource(
            &self.configuration,
            models::CheckPolicyForResourceRequest::new(
                input.action.to_string(),
                input.resource_trn.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("check_policy_for_resource", error))?;

        if response.allowed {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "Policy denied {} on {}",
                input.action, input.resource_trn
            )))
        }
    }

    async fn attach_user_policy_with_scope<'a>(
        &self,
        input: &AttachUserPolicyWithScopeInput<'a>,
    ) -> AuthResult<()> {
        auth_user_policies_api::attach_user_policy_with_scope(
            &self.configuration,
            models::AttachUserPolicyWithScopeRequest::new(
                input.policy_id.to_string(),
                input.resource_scope.to_string(),
                input.tenant_id.to_string(),
                input.user_id.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("attach_user_policy_with_scope", error))?;

        Ok(())
    }

    async fn detach_user_policy_with_scope<'a>(
        &self,
        input: &DetachUserPolicyWithScopeInput<'a>,
    ) -> AuthResult<()> {
        auth_user_policies_api::detach_user_policy_with_scope(
            &self.configuration,
            models::DetachUserPolicyWithScopeRequest::new(
                input.policy_id.to_string(),
                input.resource_scope.to_string(),
                input.tenant_id.to_string(),
                input.user_id.to_string(),
            ),
        )
        .await
        .map_err(|error| map_api_error("detach_user_policy_with_scope", error))?;

        Ok(())
    }

    async fn add_user_to_tenant<'a>(&self, input: &AddUserToTenantInput<'a>) -> AuthResult<()> {
        auth_users_api::add_user_to_tenant(
            &self.configuration,
            input.user_id.as_str(),
            models::AddUserToTenantRequest::new(input.tenant_id.to_string()),
        )
        .await
        .map_err(|error| map_api_error("add_user_to_tenant", error))?;

        Ok(())
    }

    async fn get_user_by_id<'a>(&self, input: &GetUserByIdInput<'a>) -> AuthResult<Option<User>> {
        let response = auth_users_api::get_user(&self.configuration, input.user_id.as_str())
            .await
            .map_err(|error| map_api_error("get_user", error))?;

        Ok(Some(user_from_response(response)))
    }

    async fn find_users_by_tenant<'a>(
        &self,
        input: &FindUsersByTenantInput<'a>,
    ) -> AuthResult<Vec<User>> {
        let response = auth_users_api::list_users(&self.configuration, input.tenant_id.as_str())
            .await
            .map_err(|error| map_api_error("list_users", error))?;

        Ok(response.users.into_iter().map(user_from_response).collect())
    }

    async fn get_policy_by_id<'a>(
        &self,
        input: &GetPolicyByIdInput<'a>,
    ) -> AuthResult<Option<Policy>> {
        let response = auth_policies_api::get_policy(&self.configuration, input.policy_id.as_str())
            .await
            .map_err(|error| map_api_error("get_policy", error))?;

        policy_from_response(response).map(Some)
    }

    async fn register_policy<'a>(&self, input: &RegisterPolicyInput<'a>) -> AuthResult<Policy> {
        let mut request = models::RegisterPolicyRequest::new(input.name.to_string());
        request.description = Some(input.description.map(str::to_string));
        request.is_system = Some(false);
        request.global = Some(false);
        request.tenant_id = Some(Some(input.tenant_id.to_string()));
        request.actions = Some(
            input
                .actions
                .iter()
                .map(|action| {
                    models::PolicyActionRequest::new(
                        action.action_id.clone(),
                        action.effect.clone(),
                    )
                })
                .collect(),
        );

        let response = auth_policies_api::register_policy(&self.configuration, request)
            .await
            .map_err(|error| map_api_error("register_policy", error))?;

        policy_from_response(response)
    }

    async fn find_policy_by_name<'a>(
        &self,
        _input: &FindPolicyByNameInput<'a>,
    ) -> AuthResult<Option<Policy>> {
        Err(not_supported(
            "find_policy_by_name",
            "B server endpoint addition",
        ))
    }

    async fn attach_sa_policy<'a>(&self, _input: &AttachSaPolicyInput<'a>) -> AuthResult<()> {
        Err(not_supported(
            "attach_sa_policy",
            "B server endpoint addition; no REST route is exposed yet",
        ))
    }

    async fn create_oauth2_client<'a>(
        &self,
        input: &CreateOAuth2ClientInput<'a>,
    ) -> AuthResult<OAuth2ClientCreated> {
        let mut request = models::OAuth2CreateClientRequest::new(
            input.allowed_scopes.clone(),
            input.grant_types.clone(),
            input.name.to_string(),
            input.redirect_uris.clone(),
        );
        request.use_tachyon_user_pool = Some(input.use_tachyon_user_pool);

        let response = auth_o_auth2_clients_api::create_oauth2_client(&self.configuration, request)
            .await
            .map_err(|error| map_api_error("create_oauth2_client", error))?;

        Ok(OAuth2ClientCreated {
            client_id: response.client_id,
            client_secret: response.client_secret,
            provider_user_pool_id: response.provider_user_pool_id.flatten(),
        })
    }

    async fn find_oauth2_client_by_name<'a>(
        &self,
        input: &FindOAuth2ClientByNameInput<'a>,
    ) -> AuthResult<Option<String>> {
        let response = auth_o_auth2_clients_api::list_oauth2_clients(&self.configuration)
            .await
            .map_err(|error| map_api_error("list_oauth2_clients", error))?;

        Ok(response
            .clients
            .into_iter()
            .find(|client| client.name == input.name)
            .map(|client| client.client_id))
    }
}
