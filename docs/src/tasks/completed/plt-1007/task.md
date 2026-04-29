# PLT-1007 — `tachyon image generate` short alias `-r` / `--reference`

- Linear: PLT-1007 (parent=PLT-958)
- Branch: `feature/plt-1007-cli-shortalias`
- Scope: cosmetic only — no behavior change

## Background

PLT-958 (#61) shipped `tachyon image generate --reference-image <PATH>` for image-to-image
generation. The PLT-958 close-out called out a cosmetic short alias as deferred follow-up.
This task adds `-r` (clap `short`) and `--reference` (clap `alias`, hidden long form) so users
can type any of:

- `--reference-image path.png` (canonical, unchanged)
- `--reference path.png`
- `-r path.png`

All three resolve to the same `reference_images: Vec<String>` field. Repeatable for multiple
images, max 16, gpt-image-2 only — gating logic in `image_cli.rs` is untouched.

## Primary-source verification (origin/main `cli/src/image_cli.rs:64-65`)

```rust
#[arg(long = "reference-image", value_name = "PATH")]
reference_images: Vec<String>,
```

Confirmed: `short` absent, `alias` absent. Scope valid.

## Changes

1. `cli/src/image_cli.rs:64` — add `short = 'r'`, `alias = "reference"` to clap arg attribute.
2. Doc string above the arg — note the synonyms.
3. `cli/src/image_cli.rs` `mod tests` — add a clap parse test verifying `-r` and `--reference`
   parse into the same `reference_images` field.

No runtime / API / serialization changes.

## Verify checklist

- [ ] `cargo fmt` clean
- [ ] `cargo clippy -p cli -- -D warnings` clean
- [ ] `cargo test -p cli` all pass (existing 5 + new parse test)
- [ ] `tachyon image generate --help` shows `-r, --reference-image <PATH> [aliases: reference]`
- [ ] PR CI green (Chromatic / Amplify failures ignorable per repo policy)
- [ ] admin merge to main
