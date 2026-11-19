# Архив — настройка прав RDP-Tcp

**Статус:** вынесено из продукта, 2026-11-19. **Наработки сохранены, но в текущие версии не входят.**

## Что здесь

| Файл | Откуда | Что это |
|------|--------|---------|
| `rights.rs` | `crates/rds-shadow/src/rights.rs` | WMI-модуль: `list_accounts`, `add_account`, `set_permission` (через `powershell.exe`) |
| `05-permissions.html` | `gui_html/rds-shadow/05-permissions.html` | макет экрана «Права RDP-Tcp» (редактор галочек) |
| `setup-rdp-tcp-permissions.ps1` | `scripts/setup-rdp-tcp-permissions.ps1` | идемпотентная выдача прав RDP-Tcp трём группам |
| `deployment-ad-gpo.full.md` | копия `docs/guides/deployment-ad-gpo.md` | полный гайд развёртывания на момент переноса (с разделом про редактор и скрипт) |

Историческая заметка о разработке остаётся в хронологии: `docs/sessions/2026-11-18-rights-editor.md`.

## Почему вынесено

- ТЗ §1: «Программа не выдаёт права. Разрешение или отказ делает Windows.» Редактор прав этому противоречил.
- В текущей версии продукта настройка прав не нужна; интерфейс и код ее не содержат.
- Наработки нельзя терять: модуль, макет и скрипт сохранены здесь целиком.

## Что было удалено из продукта

- `crates/rds-shadow/src/lib.rs` — `pub mod rights;` убран.
- `crates/rds-shadow/src/app.rs` — убраны `use crate::rights…`, `PermEditor`, `AppEvent::Perms`, поле `perm_editor`, `open_perm_editor`, `load_perms`, `draw_perm_editor`, кнопка «Права RDP-Tcp» в верхней панели и вызов отрисовки.
- Макеты `gui_html/rds-shadow/01..04,06` — убран пункт навигации «Права RDP-Tcp».
- `gui_html/rds-shadow/index.html` — убрана ссылка на `05-permissions.html`.
- `docs/guides/deployment-ad-gpo.md` — убраны упоминания внутриклиентского редактора; раздел про выдачу прав RDP-Tcp заменён указателем на этот архив.

## Как вернуть

1. Вернуть `rights.rs` в `crates/rds-shadow/src/` и `pub mod rights;` в `lib.rs`.
2. Восстановить в `app.rs`: импорт, `PermEditor`, событие `AppEvent::Perms`, поле и инициализацию `perm_editor`, методы `open_perm_editor` / `load_perms` / `draw_perm_editor`, кнопку и вызов — проще всего из истории git: `git log -- crates/rds-shadow/src/app.rs` до коммита переноса.
3. Вернуть макет `05-permissions.html` в `gui_html/rds-shadow/` и пункт навигации.
4. Вернуть скрипт в `scripts/` (или запускать прямо отсюда).
5. Учесть: по ТЗ §1 продукт прав не выдаёт — возврат требует пересмотра ТЗ.
