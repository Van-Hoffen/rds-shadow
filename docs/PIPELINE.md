# Pipeline — PROJECT_NAME

```yaml
approved: false
approved_on: null
mode: hybrid   # hybrid | pm-strict | build-closeout
docs_default_tier: L1
primary_agent: pm-orchestrator
```

## Modes

| Mode | Behavior |
|------|----------|
| `hybrid` | PM classifies and orchestrates multi-step; small asks can be direct; explicit closeout |
| `pm-strict` | Almost all work through orchestrator + subagents |
| `build-closeout` | Implement freely; mandatory docs/handoff at session end |

## Default loop (`fix`)

```
classify → plan → implement → review → verify → docs tier → handoff
```

## Docs tiers

See global constitution. Default **L1**. Use **L2** for API/architecture/security
and project domain-critical surfaces (list below when known):

- …

## Subagents

Prefer global IDs unless project replacements exist:

`code-explorer`, `implementer`, `reviewer`, `docs-writer`, `researcher`

## Approval

User sets `approved: true` and date after reviewing this file.
Until then, orchestrator must propose mode and wait if starting a long engagement.
