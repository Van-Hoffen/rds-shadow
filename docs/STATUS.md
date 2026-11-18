# Status

**Updated:** 2026-11-18 10:10 UTC

## One-paragraph state

Каркас продукта в GitHub: `Van-Hoffen/rds-shadow`, ветка `main`, первый коммит `9b86f91`. Три крейта: ядро, Windows-клиент, просмотрщик. CI на Linux и Windows, релиз по тегу `v*`. Локально: 19 тестов, clippy чистый, Windows-цель собирается кросс-компиляцией. На Windows-стенде клиент ещё не запускался.

## Working

- `rds-shadow-core`: INI, события, JSONL, фильтры, `mstsc_args`
- `rds-shadow-audit`: фильтры и CSV
- CI и release workflows
- CI #1 на GitHub: Linux и Windows — success
- Документация ТЗ, решение 0002, гайд по релизу

## Broken / unknown

- `docs/PIPELINE.md` всё ещё `approved: false`
- Брокер: обнаружение коллекций не реализовано, нужен `[servers] list`
- Windows-клиент не проверен на живой ферме
- `gh` в среде не установлен; push идёт через токен и askpass

## Next

1. Зелёный CI на GitHub
2. Проверка `check-config` и просмотрщика на тестовом JSONL
3. Решение по брокеру и GUI
