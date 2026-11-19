# Status

**Updated:** 2026-11-19

## One-paragraph state

Каркас продукта в GitHub: `Van-Hoffen/rds-shadow`, ветка `main`, первый коммит `9b86f91`. Три крейта: ядро, Windows-клиент, просмотрщик. CI на Linux и Windows, релиз по тегу `v*`. Локально: 22 теста в ядре, clippy чистый, Windows-цель собирается кросс-компиляцией. На Windows-стенде клиент ещё не запускался.

## Working

- `rds-shadow-core`: INI, события, JSONL, фильтры, `mstsc_args`, `rdp_file`, `mstsc_shadow_args`
- Этап 1 отображения shadow (WGC): генерация `.rdp` и запуск `mstsc <file> /shadow` в ядре (решение 0005); проверка на стенде — не сделана
- Релиз `v0.3.1`: тег запушен, release workflow запущен (сборка Windows + Linux)
- `rds-shadow-audit`: фильтры и CSV
- CI и release workflows
- CI #1, #3, #6 на GitHub: Linux и Windows — success
- Release #1 по тегу `v0.1.0`: success, pre-release с двумя артефактами
- Actions обновлены на Node 24, предупреждение Node 20 снято (CI #6)
- GUI-клиент на egui 0.37; раздельные exe `rds-shadow` (GUI) и `rds-shadow-cli`
- Редактор прав RDP-Tcp вынесен в `archive/rdp-tcp-permissions/` (решение 0004); в продукте и UI его нет
- Документация ТЗ, решение 0002, гайд по релизу

## Broken / unknown

- `docs/PIPELINE.md` всё ещё `approved: false`
- Брокер: обнаружение коллекций не реализовано, нужен `[servers] list`
- Windows-клиент не проверен на живой ферме
- `gh` в среде не установлен; push идёт через токен и askpass

## Next

1. Проверить GUI на Windows: INI, список, просмотр с согласия добровольца
2. Проверить `mstsc <file.rdp> /shadow` на Windows-стенде (этап 1, решение 0005)
3. Брокер (обнаружение коллекций)
