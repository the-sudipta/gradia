# Changelog

All notable Gradia changes are documented here.

## [0.1.0] — 2026-07-26

### Added

- Local-first Tauri desktop application with SQLite persistence.
- Semester, course, section, student, roster, policy, and assessment setup.
- Flexible gradebook views and keyboard-friendly focused mark entry.
- Exception-first attendance with five explicit states.
- Declarative calculation and conversion rules with missing-value safety.
- Institute-independent grade bands and policy-driven grade calculation.
- Section dashboard, descriptive statistics, distributions, grade frequency,
  heatmap, boundary review, and observation-only insights.
- Independent Evaluated, Marks Recorded, and Portal Uploaded pipeline stages.
- Exact Student-ID institutional Excel matching, preview, surgical Mark-cell writes,
  fidelity verification, and academic filename generation.
- Immutable result snapshots, audit logging, and checksummed backup/restore.
- Transparent Gradia branding, Windows icon set, MSI, and NSIS packages.
- Reopenable welcome and semester setup flow.
- Public repository documentation, CI, citation, issue templates, research
  prospectus, and source-available licensing.

### Security and privacy

- Normal runtime requires no account, telemetry, CDN, remote font, or update check.
- Public demo fixtures contain fictional identities only.
- Databases, backups, institutional spreadsheets, and exports are excluded from Git.

### Verified

- 6 frontend tests and 10 Rust tests.
- Production web bundle and native Windows release build.
- SQLite migration, integrity, snapshot, backup, calculation, and Excel-fidelity
  behavior.
- Windows GUI subsystem launch without a terminal window.

[0.1.0]: https://github.com/the-sudipta/gradia/releases/tag/v0.1.0
