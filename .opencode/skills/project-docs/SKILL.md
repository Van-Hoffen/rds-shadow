---
name: RDS Shadow docs
description: Project documentation layout and session update rules for rds-shadow. Use when updating docs/, STATUS, log, sessions, or HANDOFF for this project (project-docs, STATUS, session note).
---

# project-docs — RDS Shadow

## Layout

- `docs/index.md` — map
- `docs/STATUS.md` — current state
- `docs/log.md` — append-only
- `docs/PIPELINE.md` — approved pipeline
- `docs/sessions/` — session notes
- `docs/decisions/` — decisions/ADR
- `docs/research/` — research briefs
- `docs/guides/` — how-to
- `docs/architecture/` — explanation, including the v1 TZ
- `docs/templates/` — local templates
- `HANDOFF.md` — resume (may stay out of git)

## Workflow

1. Expect a **Docs Packet** from orchestrator (skill `docs-packet`).
2. Load `technical-writing` for style; tier from packet (L0–L3).
3. Update only affected pages; do not rewrite the whole vault.
4. `log.md` — append only.
5. No secrets, tokens, raw PII.
6. Do not edit code, `.opencode/` agents/skills, or commit.

## Session note minimum

- Goal / class of work
- Changes + file paths
- Evidence
- Decisions
- Next steps
- Links to STATUS / decisions / research
