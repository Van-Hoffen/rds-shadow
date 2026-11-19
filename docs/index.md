# RDS Shadow — docs map

**Updated:** 2026-11-19

| Page | Role |
|------|------|
| [[STATUS]] | current truth |
| [[log]] | append-only event log |
| [[PIPELINE]] | pipeline, not yet approved |
| [[architecture/tz-v1-draft]] | v1 spec draft |
| [[architecture/ui-standard]] | standard for writing the interface (mockups + invariants) |
| [[architecture/gui-map]] | screen map + open GUI decisions |
| [[architecture/plugin-model]] | plugin model (core vs plugins, contract, permissions) |
| [[decisions/0001-v1-scope]] | why v1 has no syslog and no copied agents |
| [[decisions/0002-repo-ci-release]] | workspace, CI and release process |
| [[decisions/0003-ui-and-plugin-standard]] | adopt depesha UI standard and plugin model |
| [[decisions/0004-archive-rdp-tcp-rights-editor]] | RDP-Tcp rights editor archived, out of product |
| [[decisions/0005-shadow-viewer-wgc]] | shadow multi-monitor: own WGC viewer, zoom/pan |
| [[decisions/0006-single-windows-crate]] | collapsed to one Windows crate, dropped Linux |
| [[guides/release]] | how to cut a release |
| [[guides/deployment-ad-gpo]] | AD groups, GPO consent, rights requirements (mechanics in archive) |
| sessions/ | session notes |
| decisions/ | architecture/product decisions |
| research/ | research briefs |
| guides/ | how-to |
| architecture/ | how the system works |
| templates/ | local templates |

Project root also has `HANDOFF.md` (resume point) and `AGENTS.md` (agent rules).

Archive (outside the product, preserved work): `archive/rdp-tcp-permissions/` — RDP-Tcp rights editor, mockup, setup script, full deployment guide copy. See decision 0004.
