# Changelog

## Unreleased

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
