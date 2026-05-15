use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::auth::{
    AuthError, AuthResult, ExecutorAction, MultiTenancyAction, TenantId,
};

#[derive(Clone, Debug)]
pub struct VariantProcurementLink {
    pub id: String,
    pub tenant_id: TenantId,
    pub variant_id: String,
    pub supplier_id: String,
    pub procurement_code: String,
    pub metadata: Value,
}

impl VariantProcurementLink {
    pub fn new(
        id: impl Into<String>,
        tenant_id: TenantId,
        variant_id: impl Into<String>,
        supplier_id: impl Into<String>,
        procurement_code: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            id: id.into(),
            tenant_id,
            variant_id: variant_id.into(),
            supplier_id: supplier_id.into(),
            procurement_code: procurement_code.into(),
            metadata,
        }
    }
}

#[derive(Debug)]
pub struct UpsertVariantProcurementLinkInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub tenant_id: &'a TenantId,
    pub variant_id: &'a str,
    pub supplier_id: &'a str,
    pub procurement_code: &'a str,
    pub metadata: Option<&'a Value>,
}

#[derive(Debug)]
pub struct DeleteVariantProcurementLinksInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub tenant_id: &'a TenantId,
    pub variant_id: &'a str,
}

#[derive(Debug)]
pub struct DeleteVariantProcurementLinkInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub tenant_id: &'a TenantId,
    pub link_id: &'a str,
}

#[derive(Clone, Debug)]
pub struct DeliverySlipLine {
    pub sku: String,
    pub quantity: i64,
    pub sku_name: Option<String>,
    pub vendor_sku_text: Option<String>,
    pub unit_price: Option<String>,
    pub lot_no: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub confidence: Option<f64>,
    pub review_required: bool,
}

impl DeliverySlipLine {
    pub fn new(sku: impl Into<String>, quantity: i64) -> Self {
        Self {
            sku: sku.into(),
            quantity,
            sku_name: None,
            vendor_sku_text: None,
            unit_price: None,
            lot_no: None,
            received_at: None,
            expires_at: None,
            confidence: None,
            review_required: false,
        }
    }
}

#[derive(Debug)]
pub struct UploadDeliverySlipInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub file_name: &'a str,
    pub content_type: &'a str,
    pub file_bytes: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct UploadedDeliverySlip {
    pub delivery_slip_id: String,
    pub ocr_document_id: String,
    pub original_object_key: Option<String>,
    pub original_url: Option<String>,
    pub original_sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub confidence: Option<f64>,
    pub review_required: bool,
    pub review_reasons: Vec<String>,
    pub lines: Vec<DeliverySlipLine>,
}

#[derive(Clone, Debug)]
pub struct CommitReceivingLineInput {
    pub sku: String,
    pub quantity: i64,
    pub lot_no: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CommitReceivingLineInput {
    pub fn new(sku: impl Into<String>, quantity: i64) -> Self {
        Self {
            sku: sku.into(),
            quantity,
            lot_no: None,
            received_at: None,
            expires_at: None,
        }
    }
}

#[derive(Debug)]
pub struct CommitReceivingInput<'a> {
    pub executor: &'a dyn ExecutorAction,
    pub multi_tenancy: &'a dyn MultiTenancyAction,
    pub slip_id: &'a str,
    pub lines: &'a [CommitReceivingLineInput],
}

#[derive(Clone, Debug)]
pub struct CommitReceivingResult {
    pub receiving_record_id: String,
    pub discrepancy_count: usize,
    pub supplier_snapshot: Option<SupplierSnapshot>,
    pub evidence_snapshot: Option<EvidenceSnapshot>,
    pub lines: Vec<DeliverySlipLine>,
}

#[derive(Clone, Debug)]
pub struct SupplierSnapshot {
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EvidenceSnapshot {
    pub delivery_slip_id: String,
    pub ocr_document_id: String,
    pub file_name: String,
    pub content_type: String,
    pub uploaded_at: String,
}

#[derive(Clone, Debug)]
pub struct LlmModelPricing {
    pub model_name: String,
    pub input_token_cost: i64,
    pub output_token_cost: i64,
    pub cached_input_token_cost: Option<i64>,
    pub cache_creation_input_token_cost: Option<i64>,
}

#[async_trait::async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ProcurementApp: Debug + Send + Sync + 'static {
    async fn find_by_supplier_and_code(
        &self,
        tenant_id: &TenantId,
        supplier_id: &str,
        procurement_code: &str,
    ) -> AuthResult<Option<VariantProcurementLink>>;

    async fn upsert_variant_procurement_link(
        &self,
        input: &UpsertVariantProcurementLinkInput<'_>,
    ) -> AuthResult<VariantProcurementLink>;

    async fn list_variant_procurement_links(
        &self,
        tenant_id: &TenantId,
        variant_id: &str,
    ) -> AuthResult<Vec<VariantProcurementLink>>;

    async fn delete_variant_procurement_links(
        &self,
        input: &DeleteVariantProcurementLinksInput<'_>,
    ) -> AuthResult<()>;

    async fn delete_variant_procurement_link(
        &self,
        input: &DeleteVariantProcurementLinkInput<'_>,
    ) -> AuthResult<()>;

    async fn upload_delivery_slip(
        &self,
        input: &UploadDeliverySlipInput<'_>,
    ) -> AuthResult<UploadedDeliverySlip>;

    async fn commit_receiving(
        &self,
        input: &CommitReceivingInput<'_>,
    ) -> AuthResult<CommitReceivingResult>;

    async fn get_llm_cost(
        &self,
        tenant_id: &TenantId,
        model_name: &str,
    ) -> AuthResult<LlmModelPricing>;
}

#[derive(Debug, Default)]
pub struct NoOpProcurementApp;

#[async_trait::async_trait]
impl ProcurementApp for NoOpProcurementApp {
    async fn find_by_supplier_and_code(
        &self,
        _tenant_id: &TenantId,
        _supplier_id: &str,
        _procurement_code: &str,
    ) -> AuthResult<Option<VariantProcurementLink>> {
        Ok(None)
    }

    async fn upsert_variant_procurement_link(
        &self,
        input: &UpsertVariantProcurementLinkInput<'_>,
    ) -> AuthResult<VariantProcurementLink> {
        Ok(VariantProcurementLink::new(
            format!("noop-{}-{}", input.variant_id, input.supplier_id),
            input.tenant_id.clone(),
            input.variant_id,
            input.supplier_id,
            input.procurement_code,
            input
                .metadata
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        ))
    }

    async fn list_variant_procurement_links(
        &self,
        _tenant_id: &TenantId,
        _variant_id: &str,
    ) -> AuthResult<Vec<VariantProcurementLink>> {
        Ok(vec![])
    }

    async fn delete_variant_procurement_links(
        &self,
        _input: &DeleteVariantProcurementLinksInput<'_>,
    ) -> AuthResult<()> {
        Ok(())
    }

    async fn delete_variant_procurement_link(
        &self,
        _input: &DeleteVariantProcurementLinkInput<'_>,
    ) -> AuthResult<()> {
        Ok(())
    }

    async fn upload_delivery_slip(
        &self,
        input: &UploadDeliverySlipInput<'_>,
    ) -> AuthResult<UploadedDeliverySlip> {
        Ok(UploadedDeliverySlip {
            delivery_slip_id: format!("noop-slip-{}", input.file_name),
            ocr_document_id: format!("noop-ocr-{}", input.file_name),
            original_object_key: None,
            original_url: None,
            original_sha256: None,
            size_bytes: Some(input.file_bytes.len() as u64),
            confidence: Some(1.0),
            review_required: false,
            review_reasons: vec![],
            lines: vec![
                DeliverySlipLine::new("mock-sku-1", 3),
                DeliverySlipLine::new("mock-sku-2", 1),
            ],
        })
    }

    async fn commit_receiving(
        &self,
        input: &CommitReceivingInput<'_>,
    ) -> AuthResult<CommitReceivingResult> {
        Ok(CommitReceivingResult {
            receiving_record_id: format!(
                "noop-receiving-{}",
                input.slip_id
            ),
            discrepancy_count: 0,
            supplier_snapshot: None,
            evidence_snapshot: None,
            lines: input
                .lines
                .iter()
                .map(|line| DeliverySlipLine::new(&line.sku, line.quantity))
                .collect(),
        })
    }

    async fn get_llm_cost(
        &self,
        _tenant_id: &TenantId,
        model_name: &str,
    ) -> AuthResult<LlmModelPricing> {
        if model_name.is_empty() {
            return Err(AuthError::BadRequest(
                "model_name must not be empty".to_string(),
            ));
        }

        Ok(LlmModelPricing {
            model_name: model_name.to_string(),
            input_token_cost: 0,
            output_token_cost: 0,
            cached_input_token_cost: None,
            cache_creation_input_token_cost: None,
        })
    }
}
