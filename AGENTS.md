# RDS Shadow — правила проекта

Глобальная конституция: `~/.config/opencode/AGENTS.md` (baseline).
Playbook: skill `project-orchestration`.
Домен: skill `rds-shadow-domain`. Черновик ТЗ: `docs/architecture/tz-v1-draft.md`.
Этот файл **уточняет** domain gates и DoD проекта.

## Назначение

**RDS Shadow** (`rds-shadow`) — Windows-клиент техподдержки для теневого просмотра и управления сеансами RDS, плюс просмотрщик файлового журнала. Допуск делает Windows, не программа.

## Не цель

- Обход согласия сотрудника.
- Теневое подключение с Linux и свой RDP-стек.
- Syslog и сервер сбора в v1.
- Скриншоты сеанса, хранение паролей, зашитые имена узлов и групп.
- Проверка logoff на чужом продуктовом сеансе.

## Роли

Primary: **`pm-orchestrator`** (global). Overlay `.opencode/agents/project-orchestrator.md` не является default.

Subagents (global, уже установлены, в проект не копируются):

| Role | Agent ID |
|------|----------|
| Explore | `code-explorer` |
| Implement | `implementer` |
| Review | `reviewer` |
| Docs | `docs-writer` (+ skill `project-docs`) |
| Research | `researcher` |

Перед правкой клиента или журнала загружать `rds-shadow-domain`.

## Domain gates

- Секреты / ПДн: паролей нет. В журнал попадают учётки, SID, номер заявки и полный текст сообщения. Кадр экрана не пишется.
- Prod / destructive: logoff и управление разрушительны. Первая проверка на ферме — список и просмотр с согласия добровольца.
- Сеть / чужие хосты: только брокер и узлы из INI текущего домена. Чужие фермы не сканировать.
- Commit/push: только по явному заданию.
- Runtime: клиент shadow — Windows 10/11; просмотрщик — Windows и Linux. MCP и docker для v1 не нужны.

## Документирование

- Корневой vault: `docs/`
- Тиры L0–L3 — global constitution; default L1. ТЗ и gates — L2.
- Session docs skill: `project-docs` + `docs-packet` + `technical-writing`
- Handoff: `HANDOFF.md` + skill `session-handoff`

## Definition of Done

Задача принята, когда:

1. Acceptance из brief выполнены.
2. Есть evidence (команды/exit codes/артефакты) — не «кажется работает».
3. Docs tier закрыт (L0+ по impact).
4. Нет секретов в diff/docs.
5. `STATUS`/`HANDOFF` не врут относительно текущего состояния (после значимой сессии).

## Pipeline

См. `docs/PIPELINE.md`. Пока `approved: false` — режим предложен `hybrid`, не включать молча.
