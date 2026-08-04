# Changelog

## Unreleased

### Added

- Add `tachyon compute apps update <app-id> --connection-id <connection-id>` to preview and update a compute app's GitHub connection. Pass an empty connection ID to clear it. (PLT-3092)

### Fixed

- Send `isSystem: true` for `global` policies in `tachyon auth manifest apply`. The request previously hard-coded `isSystem: false` while omitting `tenantId`, a combination `POST /v1/auth/policies` always rejects with "Custom policies require a tenant scope", so every global policy failed to apply. (PLT-3096)

## tachyon-cli-v0.6.5 - 2026-05-25

### Fixed

- Persist CLI login credentials and refresh them when an API request returns 401. (#114)
- Request the Cognito `aws.cognito.signin.user.admin` scope during CLI login so authenticated user calls are authorized. (#115)
- Allow slash-separated secret paths in `valueFrom.secret` manifest references. (#116)
