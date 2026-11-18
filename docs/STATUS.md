# Status

**Updated:** 2026-11-18 11:10 UTC

## One-paragraph state

Каркас продукта в GitHub: `Van-Hoffen/rds-shadow`, ветка `main`, первый коммит `9b86f91`. Три крейта: ядро, Windows-клиент, просмотрщик. CI на Linux и Windows, релиз по тегу `v*`. Локально: 19 тестов, clippy чистый, Windows-цель собирается кросс-компиляцией. На Windows-стенде клиент ещё не запускался.

## Working

- `rds-shadow-core`: INI, события, JSONL, фильтры, `mstsc_args`
- `rds-shadow-audit`: фильтры и CSV
- CI и release workflows
- CI #1, #3, #6 на GitHub: Linux и Windows — success
- Release #1 по тегу `v0.1.0`: success, pre-release с двумя артефактами
- Actions обновлены на Node 24, предупреждение Node 20 снято (CI #6)
- GUI-клиент на egui 0.37; раздельные exe `rds-shadow` (GUI) и `rds-shadow-cli`
- Редактор прав RDP-Tcp в GUI (галочки Query/Remote Control/Message/Logoff)
- Документация ТЗ, решение 0002, гайд по релизу

## Broken / unknown

- `docs/PIPELINE.md` всё ещё `approved: false`
- Брокер: обнаружение коллекций не реализовано, нужен `[servers] list`
- Windows-клиент не проверен на живой ферме
- `gh` в среде не установлен; push идёт через токен и askpass

## Next

1. Проверить GUI на Windows: INI, список, просмотр с согласия добровольца
2. Брокер (обнаружение коллекций)
3. Утвердить ТЗ и pipeline
