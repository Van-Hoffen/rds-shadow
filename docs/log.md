# Log

Append-only. Do not rewrite old entries.

## YYYY-MM-DD — bootstrap
- Changes: created from `/home/pmb/template-project/`
- Evidence: skeleton files present
- Docs: [[STATUS]]

## 2026-11-18 10:25 UTC — v0.1.0 released
- Changes: tag `v0.1.0`, 0.x marked pre-release
- Evidence: Release #1 success; assets `rds-shadow-windows-x64.zip` (707489 B), `rds-shadow-audit-linux-x64.tar.gz` (391970 B)
- Docs: [[guides/release]], [[sessions/2026-11-18-bootstrap-repo]]

## 2026-11-18 10:15 UTC — CI green
- Changes: release workflow triggers only on tags; CI badge in README
- Evidence: Actions run #1: Linux success, Windows success
- Docs: [[sessions/2026-11-18-bootstrap-repo]]

## 2026-11-18 10:10 UTC — repo bootstrap
- Changes: Cargo workspace, Windows client, audit viewer, CI/release workflows, README, release guide
- Evidence: 19 tests, clippy clean, Windows gnu target check, commit `9b86f91` pushed to `main`
- Docs: [[sessions/2026-11-18-bootstrap-repo]], [[decisions/0002-repo-ci-release]]

## 2026-11-18 09:49 UTC — rust tooling installed
- Changes: `.opencode/agents/rust-pro.md`, `.opencode/skills/rust-async-patterns/`, `.opencode/skills/memory-safety-patterns/` (from wshobson/agents, MIT)
- Evidence: files written; `model: opus` removed; references copied; user approved
- Docs: [[decisions/0001-v1-scope]]

## 2026-11-18 09:44 UTC — state save
- Changes: gates in `AGENTS.md`; skill `rds-shadow-domain`; overlay filled; decision `docs/decisions/0001-v1-scope.md`
- Evidence: skill file written locally; no network skill download; `opencode v2.3.2`
- Docs: [[sessions/2026-11-18-state-save]]

## 2026-11-18 — TZ audit cut
- Changes: v1 journal is files only; syslog/collector and screenshot stream moved out; broker/INI/farm facts written into `docs/architecture/tz-v1-draft.md`
- Evidence: user messages this session; no code; no farm query
- Docs: [[STATUS]]

## 2026-11-18 — TZ draft
- Changes: `docs/architecture/tz-v1-draft.md` updated with locked choices (bypass accepted for v1, audit fail-closed, full message text, ru+en UI)
- Evidence: user answers in session; no code, no stand check
- Docs: [[STATUS]]

## 2026-11-18 — research baseline
- Changes: разбор публичного `rds_shadow_connect`; зафиксированы решения по клиенту, допуску, согласию и аудиту
- Evidence: raw script прочитан; brief `docs/research/2026-11-18-rds-shadow-baseline.md`
- Docs: [[STATUS]]
