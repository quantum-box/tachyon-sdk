use std::fmt::Debug;

use crate::auth::{AuthResult, ExecutorAction, MultiTenancyAction, TenantId};

#[async_trait::async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait PaymentApp: Debug + Send + Sync + 'static {
    async fn check_billing<'a>(&self, input: &CheckBillingInput<'a>) -> AuthResult<()>;

    async fn consume_credits<'a>(
        &self,
        input: &ConsumeCreditsInput<'a>,
    ) -> AuthResult<ConsumeCreditsOutput>;

    async fn get_credit_balance<'a>(
        &self,
        input: &GetCreditBalanceInput<'a>,
    ) -> AuthResult<CreditBalance>;

    async fn charge_credits<'a>(
        &self,
        input: &ChargeCreditsInput<'a>,
    ) -> AuthResult<ChargeCreditsOutput>;

    async fn create_checkout_session(
        &self,
        package_id: String,
        currency: String,
        success_url: String,
        cancel_url: String,
        tenant_id: TenantId,
    ) -> AuthResult<CheckoutSessionOutput>;
}

#[derive(Debug, Clone)]
pub struct CheckBillingInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub estimated_cost: i64,
    pub resource_type: &'static str,
}

#[derive(Debug, Clone)]
pub struct ConsumeCreditsInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub amount: i64,
    pub resource_type: &'static str,
    pub resource_id: String,
    pub description: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ConsumeCreditsOutput {
    pub transaction_id: Option<String>,
    pub amount_consumed: i64,
    pub balance_after: i64,
    pub was_billed: bool,
}

#[derive(Debug, Clone)]
pub struct GetCreditBalanceInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
}

#[derive(Debug, Clone)]
pub struct CreditBalance {
    pub balance: i64,
    pub reserved: i64,
    pub available: i64,
    pub currency: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ChargeCreditsInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub package_id: String,
    pub payment_method: String,
}

#[derive(Debug, Clone)]
pub struct ChargeCreditsOutput {
    pub checkout_url: String,
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct CheckoutSessionOutput {
    pub checkout_url: String,
    pub session_id: String,
}
