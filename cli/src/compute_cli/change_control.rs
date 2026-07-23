//! Production apply change-control token verification.
//!
//! See ADR 0011 (`docs/src/adr/0011-change-control-token-gate.md`).
//!
//! IMPORTANT — trust boundary: this runs client-side in the operator's CLI
//! and can be bypassed by editing the CLI or calling the apply API directly.
//! It is therefore NOT an authorization boundary. It is an executor-side,
//! fail-fast confirmation that a change-control approval exists and is
//! current (process enforcement / defense-in-depth). The authoritative
//! approval check must live server-side in the apply API (ADR 0011 Layer 1).
//!
//! The CLI only *verifies* tokens; it never mints them. Minting is done
//! out-of-band by the change-control authority. Token values are never
//! echoed to stdout/stderr/logs.

use anyhow::{anyhow, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// Token envelope prefix. `tcct` = tachyon change-control token.
const TOKEN_PREFIX: &str = "tcct.v1.";
/// Env var carrying the HMAC verification key (never logged).
pub(crate) const VERIFICATION_KEY_ENV: &str = "TACHYON_CHANGE_CONTROL_VERIFICATION_KEY";

/// Claims carried by a `tcct.v1` token payload.
#[derive(Debug, Deserialize)]
struct ChangeControlClaims {
    /// Approved change-control reference (e.g. Linear issue id). Required.
    #[serde(rename = "ref")]
    approval_ref: String,
    /// Environment the approval is scoped to. Required.
    env: String,
    /// Expiry, unix seconds. Required.
    exp: i64,
    /// Issued-at, unix seconds. Informational.
    #[serde(default)]
    #[allow(dead_code)]
    iat: Option<i64>,
    /// Scope descriptor. Informational in v1.
    #[serde(default)]
    #[allow(dead_code)]
    scope: Option<String>,
    /// Approver identity. Informational.
    #[serde(default)]
    #[allow(dead_code)]
    by: Option<String>,
}

/// Result of a successful verification. Deliberately carries no secret
/// material so callers can log it safely.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct VerifiedApproval {
    pub(crate) approval_ref: String,
    /// True only when the signature was checked against a configured key.
    pub(crate) signature_verified: bool,
}

/// Verify a change-control token for a production apply.
///
/// This is pure: `now_unix` (clock) and `verification_key` (env) are
/// injected so the gate is deterministically testable and evaluated before
/// any API write (fail-fast).
///
/// What it validates:
/// - structural form `tcct.v1.<payload>.<sig>` (rejects arbitrary strings),
/// - decodable payload with required claims (`ref`, `env`, `exp`),
/// - `env` claim matches the apply environment,
/// - `exp` is in the future (rejects expired tokens),
/// - HMAC-SHA256 signature, *when* a verification key is configured
///   (rejects tampered payloads). Without a key, a structurally valid,
///   unexpired, env-matched token is accepted and `signature_verified` is
///   false so the caller can warn.
///
/// What it does NOT do: it is not an authorization boundary (see module
/// docs); it does not perform network/Linear lookups.
pub(crate) fn verify_change_control_token(
    token: &str,
    environment: &str,
    verification_key: Option<&[u8]>,
    now_unix: i64,
) -> Result<VerifiedApproval> {
    let token = token.trim();

    let body = token.strip_prefix(TOKEN_PREFIX).ok_or_else(|| {
        anyhow!(
            "change-control token is not a valid approval token \
             (expected a signed `{}` token, not a free-form string); \
             obtain one from the change-control approval process",
            TOKEN_PREFIX.trim_end_matches('.')
        )
    })?;

    let (payload_b64, signature_b64) = body
        .split_once('.')
        .ok_or_else(|| anyhow!("change-control token is malformed: missing signature segment"))?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| anyhow!("change-control token payload is not valid base64url"))?;
    let claims: ChangeControlClaims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| anyhow!("change-control token payload is not valid claims JSON"))?;

    if claims.approval_ref.trim().is_empty() {
        return Err(anyhow!(
            "change-control token is missing an approval reference (`ref`)"
        ));
    }

    if !env_matches(&claims.env, environment) {
        return Err(anyhow!(
            "change-control token is scoped to a different environment; \
             it does not authorize an apply to `{environment}`"
        ));
    }

    if now_unix >= claims.exp {
        return Err(anyhow!(
            "change-control token has expired; obtain a fresh approval token"
        ));
    }

    let signature_verified = match verification_key {
        Some(key) => {
            let signing_input = format!("{TOKEN_PREFIX}{payload_b64}");
            let expected = hmac_sha256(key, signing_input.as_bytes());
            let provided = URL_SAFE_NO_PAD
                .decode(signature_b64)
                .map_err(|_| anyhow!("change-control token signature is not valid base64url"))?;
            if !constant_time_eq(&expected, &provided) {
                return Err(anyhow!(
                    "change-control token signature is invalid (tampered or wrong key)"
                ));
            }
            true
        }
        None => false,
    };

    Ok(VerifiedApproval {
        approval_ref: claims.approval_ref,
        signature_verified,
    })
}

/// Environment comparison mirroring `is_production_environment` normalization
/// (case-insensitive, `prod` aliases `production`).
fn env_matches(claim_env: &str, apply_env: &str) -> bool {
    fn norm(s: &str) -> &str {
        match s.trim() {
            e if e.eq_ignore_ascii_case("prod") => "production",
            e => e,
        }
    }
    norm(claim_env).eq_ignore_ascii_case(norm(apply_env))
}

/// Constant-time byte comparison (length-independent short-circuit avoided).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// HMAC-SHA256 (RFC 2104) implemented over `sha2` to avoid a new dependency.
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut block_key = [0u8; BLOCK];
    if key.len() > BLOCK {
        let digest = Sha256::digest(key);
        block_key[..digest.len()].copy_from_slice(&digest);
    } else {
        block_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= block_key[i];
        opad[i] ^= block_key[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    outer.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Test-only token minting. The shipped CLI never mints tokens; this
    /// reproduces the change-control authority's signing for regression
    /// coverage. `key` here is a test fixture, not a real secret.
    fn issue(key: &[u8], claims: serde_json::Value) -> String {
        let payload = serde_json::to_vec(&claims).unwrap();
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload);
        let signing_input = format!("{TOKEN_PREFIX}{payload_b64}");
        let sig = hmac_sha256(key, signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
        format!("{TOKEN_PREFIX}{payload_b64}.{sig_b64}")
    }

    const KEY: &[u8] = b"plt-2722-test-verification-key";
    const NOW: i64 = 1_700_000_000;

    fn valid_claims() -> serde_json::Value {
        json!({
            "ref": "PLT-2722",
            "env": "production",
            "iat": NOW - 60,
            "exp": NOW + 3600,
        })
    }

    #[test]
    fn hmac_sha256_matches_rfc4231_test_case_2() {
        // RFC 4231, Test Case 2: key = "Jefe", data = "what do ya want ...".
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        let hex: String = mac.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn accepts_valid_signed_token() {
        let token = issue(KEY, valid_claims());
        let verified = verify_change_control_token(&token, "production", Some(KEY), NOW).unwrap();
        assert_eq!(verified.approval_ref, "PLT-2722");
        assert!(verified.signature_verified);
    }

    #[test]
    fn accepts_prod_alias_environment() {
        let token = issue(KEY, valid_claims());
        // apply env "prod" against a token scoped to "production".
        verify_change_control_token(&token, "prod", Some(KEY), NOW).unwrap();
    }

    #[test]
    fn rejects_arbitrary_non_empty_string() {
        // The previous gate accepted this; it must now be rejected.
        let err = verify_change_control_token("dummy-approved-token", "production", Some(KEY), NOW)
            .unwrap_err();
        assert!(err.to_string().contains("not a valid approval token"));
    }

    #[test]
    fn rejects_expired_token() {
        let token = issue(KEY, valid_claims());
        // now is past exp.
        let err =
            verify_change_control_token(&token, "production", Some(KEY), NOW + 7200).unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn rejects_tampered_payload() {
        let token = issue(KEY, valid_claims());
        // Flip the approval ref in the payload while keeping the old signature.
        let (_prefix, rest) = token.split_at(TOKEN_PREFIX.len());
        let (_payload_b64, sig) = rest.split_once('.').unwrap();
        let forged_payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({
                "ref": "PLT-9999",
                "env": "production",
                "exp": NOW + 3600,
            }))
            .unwrap(),
        );
        let forged = format!("{TOKEN_PREFIX}{forged_payload}.{sig}");
        let err = verify_change_control_token(&forged, "production", Some(KEY), NOW).unwrap_err();
        assert!(err.to_string().contains("signature is invalid"));
    }

    #[test]
    fn rejects_wrong_verification_key() {
        let token = issue(KEY, valid_claims());
        let err =
            verify_change_control_token(&token, "production", Some(b"wrong-key"), NOW).unwrap_err();
        assert!(err.to_string().contains("signature is invalid"));
    }

    #[test]
    fn rejects_environment_mismatch() {
        let token = issue(
            KEY,
            json!({"ref": "PLT-2722", "env": "staging", "exp": NOW + 3600}),
        );
        let err = verify_change_control_token(&token, "production", Some(KEY), NOW).unwrap_err();
        assert!(err.to_string().contains("different environment"));
    }

    #[test]
    fn rejects_empty_ref() {
        let token = issue(
            KEY,
            json!({"ref": "  ", "env": "production", "exp": NOW + 3600}),
        );
        let err = verify_change_control_token(&token, "production", Some(KEY), NOW).unwrap_err();
        assert!(err.to_string().contains("approval reference"));
    }

    #[test]
    fn accepts_structural_token_without_key_but_marks_unverified() {
        let token = issue(KEY, valid_claims());
        let verified = verify_change_control_token(&token, "production", None, NOW).unwrap();
        assert!(!verified.signature_verified);
    }

    #[test]
    fn without_key_still_rejects_expired_and_unstructured() {
        let expired = issue(
            KEY,
            json!({"ref": "PLT-2722", "env": "production", "exp": NOW - 1}),
        );
        assert!(verify_change_control_token(&expired, "production", None, NOW).is_err());
        assert!(verify_change_control_token("free-form", "production", None, NOW).is_err());
    }
}
