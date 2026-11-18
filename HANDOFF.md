# Handoff: RDS Shadow

- **Date:** 2026-11-18 10:25 UTC
- **Status:** repo bootstrapped; CI green; `v0.1.0` pre-release published; Windows client not tested on a stand
- **Repo:** https://github.com/Van-Hoffen/rds-shadow (`main`, `9b86f91`)
- **Docs:** docs/STATUS.md
- **TZ:** docs/architecture/tz-v1-draft.md
- **Decisions:** docs/decisions/0001-v1-scope.md, 0002-repo-ci-release.md
- **Pipeline:** docs/PIPELINE.md (`approved: false`; proposed `hybrid`)

## Now

- First CI run is green on Linux and Windows.
- Local toolchain installed: Rust 1.100, fmt, clippy; Windows gnu target and mingw for cross-check.
- Do not test logoff on a production session.

## Done this session

- Cargo workspace: `rds-shadow-core`, `rds-shadow` (Windows), `rds-shadow-audit`.
- Core tests: INI, events, JSONL, filters, `mstsc` args without consent bypass (19 passed).
- Windows client: WTS list, group gates, message, logoff with typed user, JSONL audit.
- CI (Linux+Windows) and Release by `v*` tag with version/tag check.
- First commit pushed. `.gitignore` excludes target, local INI, JSONL.
- Skills installed earlier: `rds-shadow-domain`, `rust-pro`, `rust-async-patterns`, `memory-safety-patterns`.

## Not done / blockers

- `v0.1.0` is a pre-release: no broker discovery, no GUI, no stand test.
- Broker collection discovery is not implemented; `[servers] list` is required.
- No Windows RDS stand; consent policy and WTS behavior are not verified.
- Pipeline not approved.

## Next steps (max 3)

1. Confirm CI is green on GitHub.
2. Run `check-config` and `rds-shadow-audit` against sample JSONL.
3. Decide: broker discovery next, or GUI, or stand test first.

## Danger / do not

- Do not add a consent bypass.
- Do not hardcode hosts or group names.
- Do not logoff a production session as a test.
- Do not commit secrets.
