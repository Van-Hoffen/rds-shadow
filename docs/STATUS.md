# Status

**Updated:** 2026-11-18 09:44 UTC

## One-paragraph state

Кода нет. Черновик ТЗ v1 лежит в `docs/architecture/tz-v1-draft.md` и не утвержден. Журнал v1 — только JSONL, действие не блокируется недоступной шарой. Проектные gates записаны в `AGENTS.md`. Добавлен навык `rds-shadow-domain`. Глобальные агенты не копировались и не скачивались.

## Working

- Каркас docs и primary `pm-orchestrator`
- Черновик ТЗ и пример INI без боевых имён
- Навык `rds-shadow-domain` и заполненный overlay `project-orchestrator`
- Rust-инструментарий: agent `rust-pro` + скилы `rust-async-patterns`, `memory-safety-patterns` (MIT)

## Broken / unknown

- `docs/PIPELINE.md` всё ещё `approved: false`
- Нет git-репозитория и GitHub. `gh` и `cargo` в этой среде отсутствуют
- Нет тестового узла RDS. Shadow отсюда не проверяется
- Не подтверждено, что специалист всегда работает из Windows-сеанса

## Next

1. Утвердить ТЗ и pipeline `hybrid`
2. По явному заданию — git и GitHub, когда появится `gh`
3. Код не начинать до утверждения ТЗ
