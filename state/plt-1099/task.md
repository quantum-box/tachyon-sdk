# PLT-1099 Phase 1 Task

Status: in-progress
Owner: leader-plt1099-adr-codex
Branch: feature/plt-1099-auth-adr
Scope: ADR draft only; no implementation and no admin merge until CEO confirmation.

## Objective

Draft an ADR for adding top-level `auth:` provider definitions to `tachyon.yml` and for issuing client credentials through `tachyon auth` CLI flows.

## Phase 1 Checklist

- [x] Read PLT-1099 directive and confirm Phase 1 scope.
- [x] Send startup ack to PdM-PF work:17.
- [ ] Inspect existing CLI auth and config loader patterns.
- [ ] Inspect existing secret-management patterns from infrastructure references.
- [ ] Draft `state/plt-1099/adr-tachyon-yml-auth.md` with the required five chapters.
- [ ] Open PR targeting `main`.
- [ ] Send PR URL and Linear sub-issue request draft to PdM-PF work:17.
- [ ] Hold for COO/CEO ADR confirmation.

## Guardrails

- Do not include secret material in taskdocs, ADR, commits, PR descriptions, or Linear comments.
- Do not implement CLI behavior in Phase 1.
- Do not merge the ADR PR before CEO confirmation.
