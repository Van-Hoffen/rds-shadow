# RDS Shadow

[![CI](https://github.com/Van-Hoffen/rds-shadow/actions/workflows/ci.yml/badge.svg)](https://github.com/Van-Hoffen/rds-shadow/actions/workflows/ci.yml)

Клиент техподдержки для теневого подключения к сеансам RDS и просмотрщик файлового журнала.

Статус: ранний каркас. ТЗ: `docs/architecture/tz-v1-draft.md`. Состояние: `docs/STATUS.md`.

## Состав

| Крейт | Платформа | Назначение |
|---|---|---|
| `rds-shadow` | Windows 10/11 | GUI: список сеансов, shadow, сообщение, logoff, журнал |
| `rds-shadow-cli` | Windows 10/11 | те же действия из командной строки |
| `rds-shadow-audit` | Windows 10/11 | просмотр JSONL, фильтры, CSV |

Допуск делает Windows. Программа не хранит пароли, не обходит согласие сотрудника и не включает кнопки, если группа не задана в конфиге.

## Запуск

`rds-shadow.exe` без аргументов открывает графический интерфейс. Верхняя строка — путь к INI и «Загрузить»; таблица — сеансы; снизу — номер заявки и кнопки Просмотр / Управление / Сообщение / Завершить сеанс. Управление и завершение требуют отдельных групп и подтверждения.

Настройка прав `RDP-Tcp` из клиента в текущую версию не входит: программа прав не выдаёт (ТЗ §1). Редактор, макет и скрипт сохранены в `archive/rdp-tcp-permissions/` (решение 0004). Подготовка фермы — `docs/guides/deployment-ad-gpo.md`.

`rds-shadow-cli.exe` — консольный вариант для диагностики и скриптов.

## Сборка

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Сборка только под Windows. Локально на Linux-машине разработки можно кросс-проверить компиляцию (без запуска):

```bash
rustup target add x86_64-pc-windows-gnu
cargo check -p rds-shadow --target x86_64-pc-windows-gnu
```

## Конфигурация

Скопировать `docs/architecture/rds-shadow.example.ini` в `rds-shadow.ini` рядом с программой и заполнить имена узлов, брокера и групп. Файл в git не коммитится.

```bash
rds-shadow-cli --config rds-shadow.ini check-config
rds-shadow-cli --config rds-shadow.ini list
rds-shadow-cli --config rds-shadow.ini shadow --server ts03 --session 14 --ticket SD-12345
rds-shadow-cli --config rds-shadow.ini message --server ts03 --session 14 --ticket SD-12345 --text "..."
rds-shadow-cli --config rds-shadow.ini logoff --server ts03 --session 14 --ticket SD-12345 --confirm-user "CORP\\m.petrov"
```

Просмотрщик:

```bash
rds-shadow-audit --dir ./logs --ticket SD-12345
rds-shadow-audit --dir ./logs --since 2026-11-18T00:00:00Z --csv
```

## Релиз

Тег `vX.Y.Z` запускает `.github/workflows/release.yml`: сборка Windows-клиента (GUI + CLI + просмотрщик), проверка версии, GitHub Release. Версия в `[workspace.package]` должна совпадать с тегом.

## Ограничения v1

- Теневая картинка — только на Windows-ПК специалиста.
- Брокер: режим объявлен, обнаружение коллекций пока не реализовано; используйте `[servers] list`.
- Журнал — локальные JSONL и, если задан, каталог на шаре. Syslog позже.
- Скриншоты сеанса не пишутся.
