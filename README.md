# RDS Shadow

[![CI](https://github.com/Van-Hoffen/rds-shadow/actions/workflows/ci.yml/badge.svg)](https://github.com/Van-Hoffen/rds-shadow/actions/workflows/ci.yml)

Клиент техподдержки для теневого подключения к сеансам RDS и просмотрщик файлового журнала.

Статус: ранний каркас. ТЗ: `docs/architecture/tz-v1-draft.md`. Состояние: `docs/STATUS.md`.

## Состав

| Крейт | Платформа | Назначение |
|---|---|---|
| `rds-shadow-core` | любая | INI-конфиг, события аудита, JSONL, фильтры, аргументы `mstsc` |
| `rds-shadow` | Windows 10/11 | список сеансов, shadow, сообщение, logoff, запись журнала |
| `rds-shadow-audit` | Windows, Linux | просмотр JSONL, фильтры, CSV |

Допуск делает Windows. Программа не хранит пароли, не обходит согласие сотрудника и не включает кнопки, если группа не задана в конфиге.

## Сборка

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Кроссплатформенная часть проверяется на Linux. Windows-клиент:

```bash
rustup target add x86_64-pc-windows-gnu
cargo check -p rds-shadow --target x86_64-pc-windows-gnu
```

## Конфигурация

Скопировать `docs/architecture/rds-shadow.example.ini` в `rds-shadow.ini` рядом с программой и заполнить имена узлов, брокера и групп. Файл в git не коммитится.

```bash
rds-shadow --config rds-shadow.ini check-config
rds-shadow --config rds-shadow.ini list
rds-shadow --config rds-shadow.ini shadow --server ts03 --session 14 --ticket SD-12345
rds-shadow --config rds-shadow.ini message --server ts03 --session 14 --ticket SD-12345 --text "..."
rds-shadow --config rds-shadow.ini logoff --server ts03 --session 14 --ticket SD-12345 --confirm-user "CORP\\m.petrov"
```

Просмотрщик:

```bash
rds-shadow-audit --dir ./logs --ticket SD-12345
rds-shadow-audit --dir ./logs --since 2026-11-18T00:00:00Z --csv
```

## Релиз

Тег `vX.Y.Z` запускает `.github/workflows/release.yml`: сборка Windows-клиента и просмотрщика, Linux-просмотрщика, проверка версии, GitHub Release с артефактами. Версия в `[workspace.package]` должна совпадать с тегом.

## Ограничения v1

- Теневая картинка — только на Windows-ПК специалиста.
- Брокер: режим объявлен, обнаружение коллекций пока не реализовано; используйте `[servers] list`.
- Журнал — локальные JSONL и, если задан, каталог на шаре. Syslog позже.
- Скриншоты сеанса не пишутся.
