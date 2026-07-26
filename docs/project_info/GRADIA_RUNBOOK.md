# Gradia Implementation Runbook

Read `GRADIA_PROJECT_MEMORY.md` first. It is the authoritative product specification. This runbook is the gated implementation and verification checklist.

## Phase 0 — Naming and scaffold

- Use `Gradia` in UI, documentation, package metadata, installer, backup names, and database name.
- Tauri identifier must be stable and Gradia-specific.
- Create the documented Rust/JS structure.
- Ensure `npm run tauri dev` launches.

**DoD:** No obsolete working-name product identifiers remain in code, package metadata, or UI; dev app launches without errors.

## Phase 1 — Database and domain core

- Implement versioned migrations for all entities in Project Memory Section 10.
- Enable foreign keys and WAL/busy timeout.
- Keep Student IDs as text.
- Enforce one active semester and unique enrollment per section/student.
- Implement audit logging helpers and transaction boundaries.

**DoD:** migration idempotency, FK, uniqueness, cascade/restrict, and restart-persistence tests pass.

## Phase 2 — Calculation and policy engine

- Grade-band validation and lookup.
- Declarative calculation rules.
- Dependency ordering and cycle rejection.
- Missing/zero/special-state semantics.
- Calculation traces.
- Immutable result snapshots.

**DoD:** operator, boundary, invalid-rule, cycle, and snapshot tests pass with hand-reconciled fixtures.

## Phase 3 — Backend commands

- Settings, semesters, courses, sections.
- Students, enrollment, roster import/search.
- Views, fields, entries.
- Policies, bands, rules.
- Attendance.
- Pipeline.
- Analytics.
- Backup/restore.
- Excel preflight/export.

**DoD:** every registered command returns typed JSON and error messages are actionable.

## Phase 4 — Application shell

- Dark Gradia design system.
- Sidebar and global context selectors.
- Dashboard, Gradebook, Student Entry, Attendance, Insights, Excel Bridge, Settings.
- Keyboard navigation and accessible status presentation.

**DoD:** all routes render, resize safely, and work without runtime network calls.

## Phase 5 — Onboarding and configuration

- Empty-state institute/semester setup.
- Course, section, roster flows.
- Policy and assessment builders.
- View/field management.

**DoD:** a new teacher can create a complete course workspace without database/manual file editing.

## Phase 6 — Marks and attendance

- Gradebook grid.
- `Ctrl+K` finder and form mode.
- Validation, special states, save/next.
- Exception-first attendance.
- Audit and undo.

**DoD:** 40 students × 20 fields remains usable; all writes persist and audit.

## Phase 7 — Analytics

- Summary metrics and grade frequency.
- Distributions, trends, comparisons, heatmaps, scatter/correlation.
- Observation-only insight text.

**DoD:** every displayed number reconciles to known database fixtures and manual calculations.

## Phase 8 — Excel bridge

- Inspect workbook package/sheets/shared strings.
- Strict header and Mark-column validation.
- ID matching preview.
- Filename preview.
- Surgical OpenXML value replacement.
- Fidelity verification.

**DoD:** supplied template exports correctly; only approved Mark cell values change; output reopens.

## Phase 9 — Backup, privacy, finalization

- Versioned backup.
- Restore preview/transaction.
- Result snapshot/revision.
- Sensitive-data warnings and local-only posture.

**DoD:** round-trip preserves all entities, rules, marks, attendance, snapshots, and audit records.

## Phase 10 — Branding and packaging

- Original Gradia icon.
- App metadata and installer.
- Windows standard and offline-ready configuration documentation.

**DoD:** `npm run tauri build` succeeds and installer launches in a clean Windows profile/VM.

## Phase 11 — Final verification and sync-back

- Run all Rust and frontend tests.
- Run workbook fidelity tests.
- Visual QA every view.
- Record actual counts, paths, and limitations in `GRADIA_PROJECT_MEMORY.md`.

**DoD:** all confirmed checks listed; failures are never hidden; final handoff links runnable source, tests, and installer.

## Verified 0.1.0 baseline — 2026-07-25

- Phases 0–5: implemented for the core Windows workflow.
- Phase 6: grid, focused entry, validation, and exception-first attendance implemented; advanced undo/bulk shortcuts remain backlog.
- Phase 7: section summaries, distributions, grade frequency, heatmap, and descriptive insights implemented; cross-section/semester advanced analytics remain backlog.
- Phases 8–10: implemented and tested; Windows MSI and NSIS packages built.
- Phase 11: 6 frontend tests and 10 Rust tests passed; production bundle, native launch, and real database initialization verified.

Run these checks before every handoff:

```powershell
npm test
npm run build
Push-Location src-tauri
cargo test
Pop-Location
npm run tauri build
```

Release artifacts:

```text
src-tauri/target/release/gradia.exe
src-tauri/target/release/bundle/msi/Gradia_0.1.0_x64_en-US.msi
src-tauri/target/release/bundle/nsis/Gradia_0.1.0_x64-setup.exe
```
