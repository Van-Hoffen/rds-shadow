# Handoff: RDS Shadow

- **Date:** 2026-11-18 09:44 UTC
- **Status:** TZ draft saved; project skill installed; no product code; no git remote
- **Docs:** docs/STATUS.md
- **TZ:** docs/architecture/tz-v1-draft.md
- **Decision:** docs/decisions/0001-v1-scope.md
- **Pipeline:** docs/PIPELINE.md (`approved: false`; proposed `hybrid`)

## Now

- User can approve the TZ and pipeline.
- Do not implement the client yet.
- Do not treat syslog or screenshots as v1.

## Done this session

- TZ draft updated: files-only audit, broker or server list via INI, Windows shadow client, Linux/Windows viewer.
- Installed project skill `.opencode/skills/rds-shadow-domain/SKILL.md`.
- Filled `AGENTS.md` gates and the optional `project-orchestrator` overlay.
- Installed Rust tooling (user-approved, MIT): agent `rust-pro`, skills `rust-async-patterns`, `memory-safety-patterns` from wshobson/agents. Bootstrapped global pack was already on disk.
- Did not install third-party agents outside the approved list. Evidence: `opencode v2.3.2`.

## Not done / blockers

- GitHub repo was requested earlier. `gh` is not installed here, so the remote was not created.
- No Windows RDS stand. Consent policy on the farm is not verified.
- Pipeline not approved.

## Next steps (max 3)

1. Approve `docs/architecture/tz-v1-draft.md`.
2. Approve pipeline mode `hybrid`.
3. Install `gh` and create the GitHub repo only after an explicit go.

## Danger / do not

- Do not add a consent bypass.
- Do not hardcode hosts or group names.
- Do not logoff a production session as a test.
- Do not commit secrets.
