use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ContractSource {
    OpenApiOperation,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ContractRegistration {
    pub id: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub success_status: u16,
    pub source: ContractSource,
    pub api_owner: &'static str,
    pub cli_owner: &'static str,
    pub consumer_schema: fn() -> Value,
    pub round_trip: fn(&Value) -> Result<Value>,
}

#[allow(dead_code)]
pub trait TachyonResponseContract:
    DeserializeOwned + Serialize + JsonSchema + Send + Sync + 'static
{
    const ID: &'static str;
    const METHOD: &'static str;
    const PATH: &'static str;
    const SUCCESS_STATUS: u16;
    const SOURCE: ContractSource;
    const API_OWNER: &'static str;
    const CLI_OWNER: &'static str;
}

#[allow(dead_code)]
fn consumer_schema<T: TachyonResponseContract>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("schemars contract schema must serialize")
}

#[allow(dead_code)]
fn round_trip<T: TachyonResponseContract>(input: &Value) -> Result<Value> {
    let decoded: T = serde_json::from_value(input.clone())
        .with_context(|| format!("decode {} maximal/minimal fixture", T::ID))?;
    serde_json::to_value(decoded).with_context(|| format!("serialize {} fixture", T::ID))
}

#[allow(dead_code)]
pub fn registered_contract<T: TachyonResponseContract>() -> ContractRegistration {
    ContractRegistration {
        id: T::ID,
        method: T::METHOD,
        path: T::PATH,
        success_status: T::SUCCESS_STATUS,
        source: T::SOURCE,
        api_owner: T::API_OWNER,
        cli_owner: T::CLI_OWNER,
        consumer_schema: consumer_schema::<T>,
        round_trip: round_trip::<T>,
    }
}

macro_rules! tachyon_response_contract {
    (
        root: $root:ty,
        id: $id:literal,
        operation: ($method:literal, $path:literal, $status:literal),
        api_owner: $api_owner:literal,
        cli_owner: $cli_owner:literal $(,)?
    ) => {
        impl crate::response_contract::TachyonResponseContract for $root {
            const ID: &'static str = $id;
            const METHOD: &'static str = $method;
            const PATH: &'static str = $path;
            const SUCCESS_STATUS: u16 = $status;
            const SOURCE: crate::response_contract::ContractSource =
                crate::response_contract::ContractSource::OpenApiOperation;
            const API_OWNER: &'static str = $api_owner;
            const CLI_OWNER: &'static str = $cli_owner;
        }
    };
}

pub(crate) use tachyon_response_contract;

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use anyhow::{anyhow, bail};
    use serde::Deserialize;
    use sha2::{Digest, Sha256};
    use syn::visit::Visit;

    use super::*;

    const ORG_SOURCE: &str = include_str!("org_cli.rs");

    fn contract_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contracts/tachyon-api")
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Provenance {
        schema_version: u32,
        repository: String,
        commit: String,
        openapi_path: String,
        openapi_sha256: String,
        bundle_sha256: String,
        files: Vec<ProvenanceFile>,
        external_contracts: Vec<ExternalContractProvenance>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ProvenanceFile {
        path: String,
        sha256: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    #[allow(dead_code)]
    struct ExternalContractProvenance {
        id: String,
        schema_path: String,
        fixture_path: String,
        source_path: String,
        schema_sha256: String,
        fixture_sha256: String,
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn bundle_sha256(files: &BTreeMap<&str, &[u8]>) -> String {
        let mut digest = Sha256::new();
        for (path, bytes) in files {
            digest.update(path.as_bytes());
            digest.update([0]);
            digest.update(bytes);
        }
        format!("{:x}", digest.finalize())
    }

    fn verify_vendored_bundle() -> Result<Value> {
        let directory = contract_directory();
        let openapi_bytes =
            std::fs::read(directory.join("openapi.json")).context("read vendored OpenAPI")?;
        let provenance_bytes = std::fs::read(directory.join("provenance.json"))
            .context("read vendored provenance.json")?;
        let fixture_overrides = std::fs::read_to_string(directory.join("fixture-overrides.toml"))
            .context("read fixture-overrides.toml")?;
        let provenance: Provenance =
            serde_json::from_slice(&provenance_bytes).context("parse vendored provenance.json")?;
        if provenance.schema_version != 1 {
            bail!(
                "unsupported provenance schema {}",
                provenance.schema_version
            );
        }
        if provenance.repository != "quantum-box/tachyon-apps" {
            bail!("unexpected producer repository {}", provenance.repository);
        }
        if provenance.commit.len() != 40
            || !provenance
                .commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            bail!("producer commit is not a lowercase 40-character SHA");
        }
        if provenance.openapi_path != "apps/tachyon/src/gen/openapi/tachyon-api.openapi.json" {
            bail!(
                "vendored source is not the authoritative OpenAPI: {}",
                provenance.openapi_path
            );
        }

        let openapi_digest = sha256(&openapi_bytes);
        if provenance.openapi_sha256 != openapi_digest {
            bail!("OpenAPI SHA-256 does not match provenance");
        }
        if !provenance.external_contracts.is_empty() {
            bail!("Phase 1 auth/list bundle must not enroll external families");
        }
        if provenance.files.len() != 1
            || provenance.files[0].path != "openapi.json"
            || provenance.files[0].sha256 != openapi_digest
        {
            bail!("provenance file inventory does not match the auth/list bundle");
        }

        let files = BTreeMap::from([("openapi.json", openapi_bytes.as_slice())]);
        if provenance.bundle_sha256 != bundle_sha256(&files) {
            bail!("bundle SHA-256 does not match vendored files");
        }
        if fixture_overrides.trim()
            != "schema_version = 1\n\n# Phase 1 auth/list contracts require no generator overrides."
        {
            bail!("fixture overrides changed without runner support");
        }

        let openapi: Value =
            serde_json::from_slice(&openapi_bytes).context("parse vendored OpenAPI")?;
        if openapi.get("openapi").and_then(Value::as_str) != Some("3.1.0") {
            bail!("vendored producer contract must be OpenAPI 3.1.0");
        }
        Ok(openapi)
    }

    fn resolve_schema<'a>(document: &'a Value, schema: &'a Value) -> Result<&'a Value> {
        let mut current = schema;
        for _ in 0..64 {
            let Some(reference) = current.get("$ref").and_then(Value::as_str) else {
                return Ok(current);
            };
            let pointer = reference
                .strip_prefix('#')
                .ok_or_else(|| anyhow!("external schema reference is unsupported: {reference}"))?;
            current = document
                .pointer(pointer)
                .ok_or_else(|| anyhow!("unresolved schema reference {reference}"))?;
        }
        bail!("schema reference depth exceeded")
    }

    fn schema_types(document: &Value, schema: &Value) -> Result<BTreeSet<String>> {
        let schema = resolve_schema(document, schema)?;
        let mut types = BTreeSet::new();
        match schema.get("type") {
            Some(Value::String(value)) => {
                types.insert(value.clone());
            }
            Some(Value::Array(values)) => {
                for value in values {
                    let value = value
                        .as_str()
                        .ok_or_else(|| anyhow!("schema type array contains a non-string"))?;
                    types.insert(value.to_string());
                }
            }
            Some(_) => bail!("schema type must be a string or string array"),
            None if schema.get("properties").is_some() => {
                types.insert("object".to_string());
            }
            None => bail!("schema has no supported type: {schema}"),
        }
        Ok(types)
    }

    fn required_properties(schema: &Value) -> Result<BTreeSet<&str>> {
        match schema.get("required") {
            None => Ok(BTreeSet::new()),
            Some(Value::Array(values)) => values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| anyhow!("required property name must be a string"))
                })
                .collect(),
            Some(_) => bail!("schema required must be an array"),
        }
    }

    fn compare_schema(
        producer_document: &Value,
        producer_schema: &Value,
        consumer_document: &Value,
        consumer_schema: &Value,
        pointer: &str,
        depth: usize,
    ) -> Result<()> {
        if depth > 64 {
            bail!("schema comparison depth exceeded at {pointer}");
        }
        let producer = resolve_schema(producer_document, producer_schema)?;
        let consumer = resolve_schema(consumer_document, consumer_schema)?;
        for keyword in ["oneOf", "anyOf", "allOf", "not"] {
            if producer.get(keyword).is_some() || consumer.get(keyword).is_some() {
                bail!("unsupported {keyword} at {pointer}");
            }
        }

        let producer_types = schema_types(producer_document, producer)?;
        let consumer_types = schema_types(consumer_document, consumer)?;
        let producer_nullable = producer_types.contains("null");
        let consumer_nullable = consumer_types.contains("null");
        if producer_nullable && !consumer_nullable {
            bail!("producer nullable value is rejected by consumer at {pointer}");
        }
        if !producer_nullable && consumer_nullable {
            bail!("narrow lint: producer non-null value is optional at {pointer}");
        }

        let producer_non_null: BTreeSet<_> = producer_types
            .iter()
            .filter(|kind| kind.as_str() != "null")
            .collect();
        let consumer_non_null: BTreeSet<_> = consumer_types
            .iter()
            .filter(|kind| kind.as_str() != "null")
            .collect();
        if producer_non_null != consumer_non_null {
            bail!(
                "JSON type mismatch at {pointer}: producer={producer_non_null:?} consumer={consumer_non_null:?}"
            );
        }

        if consumer_non_null.contains(&"object".to_string()) {
            let producer_properties = producer
                .get("properties")
                .and_then(Value::as_object)
                .ok_or_else(|| anyhow!("producer object has no properties at {pointer}"))?;
            let consumer_properties = consumer
                .get("properties")
                .and_then(Value::as_object)
                .ok_or_else(|| anyhow!("consumer object has no properties at {pointer}"))?;
            let producer_required = required_properties(producer)?;
            let consumer_required = required_properties(consumer)?;

            for (name, consumer_property) in consumer_properties {
                let child_pointer = format!("{pointer}/{name}");
                let producer_property = producer_properties.get(name).ok_or_else(|| {
                    anyhow!("consumer property is absent from producer at {child_pointer}")
                })?;
                if consumer_required.contains(name.as_str())
                    && !producer_required.contains(name.as_str())
                {
                    bail!("producer optional property is required by consumer at {child_pointer}");
                }
                if producer_required.contains(name.as_str())
                    && !consumer_required.contains(name.as_str())
                {
                    bail!("narrow lint: producer required property is optional at {child_pointer}");
                }
                compare_schema(
                    producer_document,
                    producer_property,
                    consumer_document,
                    consumer_property,
                    &child_pointer,
                    depth + 1,
                )?;
            }
        } else if consumer_non_null.contains(&"array".to_string()) {
            let producer_items = producer
                .get("items")
                .ok_or_else(|| anyhow!("producer array has no items at {pointer}"))?;
            let consumer_items = consumer
                .get("items")
                .ok_or_else(|| anyhow!("consumer array has no items at {pointer}"))?;
            compare_schema(
                producer_document,
                producer_items,
                consumer_document,
                consumer_items,
                &format!("{pointer}/*"),
                depth + 1,
            )?;
        }

        if let Some(consumer_enum) = consumer.get("enum").and_then(Value::as_array) {
            let producer_enum =
                producer
                    .get("enum")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        anyhow!("consumer narrows an unconstrained producer enum at {pointer}")
                    })?;
            for value in producer_enum {
                if !consumer_enum.contains(value) {
                    bail!("consumer enum rejects producer value {value} at {pointer}");
                }
            }
        }
        Ok(())
    }

    fn operation_schema<'a>(
        openapi: &'a Value,
        contract: &ContractRegistration,
    ) -> Result<&'a Value> {
        openapi
            .get("paths")
            .and_then(|paths| paths.get(contract.path))
            .and_then(|path| path.get(contract.method.to_ascii_lowercase()))
            .and_then(|operation| operation.get("responses"))
            .and_then(|responses| responses.get(contract.success_status.to_string()))
            .and_then(|response| response.get("content"))
            .and_then(|content| content.get("application/json"))
            .and_then(|media| media.get("schema"))
            .ok_or_else(|| {
                anyhow!(
                    "missing producer response schema for {} {} status {}",
                    contract.method,
                    contract.path,
                    contract.success_status
                )
            })
    }

    fn sentinel_number(pointer: &str) -> i64 {
        1 + pointer.bytes().map(i64::from).sum::<i64>() % 10_000
    }

    fn generate_fixture(
        document: &Value,
        schema: &Value,
        include_optional: bool,
        pointer: &str,
        depth: usize,
    ) -> Result<Value> {
        if depth > 64 {
            bail!("fixture generation depth exceeded at {pointer}");
        }
        let schema = resolve_schema(document, schema)?;
        for keyword in ["oneOf", "anyOf", "allOf", "not"] {
            if schema.get(keyword).is_some() {
                bail!("fixture generator does not support {keyword} at {pointer}");
            }
        }
        if let Some(values) = schema.get("enum").and_then(Value::as_array) {
            return values
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("empty enum at {pointer}"));
        }

        let types = schema_types(document, schema)?;
        let kind = types
            .iter()
            .find(|kind| kind.as_str() != "null")
            .map(String::as_str)
            .unwrap_or("null");
        match kind {
            "object" => {
                let properties = schema
                    .get("properties")
                    .and_then(Value::as_object)
                    .ok_or_else(|| anyhow!("object has no properties at {pointer}"))?;
                let required = required_properties(schema)?;
                let mut output = serde_json::Map::new();
                for (name, property) in properties {
                    if include_optional || required.contains(name.as_str()) {
                        output.insert(
                            name.clone(),
                            generate_fixture(
                                document,
                                property,
                                include_optional,
                                &format!("{pointer}/{name}"),
                                depth + 1,
                            )?,
                        );
                    }
                }
                Ok(Value::Object(output))
            }
            "array" => Ok(Value::Array(vec![generate_fixture(
                document,
                schema
                    .get("items")
                    .ok_or_else(|| anyhow!("array has no items at {pointer}"))?,
                include_optional,
                &format!("{pointer}/0"),
                depth + 1,
            )?])),
            "string" => Ok(Value::String(format!("sentinel:{pointer}"))),
            "integer" => Ok(Value::Number(sentinel_number(pointer).into())),
            "number" => serde_json::Number::from_f64(sentinel_number(pointer) as f64 + 0.25)
                .map(Value::Number)
                .ok_or_else(|| anyhow!("cannot make number fixture at {pointer}")),
            "boolean" => Ok(Value::Bool(sentinel_number(pointer) % 2 == 0)),
            "null" => Ok(Value::Null),
            other => bail!("unsupported fixture type {other} at {pointer}"),
        }
    }

    fn assert_projected_values(
        consumer_document: &Value,
        consumer_schema: &Value,
        input: &Value,
        output: &Value,
        pointer: &str,
        depth: usize,
    ) -> Result<()> {
        if depth > 64 {
            bail!("value assertion depth exceeded at {pointer}");
        }
        let schema = resolve_schema(consumer_document, consumer_schema)?;
        let types = schema_types(consumer_document, schema)?;
        if types.contains("object") {
            let input = input
                .as_object()
                .ok_or_else(|| anyhow!("fixture input is not an object at {pointer}"))?;
            let output = output
                .as_object()
                .ok_or_else(|| anyhow!("round-trip output is not an object at {pointer}"))?;
            let properties = schema
                .get("properties")
                .and_then(Value::as_object)
                .ok_or_else(|| anyhow!("consumer object has no properties at {pointer}"))?;
            for (name, property) in properties {
                let Some(input_value) = input.get(name) else {
                    continue;
                };
                let output_value = output
                    .get(name)
                    .ok_or_else(|| anyhow!("consumer dropped fixture value at {pointer}/{name}"))?;
                assert_projected_values(
                    consumer_document,
                    property,
                    input_value,
                    output_value,
                    &format!("{pointer}/{name}"),
                    depth + 1,
                )?;
            }
        } else if types.contains("array") {
            let input = input
                .as_array()
                .ok_or_else(|| anyhow!("fixture input is not an array at {pointer}"))?;
            let output = output
                .as_array()
                .ok_or_else(|| anyhow!("round-trip output is not an array at {pointer}"))?;
            if input.len() != output.len() {
                bail!("consumer changed array length at {pointer}");
            }
            let items = schema
                .get("items")
                .ok_or_else(|| anyhow!("consumer array has no items at {pointer}"))?;
            for (index, (input_value, output_value)) in input.iter().zip(output.iter()).enumerate()
            {
                assert_projected_values(
                    consumer_document,
                    items,
                    input_value,
                    output_value,
                    &format!("{pointer}/{index}"),
                    depth + 1,
                )?;
            }
        } else if input != output {
            bail!("consumer changed fixture value at {pointer}: {input} -> {output}");
        }
        Ok(())
    }

    #[derive(Default)]
    struct DecodeCallVisitor {
        calls: Vec<(String, String)>,
    }

    impl<'ast> Visit<'ast> for DecodeCallVisitor {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            let method = call.method.to_string();
            if matches!(
                method.as_str(),
                "get"
                    | "get_query"
                    | "get_contract"
                    | "post"
                    | "post_once"
                    | "patch"
                    | "put"
                    | "delete_json"
            ) {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(path),
                    ..
                })) = call.args.first()
                {
                    self.calls.push((method, path.value()));
                }
            }
            syn::visit::visit_expr_method_call(self, call);
        }
    }

    fn verify_coverage_and_narrow_lint(contracts: &[ContractRegistration]) -> Result<()> {
        let syntax = syn::parse_file(ORG_SOURCE).context("parse org_cli.rs for coverage")?;
        let mut visitor = DecodeCallVisitor::default();
        visitor.visit_file(&syntax);

        let mut ids = BTreeSet::new();
        let mut operations = BTreeSet::new();
        for contract in contracts {
            if !ids.insert(contract.id) {
                bail!("duplicate contract id {}", contract.id);
            }
            if !operations.insert((contract.method, contract.path, contract.success_status)) {
                bail!(
                    "duplicate contract operation {} {}",
                    contract.method,
                    contract.path
                );
            }
            if contract.source != ContractSource::OpenApiOperation {
                bail!("auth/list contract {} is not OpenAPI-enforced", contract.id);
            }
            if contract.api_owner.is_empty() || contract.cli_owner.is_empty() {
                bail!("contract {} has an empty owner", contract.id);
            }

            let matching: Vec<_> = visitor
                .calls
                .iter()
                .filter(|(_, path)| path == contract.path)
                .collect();
            if matching.len() != 1 || matching[0].0 != "get_contract" {
                bail!(
                    "{} must have exactly one enrolled get_contract call; found {matching:?}",
                    contract.id
                );
            }

            let start_marker = format!("// response-contract:{}:start", contract.id);
            let end_marker = format!("// response-contract:{}:end", contract.id);
            let block = ORG_SOURCE
                .split_once(&start_marker)
                .and_then(|(_, suffix)| suffix.split_once(&end_marker).map(|(block, _)| block))
                .ok_or_else(|| anyhow!("missing narrow-lint markers for {}", contract.id))?;
            if block.contains("serde(default") {
                bail!(
                    "narrow lint: serde(default) is forbidden for {}",
                    contract.id
                );
            }
            if block.contains("alias") {
                bail!("narrow lint: serde alias is forbidden for {}", contract.id);
            }
        }
        Ok(())
    }

    fn verify_contract(openapi: &Value, contract: &ContractRegistration) -> Result<usize> {
        let producer_schema = operation_schema(openapi, contract)?;
        let consumer_document = (contract.consumer_schema)();
        compare_schema(
            openapi,
            producer_schema,
            &consumer_document,
            &consumer_document,
            contract.id,
            0,
        )?;

        for include_optional in [false, true] {
            let fixture =
                generate_fixture(openapi, producer_schema, include_optional, contract.id, 0)?;
            let round_trip = (contract.round_trip)(&fixture)?;
            assert_projected_values(
                &consumer_document,
                &consumer_document,
                &fixture,
                &round_trip,
                contract.id,
                0,
            )?;
        }
        Ok(consumer_document
            .get("$defs")
            .and_then(Value::as_object)
            .map_or(0, serde_json::Map::len))
    }

    #[test]
    #[ignore = "run by the non-required Response contract workflow"]
    fn response_contract_gate_auth_list() -> Result<()> {
        let openapi = verify_vendored_bundle()?;
        let contracts = crate::org_cli::auth_list_contracts();
        verify_coverage_and_narrow_lint(&contracts)?;

        let expected: BTreeSet<(&str, &str, &str, u16)> = BTreeSet::from([
            (
                "auth.operators.list",
                "GET",
                "/v1/auth/operators/by-user",
                200,
            ),
            ("auth.actions.list", "GET", "/v1/auth/actions", 200),
        ]);
        let actual: BTreeSet<(&str, &str, &str, u16)> = contracts
            .iter()
            .map(|contract| {
                (
                    contract.id,
                    contract.method,
                    contract.path,
                    contract.success_status,
                )
            })
            .collect();
        if actual != expected {
            bail!("auth/list registry coverage changed: {actual:?}");
        }

        let mut nested_total = 0;
        for contract in &contracts {
            let nested = verify_contract(&openapi, contract)
                .with_context(|| format!("contract {} failed", contract.id))?;
            nested_total += nested;
        }
        println!("family=auth/list");
        println!(
            "roots_total={} nested_total={nested_total}",
            contracts.len()
        );
        println!(
            "openapi_enforced={} fixture_backed_gap=0 temporary_exemption=0",
            contracts.len()
        );
        println!("unregistered=0 expired=0");
        Ok(())
    }

    #[derive(Debug, Deserialize, Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    struct RenamedConsumer {
        service: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    struct DefaultedRenamedConsumer {
        #[serde(default)]
        service: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    struct ExtraRequiredConsumer {
        service_name: String,
        required_extra: String,
    }

    #[derive(Debug, Deserialize, Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    struct OptionalConsumer {
        service_name: Option<String>,
    }

    #[test]
    #[ignore = "run by the non-required Response contract workflow"]
    fn response_contract_gate_synthetic_regressions_fail_closed() -> Result<()> {
        let producer = serde_json::json!({
            "type": "object",
            "required": ["serviceName"],
            "properties": { "serviceName": { "type": "string" } }
        });
        for consumer in [
            serde_json::to_value(schemars::schema_for!(RenamedConsumer))?,
            serde_json::to_value(schemars::schema_for!(DefaultedRenamedConsumer))?,
            serde_json::to_value(schemars::schema_for!(ExtraRequiredConsumer))?,
            serde_json::to_value(schemars::schema_for!(OptionalConsumer))?,
        ] {
            if compare_schema(&producer, &producer, &consumer, &consumer, "synthetic", 0).is_ok() {
                bail!("synthetic incompatible consumer unexpectedly passed");
            }
        }
        Ok(())
    }
}
