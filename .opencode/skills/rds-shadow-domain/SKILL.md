---
name: RDS Shadow domain
description: Domain gates for the RDS shadow support client. Use before any change to shadow, logoff, messages, config, audit files, or the viewer (RDS, mstsc, consent, INI, JSONL).
---

# RDS Shadow — domain gates

Source of truth: `docs/architecture/tz-v1-draft.md`. If this skill and the TZ disagree, stop and ask. Do not invent a stricter or looser rule.

## Hard rules

- Do not add `/noConsentPrompt` or any consent bypass.
- Do not store or prompt for passwords. Use the current Windows logon token only.
- Do not hardcode production broker names, session hosts, or AD group names. They live in the INI. Empty group → hide the button, do not enable it.
- Shadow client is Windows-only. Do not pretend `mstsc /shadow` works from Linux.
- v1 audit is JSONL files only. Do not block an action because a share or syslog is down. Do not add a syslog client in v1.
- Do not log screen images, window titles, or clipboard. Message text is logged in full.
- Logoff requires a ticket and the typed target user name.
- Do not use a production user session as a logoff or control test.

## Load with

- Implement: `test-driven-development`, then this skill.
- Before a success claim: `verification-before-completion`.
- Docs after a change: `docs-packet`, `project-docs`, `technical-writing`.
