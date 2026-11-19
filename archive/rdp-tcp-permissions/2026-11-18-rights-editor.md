# 2026-11-18 — права RDP-Tcp в GUI

## Goal

- Дать настройку прав RDP-Tcp (query/shadow/message/logoff) прямо из exe, аналог tsconfig.msc/RDSConfig, без консоли.
- Исправить модель прав в инструкции (была ошибка: `AddAccount` не имеет `PermissionMask`).

## Class

fix

## Changes

- `crates/rds-shadow/src/rights.rs` — WMI-вызовы через PowerShell: `list_accounts`, `add_account`, `set_permission` (`Win32_TSAccount.ModifyPermissions`).
- `crates/rds-shadow/src/app.rs` — окно «Права RDP-Tcp»: выбор сервера, список учётных записей с галочками, добавление группы.
- `docs/guides/deployment-ad-gpo.md` — исправлены WMI-вызовы и битовые маски; добавлен способ через GUI.

## Evidence

- Сигнатуры сверены с Microsoft Learn: `AddAccount(AccountName, PermissionPreSet)`; `ModifyPermissions(PermissionMask, Allow)`; `PermissionMask` ids: Query=0, Logoff=2, Shadow=4, Message=7, Reset=6.
- `PermissionsAllowed` bits: Query=1, Logoff=4, Shadow=16, Message=128.
- `cargo clippy -p rds-shadow --target x86_64-pc-windows-gnu --all-targets -- -D warnings` — ok.

## Decisions

- Редактор прав использует WMI через спавн `powershell.exe` (надёжнее, чем COM из Rust), результат — JSON.
- Маски групп исправлены: View=145, Control=17, Logoff=5 (было неверно 65).

## Docs tier

L2 (безопасность/права, новый компонент)

## Next

1. Проверить редактор прав на Windows-стенде от админа.
2. Брокер (обнаружение коллекций).
3. Расширить i18n-словарь.
