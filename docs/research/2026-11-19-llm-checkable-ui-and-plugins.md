# Research Brief — Проверяемый агентами интерфейс и модель плагинов depesha

- Date: 2026-11-19
- Question: Как проект `fgbm/depesha` делает интерфейс проверяемым агентами LLM без участия человека, и какова его модель плагинов — что из этого переносится в RDS Shadow.
- Scope / out of scope: разбор подхода depesha к макетам, инвариантам, скриншотам, e2e и плагинному контракту. Не реализация, не правка кода RDS Shadow.
- Источник: клон `https://github.com/fgbm/depesha` (ветка по умолчанию), файлы `AGENTS.md`, `design/mockups/`, `scripts/`, `e2e/`, `plugins/README.md`, `src/plugin-api/`.

## Findings

- Интерфейс задаётся машиночитаемым артефактом: самодостаточные HTML-макеты по версиям `design/mockups/<версия>/NN-имя.html`, открываются через `file://`, без сети. Рисуются до реализации. — mockups + README — high
- Контракт макета — `data-*`-атрибуты: `data-q`, `data-q-title`, `data-variants`, `data-recommended`, `data-variant`, `data-q-closed`. Плюс семантика `aria-*` и литеральные `role=`. — mockups/README — high
- `scripts/mockup-kit.mjs` вставляет в макет интерактивную панель «Ответы» (вопросы/варианты/заметки, localStorage, сводка). Маркеры `<!-- mockup-kit:start/end -->` позволяют повторно вставлять набор без правки файла вручную. — mockup-kit.mjs — high
- Инварианты фронтенда — снимок набора `aria-*`, `role=`, видов `data-*` (только тех, что читает e2e) и членов `app.*`. CI падает только на ухудшении; рост — строкой с причиной в `exceptions.txt`. — frontend-invariants.sh — high
- Детерминированные скриншоты `scripts/shots.sh` → `e2e/shots.mjs`: сеяные демо-данные (фикс. адреса, фикс. даты), съёмка на Xvfb `:99`, результат в `docs/screenshots/`. Повторяемо, без человека. — shots.sh + e2e/README — high
- E2E через WebDriver гоняет настоящее приложение на виртуальном дисплее и проверяет то, что видит человек (меню, тосты, клавиши, позиции). Каждый шаг помечен номером критерия из `docs/acceptance.md`. — e2e/README — high
- Плагинная философия: «ядро — только почта, всё остальное — плагин». Контракт — `src/plugin-api/` (~363 строки), защищён снимком `docs/plugin-api.snapshot.d.ts` (CI падает на дрейфе). — plugins/README — high
- Всё идёт через объект контекста `ctx`: точки расширения `ctx.ui.*` (command, keybinding, banner, rowTag, view, listFilter, composeControl, sendCheck, settingsSection, overlay…), `ctx.backend`, `ctx.settings` (на плагин), `ctx.t`/`ctx.plural` (свои строки). — plugins/README — high
- Два вида плагинов: built-in (TS/Svelte, доверенные, в бандле) и community (чистый JS в Web Worker в песочнице `ext://`, с правами). — plugins/README — high
- Права + согласие: manifest декларирует `messages.read/modify`, `storage`, `network:<host>`; пользователь видит их словами до установки; `granted.json`; `network:<host>` валидируется против reserved-зон и форм IP. — plugins/README — high
- `defaultOff` для вкусовщины, `defaultOn` для защиты от реальной ошибки — различие по вреду, а не по предпочтению. Точки расширения добавляются минимально, ядро без плагинов поведение не меняет. — plugins/README + AGENTS.md — high

## Fit for this project

- applicable — HTML-макеты + `data-*`/`aria-*` контракт + инварианты-гард. В RDS Shadow уже есть `gui_html/` с шестью макетами; осталось довести до контракта и добавить baseline-гард.
- partial — детерминированные скриншоты: применимы к HTML-макетам; для нативного egui-GUI нужен отдельный харнесс (`egui_kittest` + `accesskit`), это отдельная задача.
- no — WebDriver/Svelte e2e и TS-плагинный код: RDS Shadow — Rust + egui, не web. Переносится модель (контракт, точки расширения, права/согласие, core-vs-plugin), не реализация.

## Options

1. **Макеты + инварианты (принято).** HTML-макеты с `data-*`/`aria-*` контрактом и baseline-гард инвариантов. Скриншоты — для HTML-макетов. `egui_kittest` — отдельной задачей.
2. Только макеты-контракт, без CI-гарда.
3. Всё выше + сразу спроектировать `egui_kittest`/`accesskit` харнесс.

## Recommend next

- Принять стандарт интерфейса и модель плагинов как документы архитектуры (см. `docs/architecture/ui-standard.md`, `docs/architecture/plugin-model.md`, решение 0003).
- Реализацию контракта плагинов и `egui_kittest`-харнесса отложить до стабилизации модели и утверждения pipeline.
