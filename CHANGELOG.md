# Changelog

All notable Gradia changes are documented here.

## [0.2.2] — 2026-07-27

### Added

- Every calculation operation now presents a live “What this operation does”
  card with a plain-language explanation and concrete numerical example.
- Guidance covers sum, average, best single, best N, drop lowest, multiplication,
  maximum conversion, weighted combinations, and ordered subtraction.
- The same dynamic guidance is available when creating or editing a calculated
  assessment field.

### Validation

- Frontend guidance coverage now verifies all nine calculation operations.

## [0.2.1] — 2026-07-27

### Fixed

- The Windows release now includes a versioned standalone executable, closing the
  distribution gap that left standalone users on the older v0.1.0 behavior.
- Quick Entry exposes the active roster range and keeps the newly selected student
  visible when Next student crosses positions 12→13, 24→25, and 36→37.
- The active student row is brought into view after navigation.

### Added

- A visible version badge in the main sidebar so users can immediately confirm
  which executable is running.
- Explicit “What this type does” and “What this view means” labels above the
  dynamic assessment and gradebook-view explanations.
- Settings now presents complete-database movement as Export database and Import
  database using a validated `.gradia` transfer file.
- Database transfers record the creating Gradia version and enforce the `.gradia`
  extension.
- Complete edit paths for semesters, courses and accent colors, sections,
  students and enrollment order/status, gradebook views, assessment/calculation
  fields, grading policies, and attendance-session metadata.
- Permanent semester, course, and section deletion with a cascade-impact preview
  and exact typed confirmation.

### Security and validation

- Import verifies the Gradia format, embedded SHA-256 digest, SQLite integrity,
  and migrations before replacing the current local database.
- The UI accurately states that `.gradia` transfer files are integrity-protected
  but are not password encrypted.
- Structural edits capture old/new audit values. Deletion records its impact and
  safely activates another semester when the active semester is removed.
- Lowering an assessment maximum is rejected when existing recorded marks exceed
  the proposed maximum.
- Regression coverage uses the reported 40-student roster and tests all 12-record
  boundaries through the final positions.

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
[0.2.2]: https://github.com/the-sudipta/gradia/releases/tag/v0.2.2
[0.2.1]: https://github.com/the-sudipta/gradia/releases/tag/v0.2.1
