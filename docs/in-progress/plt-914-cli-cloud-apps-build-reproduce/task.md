# PLT-914 — `tachyon compute builds reproduce` subcommand

- Linear: PLT-914 (Backlog / High, blocked by PLT-913 = tachyon-api build-config endpoint)
- Branch: `feature/plt-914-cli-cloud-apps-build-reproduce`
- Worktree: `~/tachyon-sdk.leader-plt841`
- Leader: leader-plt914 (work:6, account2)
- ETA: Phase 1 = 1–2 d / Phase 2 = +1–2 d after PLT-913 lands

## Goal

Allow developers to reproduce a Cloud Apps (Compute) cloud build locally:

```
tachyon compute builds reproduce <build_id> [--dry-run] [--image <ecr-image>]
```

Flow:
1. Fetch buildspec + env via tachyon-api endpoint (PLT-913) → Phase 1 uses local mock fixture.
2. Parse buildspec.yml (install/pre_build/build/post_build phases).
3. Run phases in a CodeBuild-compatible Docker container (`public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0` default).
4. Surface exit code + log to stdout/stderr.

## Naming decision

Spec was ambiguous (`cloud-apps build reproduce` vs "compute_cli.rs 拡張 OK").
Going with **`tachyon compute builds reproduce`** to match the existing `Compute::Builds` namespace
(parallel to Trigger / Cancel / Logs). No parallel `cloud-apps` top-level — codebase consistency wins.

The legacy `Compute::Build` (singular) at `cli/src/compute_cli.rs:52` is unrelated — it is a Cloudflare
Pages local-build helper for hardcoded apps (tachyon/cms/docs). New work lives under `Builds` (plural).

## Open question for PdM-PF

`<build_id>` vs `<deployment_id>` arg name — spec says deployment_id, but the existing CLI keys off
build_id. Phase 1 mock works for either; must be confirmed with PLT-913 endpoint shape before Phase 2.
Surfacing in 着手宣言.

## Phase 1 (mock, this PR) — DONE, PR #49 merged 2026-04-25

- [x] (a) Add `BuildsCommand::Reproduce` to `compute_cli.rs` with clap derive
- [x] (b) `serde_yaml` dependency + buildspec parser
- [x] (c) Docker wrapper using `std::process::Command` (or `--dry-run` for tests)
- [x] (d) `tests/fixtures/mock-build-config.yaml` fixture + parser unit test
- [x] (e) `--dry-run` E2E test that asserts the docker invocation without running docker
- [x] (f) README update — document subcommand + Phase 2 future-work note
- [x] (g) PR draft → CI green → admin merge

Followup landed in same PR: mirror `$CODEBUILD_SRC_DIR` env var (advisor-flagged
foot-gun — buildspecs frequently `cd $CODEBUILD_SRC_DIR/sub`).

## Phase 2 (real endpoint, separate PR after PLT-913)

- [ ] Switch mock to `ApiClient::get` against PLT-913 endpoint
- [ ] Auth (existing Bearer + x-operator-id flow via `ApiClient::new`)
- [ ] Integration test against staging tachyon-api

## Constraints

- No direct push to main — PR only
- Admin merge bypass authorized (CEO 直命「治るまでいちいち聞かない」)
- Mock-only state must be flagged in PR description + README
- aws-codebuild-docker-images = pull only, no custom build
- Test strategy: parser unit tests + `--dry-run` CLI tests (no docker-in-CI)

## Primary sources

- `cli/src/compute_cli.rs:309` (BuildsCommand) — extension point
- `cli/src/client.rs` (ApiClient) — auth/HTTP wrapper to reuse for Phase 2
- `cli/src/main.rs:72` (Compute subcommand wiring)
- aws-codebuild-docker-images: `public.ecr.aws/codebuild/amazonlinux-x86_64-standard:5.0` (verified
  via web search, 2026-04: AL2023-based, default standard image)
