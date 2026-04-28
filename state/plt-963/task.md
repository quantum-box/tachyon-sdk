# PLT-963 Task Log

## Scope

- Update Tachyon CLI image generation default model to `gpt-image-2`.
- Keep `cli/src/image_cli.rs` as the SSoT target.
- Do not touch `tachyon-apps/apps/tachyon-cli/`.

## Acceptance

- `tachyon image generate --help` lists `gpt-image-2` and shows it as the default model.
- Existing explicit model usage such as `-m gpt-image-1.5` remains accepted by the CLI.
- `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo build` pass.
- PR is opened against `quantum-box/tachyon-sdk`, CI is green, then admin-merged.

## Notes

- Background: CEO 2026-04-25 default model direction documented in `CLAUDE.md`.
- Parent follow-up: PLT-958 B-7 missed this CLI default/help-text update.
