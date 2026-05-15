# PLT-1356 Tachyon Shared User Pool Client Issuance

## Overview

Add `tachyon auth issue` support for issuing an app-specific Cognito App Client from the Tachyon shared User Pool (`ap-northeast-1_8Ga4bK5M4`).

## Background

ERP / bakuure-api needs a one-command path to get a dedicated client ID without creating a new User Pool. PLT-1099 added the `auth` block and local credential output, but it does not yet model shared User Pool selection.

## Scope

- Add `auth.user_pool: shared` to `tachyon.yml` parsing and validation.
- Add `tachyon auth issue --shared-pool`.
- Send the shared pool selection to the backend credential issuance API.
- Persist `client_id`, optional `client_secret`, and `user_pool_id` to `.tachyon/credentials.json` with chmod 600.
- Keep existing new/default pool behavior compatible.

## Tasks

### Phase 1: Discovery ✅

- [x] Inspect existing `tachyon auth init/issue` implementation.
- [x] Inspect local credentials output and chmod behavior.
- [x] Confirm PLT-1356 / PLT-1099 / PLT-1104 requirements from Linear.

### Phase 2: CLI implementation ✅

- [x] Extend auth schema with `user_pool`.
- [x] Add `--shared-pool` flag and auto-detection from `tachyon.yml`.
- [x] Include `user_pool` in issue API request.
- [x] Store returned `user_pool_id`.

### Phase 3: Verification 🔄

- [x] Add CLI tests for shared pool schema and request body.
- [x] Run targeted CLI tests.
- [x] Record AWS live verification result or blocker.

## Verification

- `cargo test --manifest-path cli/Cargo.toml --test auth_cli`: PASS, 4 tests passed.
- AWS live Cognito creation was not executed from this workspace because AWS credentials/session were not available in the paired tachyon-apps environment. Backend mock/dev path verification covers request shape and `user_pool_id` persistence; live verification remains a PR environment follow-up.

## ERP Example

```yaml
auth:
  user_pool: shared
  providers:
    - name: bakuure-api
      type: oauth2_client_credentials
      audience: https://api.tachyon.cloud
```

Run:

```bash
tachyon auth issue bakuure-api
```

The command writes `.tachyon/credentials.json` locally for dev, or updates `secret_ref` for staging/prod, without printing secret material.
