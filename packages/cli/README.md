# @tachyon-sdk/cli

Global npm installer for the Tachyon CLI.

## Install

```sh
npm install -g @tachyon-sdk/cli
```

This installs the `tachyon` command, plus the short alias `tc`.

```sh
tachyon --version
tc --version
```

During `postinstall`, the package downloads the matching native binary from
the `quantum-box/tachyon-sdk` GitHub Release. The npm package version matches
the CLI release version, so `@tachyon-sdk/cli@0.6.14` downloads
`tachyon-cli-v0.6.14`.

Supported platforms:

| OS | Architecture | Release asset |
| --- | --- | --- |
| Linux | x64 | `tachyon-linux-x86_64.tar.gz` |
| Linux | arm64 | `tachyon-linux-arm64.tar.gz` |
| macOS | arm64 | `tachyon-darwin-arm64.tar.gz` |
| macOS | x64 | `tachyon-darwin-x86_64.tar.gz` |

## Configuration

The npm installer uses these optional environment variables:

| Variable | Purpose |
| --- | --- |
| `TACHYON_CLI_SKIP_DOWNLOAD=1` | Skip binary download during install. The wrapper will try again on first run. |
| `TACHYON_CLI_VERSION` | Override the version used to build the default release tag. |
| `TACHYON_CLI_TAG` | Override the release tag, for example `tachyon-cli-v0.6.14`. |
| `TACHYON_CLI_REPOSITORY` | Override the GitHub repository, default `quantum-box/tachyon-sdk`. |
| `TACHYON_CLI_DOWNLOAD_URL` | Override the full `.tar.gz` download URL. |

## Cloud App internalService Support

`@tachyon-sdk/cli@0.6.13` is the first release line that can apply Cloud App
manifests with `envVars[].valueFrom.internalService`. The CLI collects those
references, calls the control-plane preflight endpoint before app mutations,
and then lets the server-side manifest apply path persist the unresolved
reference for deploy-time resolution.

CLI `0.6.12` does not support this field. Run `tachyon self-update` or install
`@tachyon-sdk/cli@0.6.13` before applying manifests that depend on internal
Cloud App service URLs.

## Alternative Install

The standalone installer remains available:

```sh
curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh
```
