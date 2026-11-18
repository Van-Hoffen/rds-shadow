# 2026-11-18 — save TZ state and install project skill

## Goal

- Сохранить состояние ТЗ и зафиксировать, какие агенты и навыки проекту нужны.

## Class

meta

## Changes

- `AGENTS.md` — имя проекта и domain gates.
- `.opencode/skills/rds-shadow-domain/SKILL.md` — новый проектный навык.
- `.opencode/skills/project-docs/SKILL.md` — плейсхолдеры заменены.
- `.opencode/agents/project-orchestrator.md` — overlay заполнен, default не менялся.
- `.opencode/SKILLS.md` — инвентарь.
- `docs/decisions/0001-v1-scope.md` — почему глобальные роли не копировались.

## Evidence

- `opencode v2.3.2` на машине. Каталога `opencode skill install` в CLI нет.
- Глобальные агенты уже были: `pm-orchestrator`, `code-explorer`, `implementer`, `reviewer`, `docs-writer`, `researcher`.
- Новый файл навыка записан в `.opencode/skills/rds-shadow-domain/`. Скачивания из сети не было.

## Decisions

- Не копировать глобальные агенты в проект.
- Не ставить непроверенные навыки из чужих репозиториев.
- Primary не переключать.

## Docs tier

L2 (gates и ТЗ)

## Follow-up: Rust skill install

- Installed (user-approved): `rust-pro` agent + `rust-async-patterns` + `memory-safety-patterns` (MIT, wshobson/agents).
- Evidence: files under `.opencode/agents/`, `.opencode/skills/`; frontmatter license added; `model: opus` removed; references copied.
- Docs: decision [[0001-v1-scope]] updated.

## Next

1. Утвердить `docs/architecture/tz-v1-draft.md` и режим `hybrid` в `docs/PIPELINE.md`.
2. Подтвердить, что у специалиста есть Windows-сеанс.
3. Репозиторий GitHub и релиз — отдельная задача: `gh` в этой среде не установлен.
