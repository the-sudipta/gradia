# Changelog

All notable Gradia changes are documented here.

## [0.2.0] — 2026-07-26

### Added

- Context-sensitive explanations and examples for every assessment field type.
- Matching explanations for No specific view, Midterm, Final, Semester Result,
  Attendance, and future custom gradebook views.
- A visible Setup destination path explaining how the selected semester, course,
  and section govern new records.
- Subtraction in the visual calculation builder for penalty-aware results.
- Native release automation for Windows x64, Linux x64, macOS Apple Silicon, and
  macOS Intel packages.

### Changed

- Focused entry now pages the left student list around the selected student instead
  of leaving it fixed on the first 12 roster entries.
- Text and Note assessment fields now render and save written values in both the
  gradebook and focused-entry form.
- The assessment type selector now uses plain-language option labels.
- macOS packages use an ad-hoc signature when a commercial Apple signing identity
  is not configured.

### Security and validation

- The Rust command boundary now rejects unknown assessment field types.
- Calculated fields must contain a calculation recipe, while raw fields cannot
  smuggle one.

### Verified

- Focus-window boundaries for 34-student rosters, including the 12→13 transition.
- Complete guidance metadata for all seven assessment field types and the starter
  gradebook views.
- Frontend tests, Rust tests, formatting, production build, and native
  multi-platform release jobs.

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
[0.2.0]: https://github.com/the-sudipta/gradia/releases/tag/v0.2.0
