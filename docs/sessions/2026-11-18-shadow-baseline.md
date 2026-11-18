# 2026-11-18 — shadow baseline

## Goal

- Разобрать публичный скрипт RDS shadow и собрать решения для ТЗ безопасного Rust-клиента.

## Class

research

## Changes

- `docs/research/2026-11-18-rds-shadow-baseline.md` — разбор скрипта, решения пользователя, варианты. Evidence: raw `rds_shadow_connect` прочитан целиком (387 строк).

## Decisions

- Клиент на Windows-ПК специалиста; допуск — AD + GPO; view по умолчанию, control отдельно; согласие обязательно.
- v1: список, shadow, сообщение, logoff.
- Аудит в четыре приёмника; содержимое экрана не логировать.
- Обход согласия не проектируем.

## Checks

- Исходник сверен с raw GitHub, не с HTML-обёрткой страницы blob.

## Docs tier

L0 (research note, кода нет)

## Next

1. Закрыть оставшиеся вопросы ТЗ (ферма, группы, syslog, просмотрщик).
2. Утвердить pipeline `hybrid` и черновик ТЗ.
3. Не начинать код до ТЗ и Windows-стенда для проверки WTS/GPO.
