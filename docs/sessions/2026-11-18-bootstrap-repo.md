# 2026-11-18 — bootstrap repo, CI and release

## Goal

- Начать разработку: каркас Rust, git, CI, релиз по тегу.

## Class

fix

## Changes

- `Cargo.toml`, `rust-toolchain.toml` — workspace, stable, fmt/clippy.
- `crates/rds-shadow-core/` — INI-парсер, события, JSONL, фильтры, `mstsc_args` (19 тестов).
- `crates/rds-shadow/` — Windows-клиент: WTS API, гейты групп, сообщение, logoff, аудит.
- `crates/rds-shadow-audit/` — просмотрщик JSONL, фильтры, CSV.
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`.
- `README.md`, `docs/guides/release.md`, `docs/decisions/0002-repo-ci-release.md`.
- `.gitignore` — target, локальный INI, JSONL.

## Evidence

- `cargo fmt --all --check` — ok.
- `cargo clippy --workspace --all-targets -- -D warnings` — ok.
- `cargo test --workspace` — 19 passed в `rds-shadow-core`.
- `cargo check --workspace --target x86_64-pc-windows-gnu` — ok (после правок импортов windows-sys).
- Первый коммит `9b86f91`, push в `main`.

## Decisions

- Ядро отдельно от Windows-клиента, чтобы тесты шли на Linux.
- Одна версия workspace, тег обязан совпадать.
- Брокер пока требует список серверов.

## Checks

- Секрет-скан по репозиторию: совпадение только `CancellationToken` в учебном примере.

## Docs tier

L2 (архитектура, релизный процесс, gates)

## Next

1. Дождаться зелёного CI на GitHub.
2. Проверить `rds-shadow check-config` и `rds-shadow-audit` на реальном JSONL.
3. Реализовать обнаружение коллекций брокера или явно оставить список серверов.
