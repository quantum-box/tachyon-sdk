# Changelog

## Unreleased

### Added

- Add `tachyon compute deployments rollback --to-previous` to roll back to the newest production rollback candidate without picking a deployment ID, and `tachyon compute deployments rollback-candidates` to list the eligible targets (successful serving record, provider metadata present) newest first. During an incident the deployment list mixes failed and deleted rows, so finding a valid rollback target used to require manual archaeology. (PLT-3787)

- Add `requires` to auth manifest `ActionSpec`: each entry names a full action (`context:Name`) the declaring action depends on within a call context. `manifest apply` sends the declared edge set to `PUT /v1/auth/action-dependencies`, which validates ownership, cycles, and the system-action allowlist server-side. A manifest without `requires` never touches the platform's edge set. (PLT-3597)

### Fixed

- Update existing auth manifest policies through the generated SDK PATCH API instead of reporting duplicate POST responses as a successful skip. Existing policy updates now report `updated`, failures exit non-zero, and output explicitly warns that the current API cannot verify or remove stale action membership. (PLT-3670)

- Send `resourcePattern` in camelCase when registering actions from an auth manifest. The server silently ignored the snake_case key, so a declared `resource_pattern` was never stored and `manifest plan` reported a permanent diff. (PLT-3597)

- Stream Cloud App build command output while retaining a bounded callback tail, allowing the control plane to distinguish active long builds from output stalls. (PLT-3402)

## tachyon-cli-v0.6.32

### Added

- Add `--limit`, `--cursor`, and `--all` to `tachyon pm issue list` and `tachyon linear issue list`. The command previously had no way to page and stopped at 100 issues, so older issues could not be listed at all. (PLT-3235)

### Fixed

- Warn on stderr when `issue list` returns a truncated result. The 100-issue cutoff used to be invisible, so a full page looked identical to the end of the list and callers could wrongly conclude they had seen every issue. The warning names the next cursor and is emitted under `--json` too, keeping stdout parseable. (PLT-3235)

## tachyon-cli-v0.6.31

### Added

- Add `tachyon compute apps update <app-id> --connection-id <connection-id>` to preview and update a compute app's GitHub connection. Pass an empty connection ID to clear it. (PLT-3092)
- Publish statically linked musl release assets (`tachyon-linux-musl-x86_64.tar.gz`, `tachyon-linux-musl-arm64.tar.gz`). The existing gnu assets are built on Ubuntu 24.04 and require GLIBC_2.39, so they cannot start on older images such as Amazon Linux 2023 (glibc 2.34) used by the CodeBuild Cloud App build runners. (PLT-3176)
- Show the Tachyon IaC step result in `tachyon compute builds get`. An IaC failure never appears in `error_message`, which carries the tail of the application build log, so a build that failed in IaC used to look like a clean compile. (PLT-3176)

## tachyon-cli-v0.6.5 - 2026-05-25

### Fixed

- Persist CLI login credentials and refresh them when an API request returns 401. (#114)
- Request the Cognito `aws.cognito.signin.user.admin` scope during CLI login so authenticated user calls are authorized. (#115)
- Allow slash-separated secret paths in `valueFrom.secret` manifest references. (#116)
