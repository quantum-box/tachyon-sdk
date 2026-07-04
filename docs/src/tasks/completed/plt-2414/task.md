# PLT-2414 — tachyon CLI PM/Linear command surface inventory

- Linear: PLT-2414
- GitHub mirror: quantum-box/tachyon-apps#6298
- Scope: documentation / inventory only
- Repository: tachyon-sdk

## Background

The CEO asked for a concrete inventory of how much of the PM/Linear workflow
is covered by the `tachyon` CLI. This task records the current command surface
so future Project, Initiative, Cycle, Label, and Comment CRUD work can be
planned from a fixed baseline.

## Primary Sources

- `cli/src/pm_cli.rs`
- `cli/src/linear_cli.rs`
- `tachyon linear issue --help`
- `tachyon linear issue create --help`
- `tachyon linear issue list --help`
- `tachyon linear issue update --help`
- `tachyon-apps:apps/tachyon-api/src/router.rs`
- `tachyon-apps:apps/tachyon-api/src/roadmap.rs`

## Supported Today

The CLI currently supports tenant-scoped issue create, list, and update.

```bash
tachyon linear issue create --title <TITLE> \
  [--description <TEXT>] \
  [--team <TEAM> | --team-id <TEAM_ID>] \
  [--assignee-id <USER_ID>] \
  [--label-id <LABEL_ID>]... \
  [--priority <PRIORITY>] \
  [--project <PROJECT> | --project-id <PROJECT_ID>] \
  [--due-date YYYY-MM-DD] \
  [--related-issue-id <ISSUE_ID>]... \
  [--skip-if-exists] \
  [--json]
```

```bash
tachyon linear issue list \
  [--team <TEAM> | --team-id <TEAM_ID>] \
  [--project <PROJECT> | --project-id <PROJECT_ID>] \
  [--include-completed] \
  [--json]
```

```bash
tachyon linear issue update <ISSUE_ID> \
  [--status <STATUS> | --status-id <STATUS_ID>] \
  [--state <STATE> | --state-id <STATE_ID>] \
  [--title <TITLE>] \
  [--assignee-id <USER_ID>] \
  [--priority <PRIORITY>] \
  [--json]
```

The provider-agnostic `tachyon pm issue ...` and the Linear-specific
`tachyon linear issue ...` paths share the same implementation. `linear_cli.rs`
is a thin compatibility wrapper over `pm_cli::run_issue(...)` with the provider
fixed to `linear`.

## Backend Routes Used By The CLI

The supported issue operations are backed by the Tachyon API PM endpoints:

- `POST /v1beta/:tenant_id/pm/issues`
- `GET /v1beta/:tenant_id/pm/issues`
- `PATCH /v1beta/:tenant_id/pm/issues/:issue_id`

The issue list path requires roadmap read authorization. The create/update
paths require the corresponding roadmap mutation authorization.

## Not Supported Yet

The following operations are not exposed by the current CLI PM/Linear command
surface:

- Issue get by id/key
- Issue delete
- Issue comments: list/create/update/delete
- Updating labels after issue creation
- Project CRUD
- Initiative CRUD
- Cycle CRUD
- Team CRUD
- Label CRUD
- Document CRUD
- Milestone CRUD
- Sub-issue hierarchy operations

`--project` and `--project-id` only filter issue lists or set issue membership
at creation time. They are not Project CRUD commands.

## Conclusion

As of this inventory, `tachyon` CLI PM/Linear support is intentionally narrow:
issue create/list/update only. Project, Initiative, Cycle, Team, Label,
Document, Milestone, Comment, and hierarchy operations need explicit follow-up
issues before they can be treated as supported CLI workflows.

## Validation

- `tachyon linear issue --help`
- `tachyon linear issue create --help`
- `tachyon linear issue list --help`
- `tachyon linear issue update --help`

No env, secret, production, or txcloud-proxy changes were made.
