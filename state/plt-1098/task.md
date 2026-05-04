# PLT-1098 Task

## Scope

- Add `tachyon init` to generate a minimal `tachyon.yml` CloudApp config.
- Add repo-local `tachyon.yml` discovery so CLI defaults can come from project config.

## Acceptance

- `tachyon init --non-interactive --name ... --framework ... --tenant-id ...` creates `tachyon.yml`.
- Existing `tachyon.yml` is not overwritten unless `--force` is set.
- Runtime commands can load `metadata.name` and `metadata.tenant_id` from config.
- Config precedence is `TACHYON_CONFIG` > `--config` > cwd/parent discovery.
- Explicit CLI flags/arguments continue to override config defaults.

## Verification

- `cargo fmt --check`
- `cargo test`
- Real temp repo verification for `tachyon init` and config discovery.
