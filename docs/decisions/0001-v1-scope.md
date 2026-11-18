# 0001 — Объём v1 и набор агентов

- Статус: принято в сессии 2026-11-18; ТЗ ещё не утверждено пользователем
- Тип: explanation

## Решение

v1 — Windows-клиент теневого подключения и отдельный просмотрщик JSONL. Журнал только файлы. Syslog, сервер сбора и скриншоты отложены. Имена узлов и групп только в INI.

Primary остаётся глобальный `pm-orchestrator`. Глобальные субагенты в проект не копируются: отдельного каталога навыков OpenCode нет, файлы уже лежат в `~/.config/opencode/`.

В проект добавлен только недостающий навык `rds-shadow-domain`.

## Почему не скачивали пачку с GitHub

Документация OpenCode v2 ставит навык как `SKILL.md` в `.opencode/skills` или в глобальный каталог. HTTP-каталога у этого пользователя нет. Чужие промпты агентов из сети не ставим: они не проверены и легко расходятся с gates.

## Обновление: Rust-спецнавыки (2026-11-18)

После явного согласия пользователя установлены три артефакта из `wshobson/agents` (MIT):

- агент `rust-pro` — `.opencode/agents/rust-pro.md` (без `model: opus`, добавлены project constraints);
- навык `rust-async-patterns` — `.opencode/skills/rust-async-patterns/` (SKILL.md + references/details.md);
- навык `memory-safety-patterns` — `.opencode/skills/memory-safety-patterns/` (SKILL.md + references/details.md).

Проверка перед установкой: raw-файлы прочитаны, вредоносных инструкций нет, лицензия MIT сохранена в frontmatter. `Mindrally/skills` и `anthropics/skills` не использованы: дублируют материал или не содержат Rust-специфики.

## Отклонено (повторно)

- Копия `implementer` / `reviewer` / `docs-writer` в `.opencode/agents/`.
- Свой приёмник syslog в v1.
- Смена `default_agent` на `project-orchestrator`.
- Любой сторонний агент/скилл без предварительного чтения содержимого.
