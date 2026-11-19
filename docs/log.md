# Log

Append-only. Do not rewrite old entries.

## YYYY-MM-DD — bootstrap
- Changes: created from `/home/pmb/template-project/`
- Evidence: skeleton files present
- Docs: [[STATUS]]

## 2026-11-19 — Release v0.3.1
- Changes: закоммичена вся работа (этап 1 отображения shadow, стандарты интерфейса/плагинов, макеты 01–06 и 07 v1–v6, архив прав, решения 0003–0005); версия поднята до 0.3.1 (v0.3.0 уже был занят редактором прав); тег `v0.3.1` запушил → release workflow запущен
- Evidence: `git push` ok (main f5e40f1..e66a2b2, new tag v0.3.1); локально `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (22) — зелёные
- Docs: [[STATUS]]

## 2026-11-19 — Этап 1 генерации .rdp + запуска; правка про горячие клавиши
- Changes: принято без нового макета — горячие клавиши функций WGC-окна настраиваются в разделе «Горячие клавиши» настроек (ТЗ §8, решение 0005, gui-map). Этап 1 реализован: `rds-shadow-core::command::rdp_file` и `mstsc_shadow_args`; `platform::run_mstsc` пишет временный `.rdp` и запускает `mstsc <file> /shadow:<id>` (проброс `rdp_port`)
- Evidence: `cargo test -p rds-shadow-core` 22 passed (3 новых); `cargo fmt --check` ok; `cargo clippy --target x86_64-pc-windows-gnu -p rds-shadow --all-targets -- -D warnings` exit 0; `cargo check --workspace` exit 0
- Docs: [[decisions/0005-shadow-viewer-wgc]], [[architecture/tz-v1-draft]], [[STATUS]]

## 2026-11-19 — WGC-окно: v6 (основная концепция, управляемая мини-карта, помощь по функциям)
- Changes: макет `07-shadow-display-v6.html` — F принят за основную концепцию; мини-карта управляемая (кнопка «Мини-карта», `Ctrl+M`, появление при масштабировании); окно помощи описывает функции окна с кратким пояснением, убран очевидный нативный UI и мета; 7.3 сокращён до мини-карты и запоминания; ТЗ §8 и решение 0005 обновлены
- Evidence: v6 balance div 116/116, section 6/6, script 1/1; в помощи нет строк про перетаскивание/лупу; есть minimap-toggle и Ctrl+M; код не менялся
- Docs: [[decisions/0005-shadow-viewer-wgc]], [[architecture/tz-v1-draft]], [[architecture/gui-map]]

## 2026-11-19 — WGC-окно: компоновка v5
- Changes: макет `07-shadow-display-v5.html` — сеанс в заголовке (перетаскивание), режим и масштаб в подвале, подсказки плашкой 5 с → «?» → панель слева; убраны кнопки выбора монитора (#1/#2/#3); ТЗ §8 дополнено компоновкой окна
- Evidence: v5 balance div 116/116, section 6/6, script 1/1; в окне WGC нет левой панели и кнопок мониторов; код не менялся
- Docs: [[architecture/tz-v1-draft]], [[architecture/gui-map]]

## 2026-11-19 — Shadow multi-monitor: пояснения вне окна (v4) + решение 0005
- Changes: макет `07-shadow-display-v4.html` — пояснительная проза вынесена из панели окна в блоки «не часть окна»; панель E/F — только операционные элементы; зафиксированы выборы (Детали 1:1, монитор RDS Shadow, WGC, все доп.); ТЗ §8 дополнено разделом отображения; решение 0005 (WGC, план, риски); в `ui-standard.md` правило «пояснения вне окна»
- Evidence: v4 balance div 123/123, section 6/6, script 1/1, label 15/15, 2 блока `.annot`; код не менялся (WGC — Windows-only, нужен стенд)
- Docs: [[decisions/0005-shadow-viewer-wgc]], [[architecture/tz-v1-draft]], [[architecture/ui-standard]], [[architecture/gui-map]]

## 2026-11-19 — Shadow multi-monitor: контекст к вопросам (v3)
- Changes: макет `07-shadow-display-v3.html` — у каждого вопроса (7.1–7.4) добавлен короткий контекст «что спрашивается / что это не значит / ограничения»; уточнены подписи 7.2; в `ui-standard.md` закреплено правило «у каждого вопроса — контекст»; v1 и v2 сохранены
- Evidence: 4 блока `.qctx`; баланс v3 div 116/116, script 1/1, label 15/15; код не менялся
- Docs: [[architecture/ui-standard]], [[architecture/gui-map]]

## 2026-11-19 — Shadow multi-monitor: smart sizing и встраивание
- Changes: доисследованы smart sizing на лету (без переподключения) и встраивание сеанса (SetParent / WGC / ActiveX); макет `07-shadow-display-v2.html` с выборами (radio/checkbox) и авто-промтом; в `ui-standard.md` зафиксированы версионирование макетов и механизм выборов; v1 сохранён
- Evidence: Microsoft Learn (SetParent, WGC, DWM, MsTscAx), woshub/superuser (smart sizing на лету); код не менялся
- Docs: [[research/2026-11-19-shadow-multimonitor-visualization]], [[architecture/ui-standard]], [[architecture/gui-map]]

## 2026-11-19 — Shadow multi-monitor visualization
- Changes: research-бриф по визуализации shadow при 2–3 мониторах; макет `07-shadow-display.html` (обзор / детали 1:1 / три монитора / диалог «Отображение»); вопросы 7.1–7.3 размечены
- Evidence: разобраны возможности mstsc (smart sizing, scroll, /span, selectedmonitors) по внешним источникам; код не менялся
- Docs: [[research/2026-11-19-shadow-multimonitor-visualization]], [[architecture/gui-map]]

## 2026-11-19 — RDP-Tcp rights editor archived
- Changes: редактор прав RDP-Tcp вынесен в `archive/rdp-tcp-permissions/` (код `rights.rs`, макет `05-permissions.html`, скрипт `setup-rdp-tcp-permissions.ps1`, копия гайда `deployment-ad-gpo.full.md`); удалён из `lib.rs`/`app.rs` и из интерфейса (макеты 01–04, 06, `index.html`); живой гайд сокращён до требований и указателя на архив; ТЗ §2 дополнено
- Evidence: `cargo check --target x86_64-pc-windows-gnu -p rds-shadow` exit 0; `cargo clippy --target x86_64-pc-windows-gnu -p rds-shadow --all-targets -- -D warnings` exit 0; `cargo check --workspace` exit 0; в `crates/rds-shadow/src` нет ссылок `rights`/`PermEditor`/`Права RDP-Tcp`
- Docs: [[decisions/0004-archive-rdp-tcp-rights-editor]], [[guides/deployment-ad-gpo]], [[architecture/tz-v1-draft]]

## 2026-11-19 — UI standard and plugin model (from depesha)
- Changes: приняты стандарт интерфейса и модель плагинов по мотивам `fgbm/depesha`; research-бриф, `architecture/ui-standard.md`, `architecture/plugin-model.md`, решение 0003
- Evidence: клон depesha прочитан (`AGENTS.md`, `design/mockups/`, `scripts/`, `e2e/`, `plugins/README.md`, `src/plugin-api/`); код RDS Shadow не менялся
- Docs: [[architecture/ui-standard]], [[architecture/plugin-model]], [[decisions/0003-ui-and-plugin-standard]], [[research/2026-11-19-llm-checkable-ui-and-plugins]]

## 2026-11-19 — GUI mockups markup
- Changes: доразметка `gui_html/rds-shadow/` (01–06): семантика aria/role, стабильные data-testid, контракт вопросов data-q/data-variant; карта экранов `architecture/gui-map.md`
- Evidence: баланс тегов сведён (div/button без расхождений); 4 открытых вопроса размечены (1.1, 2.1, 3.1, 4.1)
- Docs: [[architecture/gui-map]], [[architecture/ui-standard]]

## 2026-11-19 — GUI decisions applied
- Changes: применены решения 1.1 (кнопки в панели), 2.1 (акцент на «Управление»), 3.1 (без «Отменить», поток logoff с опросом 2 с / 40 с), 4.1 (колонка «Теневой доступ»); ТЗ §8 дополнено поведением logoff
- Evidence: макеты 01/02/04/05 обновлены; `gui-map.md` — таблица решений и политика опроса; баланс div сохранён
- Docs: [[architecture/gui-map]], [[architecture/tz-v1-draft]]

## 2026-11-18 12:00 UTC — Remote Desktop Users requirement
- Changes: `mstsc /shadow` также требует права входа (Remote Desktop Users); скрипт теперь добавляет группы туда (`-SkipRemoteDesktopUsers` отключает); гайд дополнен разделом 3.3
- Evidence: подтверждено на стенде — после добавления в Remote Desktop Users shadow заработало
- Docs: [[guides/deployment-ad-gpo]]

## 2026-11-18 11:40 UTC — fix Deny Logon bug
- Changes: убран запрет Logon из `setup-rdp-tcp-permissions.ps1` (был Deny, ломал RDP-вход); исправлена обработка ошибок; убран `-Force`; проверка Allowed+Denied
- Docs: [[guides/deployment-ad-gpo]]

## 2026-11-18 11:25 UTC — script safety notes
- Changes: в гайд добавлен раздел «Безопасность применения и откат» (снапшот, откат)
- Docs: [[guides/deployment-ad-gpo]]

## 2026-11-18 11:20 UTC — permission setup script
- Changes: `scripts/setup-rdp-tcp-permissions.ps1` — идемпотентная выдача прав трём группам на RDP-Tcp; ссылка в гайде
- Evidence: скрипт сверен с ModifyPermissions ids (0/4/7/2) и снятием Logon-пресета
- Docs: [[guides/deployment-ad-gpo]]

## 2026-11-18 11:10 UTC — rights editor
- Changes: GUI-редактор прав RDP-Tcp (`rights.rs` + окно), исправлены WMI-вызовы и битовые маски в гайде
- Evidence: ModifyPermissions ids сверены с MS Learn; clippy ok под windows-gnu
- Docs: [[sessions/2026-11-18-rights-editor]], [[guides/deployment-ad-gpo]]

## 2026-11-18 10:55 UTC — deployment guide
- Changes: `docs/guides/deployment-ad-gpo.md` — AD groups, RDP-Tcp rights masks, GPO consent, verification
- Evidence: WINSTATION bit masks; no code change
- Docs: [[guides/deployment-ad-gpo]]

## 2026-11-18 10:50 UTC — GUI
- Changes: egui client, GUI/CLI split, bundled DejaVuSans, release package includes cli exe
- Evidence: clippy ok under windows-gnu; subsystems GUI/CUI verified
- Docs: [[sessions/2026-11-18-gui]]

## 2026-11-18 10:40 UTC — Actions on Node 24
- Changes: checkout v4→v7, upload-artifact v4→v7, download-artifact v4→v8, action-gh-release v2→v3
- Evidence: CI #6 success; Node.js 20 deprecation annotation gone (was present in CI #5)
- Docs: [[guides/release]]

## 2026-11-18 10:25 UTC — v0.1.0 released
- Changes: tag `v0.1.0`, 0.x marked pre-release
- Evidence: Release #1 success; assets `rds-shadow-windows-x64.zip` (707489 B), `rds-shadow-audit-linux-x64.tar.gz` (391970 B)
- Docs: [[guides/release]], [[sessions/2026-11-18-bootstrap-repo]]

## 2026-11-18 10:15 UTC — CI green
- Changes: release workflow triggers only on tags; CI badge in README
- Evidence: Actions run #1: Linux success, Windows success
- Docs: [[sessions/2026-11-18-bootstrap-repo]]

## 2026-11-18 10:10 UTC — repo bootstrap
- Changes: Cargo workspace, Windows client, audit viewer, CI/release workflows, README, release guide
- Evidence: 19 tests, clippy clean, Windows gnu target check, commit `9b86f91` pushed to `main`
- Docs: [[sessions/2026-11-18-bootstrap-repo]], [[decisions/0002-repo-ci-release]]

## 2026-11-18 09:49 UTC — rust tooling installed
- Changes: `.opencode/agents/rust-pro.md`, `.opencode/skills/rust-async-patterns/`, `.opencode/skills/memory-safety-patterns/` (from wshobson/agents, MIT)
- Evidence: files written; `model: opus` removed; references copied; user approved
- Docs: [[decisions/0001-v1-scope]]

## 2026-11-18 09:44 UTC — state save
- Changes: gates in `AGENTS.md`; skill `rds-shadow-domain`; overlay filled; decision `docs/decisions/0001-v1-scope.md`
- Evidence: skill file written locally; no network skill download; `opencode v2.3.2`
- Docs: [[sessions/2026-11-18-state-save]]

## 2026-11-18 — TZ audit cut
- Changes: v1 journal is files only; syslog/collector and screenshot stream moved out; broker/INI/farm facts written into `docs/architecture/tz-v1-draft.md`
- Evidence: user messages this session; no code; no farm query
- Docs: [[STATUS]]

## 2026-11-18 — TZ draft
- Changes: `docs/architecture/tz-v1-draft.md` updated with locked choices (bypass accepted for v1, audit fail-closed, full message text, ru+en UI)
- Evidence: user answers in session; no code, no stand check
- Docs: [[STATUS]]

## 2026-11-18 — research baseline
- Changes: разбор публичного `rds_shadow_connect`; зафиксированы решения по клиенту, допуску, согласию и аудиту
- Evidence: raw script прочитан; brief `docs/research/2026-11-18-rds-shadow-baseline.md`
- Docs: [[STATUS]]
