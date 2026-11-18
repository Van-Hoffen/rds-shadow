# Project skills — RDS Shadow

| ID | Path | Purpose |
|----|------|---------|
| `project-docs` | `.opencode/skills/project-docs/` | docs layout and session notes |
| `rds-shadow-domain` | `.opencode/skills/rds-shadow-domain/` | consent, config, audit, and logoff gates |
| `rust-async-patterns` | `.opencode/skills/rust-async-patterns/` | Tokio/async patterns (MIT, wshobson/agents) |
| `memory-safety-patterns` | `.opencode/skills/memory-safety-patterns/` | RAII/ownership/unsafe discipline (MIT, wshobson/agents) |

Global skills are not copied here. They already load from `~/.config/opencode/skills`. Same ID in this tree would override the global skill.

Use these global IDs, do not reinstall them:

`project-orchestration`, `docs-packet`, `session-handoff`, `technical-writing`, `writing-plans`, `test-driven-development`, `systematic-debugging`, `verification-before-completion`, `subagent-driven-development`, `docs-sync-audit`.

Not used in v1: `api-documentation` (no HTTP API), `documentation-strategy` (no docs-system redesign yet).

## Project agents

| ID | Path | Purpose |
|----|------|---------|
| `rust-pro` | `.opencode/agents/rust-pro.md` | Rust 1.75+/async/systems expertise (MIT, wshobson/agents; model field removed, project constraints appended) |

Global agents remain the default pool: `pm-orchestrator` (primary), `code-explorer`, `implementer`, `reviewer`, `docs-writer`, `researcher`.
