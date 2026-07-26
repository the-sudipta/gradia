# Gradia — Authoritative Project Memory

**Tagline:** Smarter academic assessment.

**Product name:** Gradia

**Status:** Version 0.1.0 working Windows release built and verified; expansion backlog recorded below.

**Owner:** Master (AIUB CSE Instructor)

**Last consolidated:** 2026-07-25
**Authority:** This file supersedes every earlier pre-Gradia concept document whenever they conflict.

---

## 1. Product Mission

Gradia is a private, local-first desktop application that replaces the repetitive academic parts of raw Microsoft Excel for an individual teacher. It combines:

- semester, course, section, and roster management;
- flexible spreadsheet-like gradebooks and custom assessment fields;
- fast form-based and grid-based marks entry;
- exception-first attendance;
- institute-independent grading policies;
- reusable calculation and mark-conversion rules;
- administrative completion tracking;
- section/course/semester analytics and visual insights;
- exact institution-supplied Excel template filling and export;
- local backups, audit history, and result snapshots.

Gradia is not intended to reproduce every general-purpose Excel capability. It is intended to be substantially better than Excel for a teacher's assessment, attendance, calculation, analysis, and institutional reporting workflow.

---

## 2. Non-Negotiable Principles

1. **Local-first:** normal operation makes zero network calls and requires no account.
2. **Institute-independent:** no AIUB grade range, course structure, terminology, or formula is hard-coded.
3. **Single source of truth:** forms, grids, dashboards, calculations, and exports read the same records.
4. **Auditable calculations:** every derived result exposes its inputs, rule, intermediate values, and output.
5. **Missing is not zero:** empty, zero, absent, excused, withdrawn, incomplete, and not-applicable are distinct states.
6. **Safe editing:** destructive schema changes require confirmation and data-bearing fields are archived before permanent removal.
7. **Deterministic exports:** Excel matching uses Student ID; names are never the primary key.
8. **Template fidelity:** Excel export changes only approved Mark cells and never overwrites the source template.
9. **Versioned policy:** changing a grade scale or formula must not silently rewrite finalized historical results.
10. **Privacy by design:** student identities and marks are sensitive; backups and database access must be treated accordingly.
11. **Cross-platform data:** the application is built separately per OS, but the SQLite data model and Gradia backups remain portable.
12. **Vanilla desktop architecture:** Tauri v2 + Rust + SQLite + vanilla HTML/CSS/JS; no heavy frontend framework.

---

## 3. Primary User Journey

### First launch

1. Create an institute profile and choose a default grading policy.
2. Create a semester using structured fields:
   - season: Spring, Summer, Fall, or custom;
   - session: e.g. `2025-2026`.
3. Add courses and official export names.
4. Add sections.
5. Add/import students into each section.
6. Configure Midterm/Final/custom assessment structures.

### Daily work

1. Open Dashboard.
2. Select semester → course → section.
3. Enter attendance or marks using:
   - spreadsheet-like grid; or
   - `Ctrl+K` student search → focused entry form.
4. Review validation warnings and incomplete work.
5. Inspect section analytics and administrative pipeline status.
6. Finalize results when ready.
7. Fill an institution Excel template and export it with the correct filename.

---

## 4. Functional Scope

### 4.1 Semester, course, and section management

- Create and switch semesters.
- Enforce a single active semester.
- Store season and session separately while displaying `Season Session`.
- Create courses with code, name, official export name, and accent color.
- Create, rename, reorder, archive, and delete empty sections.
- Maintain different schemas per course and term.

### 4.2 Student roster

- Student ID stored as text and preserved exactly.
- Name, optional email/notes, and active/withdrawn state.
- A student may be enrolled in multiple sections/semesters.
- Import rosters from CSV/XLSX after preview and duplicate checks.
- Search by partial ID or partial name, case-insensitively.
- Duplicate IDs within a section are prohibited.
- Students missing from an uploaded institutional template are ignored during export.

### 4.3 Gradebook views

Teachers can create unlimited named views analogous to workbook sheets:

- Midterm Summary
- Midterm Lab Tasks
- Final Summary
- Final Lab Tasks
- Viva
- Attendance
- Grade Distribution
- OBE Attainment
- Custom View

Views reference the authoritative fields and entries rather than copying student data.

### 4.4 Assessment fields / columns

Supported field types:

- score;
- number;
- text;
- checkbox;
- date;
- attendance summary;
- bonus;
- penalty;
- calculated score;
- percentage;
- grade/GPA/result;
- note.

Every score field stores:

- label and stable key;
- term;
- raw maximum;
- contribution maximum;
- display order;
- optional calculation rule;
- archived state.

Teachers can add, insert, rename, reorder, hide, archive, and restore fields.

### 4.5 Marks entry

#### Grid mode

- Frozen Student ID/Name columns.
- Horizontal and vertical scrolling.
- Direct cell entry with maximum/minimum validation.
- Copy/paste and fill-down.
- Sort, filter, search, and show only missing/invalid entries.
- Heatmap and grade-band coloring.
- Optimistic save with error reconciliation.

#### Form mode

- `Ctrl+K` opens the student finder.
- Suggestions show name, full ID, course, and section.
- Ranking: exact ID → ID prefix → name prefix → word prefix → contains.
- Focused assessment form displays maximums and current values.
- `Save`, `Save & Next`, and `Save & Next Unmarked`.
- Change history and undo.

### 4.6 Attendance

Exception-first attendance is the default:

1. Create a dated session.
2. Mark all active students Present.
3. Change only exceptions to Absent, Late, Excused, or Left Early.

Additional behavior:

- keyboard shortcuts P/A/L/E;
- search by name/ID;
- bulk change;
- session notes;
- live counts;
- undo;
- consecutive-absence warnings;
- attendance percentage calculation;
- optional attendance-to-marks conversion rule.

### 4.7 Administrative pipeline

The original pipeline evolves to:

1. **Evaluated**
2. **Marks Recorded**
3. **Portal Uploaded**

Each stage is independent and timestamped per assessment/section. Pending notes are supported. Excel template exports are recorded separately as export events.

---

## 5. Institute-Independent Grading Policies

A policy contains ordered grade bands:

- minimum and maximum percentage;
- inclusive/exclusive boundaries;
- grade label;
- GPA/value;
- result label (Pass/Fail/etc.);
- color.

Required validation:

- no overlapping bands;
- no unintended gaps;
- valid numeric ranges;
- deterministic boundary behavior;
- explicit handling for Absent, Incomplete, Withdrawn, and Not Applicable.

Policies are reusable, cloneable, importable/exportable, and versioned. Courses can use the institute default or an override.

---

## 6. Calculation and Conversion Engine

Gradia uses declarative rules rather than arbitrary executable code.

### Required operations

- direct value;
- sum;
- average;
- minimum/maximum;
- best N of M;
- drop lowest N;
- multiply;
- scale from one maximum to another;
- percentage contribution;
- weighted sum;
- cap/floor;
- bonus/penalty;
- conditional replacement;
- makeup substitution;
- attendance conversion;
- grade-policy lookup.

### Example

```text
Lab Task Final =
  best 2 of [Lab Task 1, Lab Task 2, Lab Task 3]
  then scale from 40 to 30

Semester Result =
  Midterm Total normalized to 40%
  + Final Total normalized to 60%
```

Rules reference stable field IDs so renaming a field does not break dependencies. Cycles are rejected. A trace records input values, intermediate results, and the final output.

---

## 7. Analytics and Insights

### Section dashboard

- count, mean, median, min, max, standard deviation;
- missing, zero, absent, incomplete, pass, and fail counts;
- grade frequency and distribution;
- assessment completion;
- attendance rate;
- component performance;
- OBE/outcome attainment;
- grade-boundary proximity.

### Visualizations

- bars and grouped bars;
- histogram;
- line/trend;
- donut;
- box-plot-style distribution;
- heatmap;
- radar/outcome view;
- attendance-performance scatter plot;
- cumulative distribution.

### Combined analytics

- section-to-section comparison;
- Midterm-to-Final change;
- course and semester overview;
- component difficulty comparison;
- grade distributions across courses;
- attendance/performance association;
- anomaly and missing-data warnings.

Insight text must distinguish observation from causation.

---

## 8. Excel Template Bridge

### Import/preflight

1. User selects semester, course, section, and term.
2. User uploads `.xlsx`.
3. Gradia lists worksheets and detects the likely data sheet.
4. The selected sheet must contain a `Student ID` header in the first column for the strict institutional workflow.
5. Gradia locates the `Mark` column or asks for explicit mapping.
6. Midterm/Final is inferred from the filename but always confirmed.
7. IDs are normalized for matching while original display text is preserved.

### Preview

Display:

- template row count;
- matched IDs;
- Gradia students absent from template (ignored);
- template IDs absent from Gradia (unchanged);
- duplicate IDs;
- students without finalized marks;
- cells that will change;
- proposed output filename.

Duplicate IDs, missing destination column, invalid headers, or formulas in destination Mark cells block export until explicitly resolved.

### Surgical OpenXML update

- copy the original workbook package;
- preserve every ZIP member;
- modify only the approved `<v>` numeric values in the target worksheet cells;
- preserve styles, borders, dimensions, merged cells, formulas, charts, drawings, print settings, external links, and metadata;
- never overwrite the source;
- verify all non-target package parts are byte-identical;
- verify the target XML changed only at approved cell values;
- reopen and validate the exported workbook.

### Filename

Store semester `season` and `session` separately.

```text
section matches /[0-9]+$/ → [SECTION]
otherwise                 → SECTION
```

Examples:

```text
INTRODUCTION TO PROGRAMMING LAB [B7] Midterm for 2025-2026 Fall.xlsx
DATA STRUCTURE LAB G Midterm for 2025-2026 Spring.xlsx
DATA STRUCTURE LAB AA Finalterm for 2025-2026 Summer.xlsx
```

Filename generation uses the course's official export name, selected section, confirmed term label, session, and season. It does not depend on fragile double-space replacement.

---

## 9. Finalization, Audit, and Safety

- Every important write creates an audit record with entity, old value, new value, timestamp, and optional reason.
- Finalizing results creates an immutable snapshot of policy, rules, marks, totals, and grades.
- Historical snapshots do not change when current policy/rules change.
- Corrections create a new revision rather than erasing history.
- Import and restore are transactional.
- Backups are versioned and checksummed.
- Sensitive backups should use an encrypted Gradia backup container by default; explicit JSON export is an advanced option.

---

## 10. Data Architecture

SQLite file: `gradia.db`, located in Tauri's `app_data_dir()`.

Core entities:

1. `settings`
2. `semesters`
3. `courses`
4. `sections`
5. `students`
6. `enrollments`
7. `gradebook_views`
8. `assessment_fields`
9. `grade_entries`
10. `grading_policies`
11. `grade_bands`
12. `calculation_rules`
13. `attendance_sessions`
14. `attendance_records`
15. `pipeline_status`
16. `export_templates`
17. `export_jobs`
18. `result_snapshots`
19. `snapshot_entries`
20. `audit_log`

Foreign keys are enabled on every connection. Deletions use restrictive or cascading behavior intentionally per entity. Migrations use `PRAGMA user_version`.

---

## 11. Technical Architecture

| Layer | Choice |
|---|---|
| Desktop shell | Tauri v2 |
| Backend | Rust stable |
| Database | SQLite via `rusqlite` with bundled SQLite |
| Serialization | `serde`, `serde_json` |
| Time | UTC RFC 3339 via `chrono` |
| Excel bridge | Rust ZIP + XML surgical editing |
| Frontend | Vanilla HTML/CSS/JS ES modules |
| Charts | Local SVG/Canvas components; no CDN |
| Fonts | Locally bundled system-safe font stack/assets |
| Testing | Rust unit/integration + Node tests |
| Packaging | Windows MSI/NSIS first; macOS/Linux builds supported |

Runtime network access, telemetry, and update checks are disabled.

---

## 12. UX and Visual Direction

- Near-black workspace with warm neutral cards.
- Distinct course accent colors.
- Clear typographic hierarchy.
- Dense gradebook optimized for real data entry.
- Keyboard-first interactions.
- Accessible colors and non-color-only status indicators.
- Responsive minimum window with scroll rather than squeezed grids.
- Original Gradia mark: abstract grade bands/check or ascending assessment bars; no copyrighted imagery.

---

## 13. Completion Evidence

Gradia is complete only when all applicable checks are green:

### Automated

- migration idempotency and foreign keys;
- CRUD and active-semester invariants;
- roster duplicate prevention and search ranking;
- grade-entry state/maximum validation;
- every calculation operator and dependency-cycle rejection;
- grading-band gap/overlap/boundary tests;
- attendance default-present and exceptions;
- analytics reconciled to known fixtures;
- pipeline timestamps;
- audit records and immutable snapshots;
- backup/restore round trip;
- Excel preflight, matching, filename generation, and surgical update;
- frontend rendering, optimistic entry, keyboard search, form validation, and charts.

### Workbook fidelity

- supplied institutional template fills correct Mark cells;
- source workbook remains unchanged;
- all non-target OpenXML parts remain byte-identical;
- target XML differs only at approved cells;
- exported workbook reopens and retains styles/borders/layout.

### Manual

- complete empty-state onboarding;
- create semester/course/section/roster;
- configure a grading policy and assessment schema;
- use both form and grid entry;
- record attendance;
- verify calculations and analytics manually;
- finalize results;
- export/import backup;
- generate the institutional Excel file;
- close/reopen and confirm persistence;
- test 40+ students, 20+ fields, and multiple sections;
- install and launch packaged Windows build.

### Documentation

- real test counts and artifact paths recorded;
- unresolved issues explicitly listed;
- dated changelog updated;
- no phase is reported complete without evidence.

---

## 14. Decision Log

- **2026-07-25:** Initial local-first completion tracker concept created under temporary working names.
- **2026-07-25:** Product expanded to store student rosters, attendance, marks, calculations, analytics, and institutional Excel exports.
- **2026-07-25:** Institute-independent policies and reusable declarative calculation rules required.
- **2026-07-25:** Form entry and spreadsheet-like grid are both required.
- **2026-07-25:** Exception-first attendance selected as the default attendance workflow.
- **2026-07-25:** Exact Student-ID-based surgical Excel template filling selected.
- **2026-07-25:** Product renamed to **Gradia** with tagline **“Smarter academic assessment.”**
- **2026-07-26:** Windows releases switched to the GUI subsystem so launching Gradia never opens a companion terminal.
- **2026-07-26:** The user-selected purple/mint Gradia mark was extracted to true transparency and integrated into the application shell, onboarding, favicon, ICO, ICNS, PNG, Windows Store, Android, and iOS assets.
- **2026-07-26:** Packaged frontend paths changed from root-absolute to bundle-relative URLs; the logo is imported through Vite so standalone releases resolve the hashed asset correctly.
- **2026-07-26:** Returning users can reopen onboarding through **Settings → Welcome & semester setup** without deleting or altering existing academic data.
- **2026-07-26:** Focused entry student lists became selected-student-aware roster windows, so advancing beyond the twelfth visible record also advances the left panel.
- **2026-07-26:** Assessment field types and gradebook views require in-product, context-sensitive explanations and concrete examples rather than undocumented labels.
- **2026-07-26:** Setup must show the active Semester → Course → Section destination because the left-panel academic context governs where new courses, sections, rosters, and records are written.
- **2026-07-26:** Version 0.2.0 release automation targets native Windows x64, Linux x64, macOS Apple Silicon, and macOS Intel packages; unsigned/unnotarized status must remain explicit.

---

## 15. Version 0.1.0 Implementation Record

This section records observed implementation evidence, not aspiration.

### Delivered teacher workflows

- first-run semester onboarding;
- semester, course, section, and individual student creation;
- `.xlsx` roster preview/import with duplicate blocking and idempotent enrollment;
- spreadsheet-like gradebook with frozen identity columns, term tabs, search, direct validation, missing-vs-zero semantics, and immediate persistence;
- `Ctrl+K`/sidebar quick entry with ranked partial ID/name search and Save & Next Unmarked;
- editable institute-independent grade policies, versioning, colors, grade points, pass/fail labels, and overlap validation;
- visual calculation builder for sum, average, maximum, best-N, drop-lowest, multiplication, scaling, and weighted combinations;
- backend calculation support for constants, minimum, cap/floor, add, and subtract as well;
- exception-first attendance with Present/Absent/Late/Excused/Left Early and live counts;
- independent Evaluated → Marks Recorded → Portal Uploaded pipeline stages;
- section descriptive statistics, distribution chart, grade frequency, result heatmap, and observation-only insight text;
- strict Student-ID Excel bridge with sheet/term confirmation, match preview, filename generation, source non-overwrite, surgical Mark-cell writes, and package fidelity verification;
- immutable result snapshots and audit logging;
- checksummed `.gradia` backup/restore with SQLite integrity verification;
- fully local normal runtime with no telemetry, remote fonts, CDN assets, or update checks.

### Implemented data and portability

- verified Windows database location:
  `%APPDATA%\app.gradia.desktop\gradia.db`;
- database schema version: `1`;
- domain table count: `20`;
- bundled SQLite means users do not install a database server;
- the same database/backup model is portable, while application binaries must be built separately for Windows, macOS, and Linux.

### Verification evidence — 2026-07-25

| Check | Observed result |
|---|---|
| Frontend unit tests | 6 passed, 0 failed |
| Rust unit/integration tests | 10 passed, 0 failed |
| Supplied institutional workbook | 39 roster/template rows; Student ID column A; Mark column C; Midterm detected |
| Supplied DS workbook | valid XLSX package with 7 worksheets |
| XLSX fidelity fixture | non-target package members byte-identical after surgical write |
| Backup fixture | round-trip preserved database rows and checksum verification |
| Production web bundle | `npm run build` passed |
| Native Windows executable | release build passed; PE subsystem code 2 (`Windows GUI`); process remained alive |
| Console-window check | no new `conhost`, `cmd`, or Windows Terminal process appeared on launch |
| Live desktop database | `PRAGMA integrity_check = ok`, `user_version = 1`, 20 tables, one default policy |
| Windows MSI | built successfully |
| Windows NSIS setup | built successfully |
| Visual QA | dashboard, gradebook, quick search, policy editor, calculation builder, attendance, pipeline, insights, Excel bridge, and transparent logo inspected at 1280px; zero console errors |
| Transparent logo | 1254×1254 RGBA; corner alpha all zero; alpha range 0–255; platform icon set regenerated |

### Release artifacts

```text
src-tauri/target/release/gradia.exe
src-tauri/target/release/bundle/msi/Gradia_0.1.0_x64_en-US.msi
src-tauri/target/release/bundle/nsis/Gradia_0.1.0_x64-setup.exe
```

SHA-256:

```text
gradia.exe
3751572D0A85F63BAD402E469CC0B056C0EB0F081C51C9D61DC5DE5E6EB51ADF

Gradia_0.1.0_x64_en-US.msi
72963CCBAC4F9BD4A4E713844D5FA1F25A93BCF8F1373718C366B3124764F685

Gradia_0.1.0_x64-setup.exe
7C7C7CCC7F280F568A5FC3976FD5F4840A499D09F1D48E55D3E06D8354819050
```

### Honest expansion backlog

Version 0.1.0 is a usable local teacher workflow, but the broader vision in Sections 4–7 intentionally remains larger. Future audited releases should add:

- CSV roster import in addition to the delivered XLSX importer;
- UI management for renaming/reordering/archiving fields and custom gradebook views;
- multi-cell paste/fill-down and visible per-entry undo/history controls;
- attendance bulk actions, keyboard status shortcuts, and consecutive-absence alerts;
- visual access to every backend rule operator plus makeup/conditional/attendance conversion recipes;
- course/semester combined comparisons, trends, scatter/correlation, OBE/radar, and box-plot views;
- encrypted backup containers (0.1.0 backups are checksummed ZIP containers and must be stored securely);
- signed/notarized macOS and packaged Linux releases, built and tested on those operating systems;
- clean-profile/VM installer test and large 40×20 interactive performance exercise.

No backlog item may be described as delivered until its evidence is added here.

---

## 16. Version 0.2.0 Implementation Record

### Delivered usability corrections

- the empty-search focused-entry list displays the 12-student roster window that contains the selected student;
- advancing from enrollment position 12 to 13 moves the left list from positions 1–12 to 13–24;
- wrapping to the first enrollment returns the list to the first window;
- Score, Calculated, Attendance, Bonus, Penalty, Text, and Note display dynamic behavioral guidance and examples;
- No specific view, Midterm, Final, Semester Result, Attendance, and future custom views display matching guidance;
- Text and Note fields accept and persist written values in grid and form entry;
- visual calculations expose subtraction so non-negative Penalty inputs can reduce a result;
- Setup displays the current Semester → Course → Section destination and explains the scope of each creation action;
- the public README mirrors the academic-context, field-type, and gradebook-view guidance.

### Validation and release engineering

- Rust rejects unknown field types at the command boundary;
- Calculated fields require a recipe and non-calculated fields reject recipe JSON;
- macOS uses the Tauri-recommended ad-hoc signing identity until commercial signing and notarization are configured;
- the GitHub release matrix builds on native Windows, Ubuntu, and macOS runners, including separate Apple Silicon and Intel targets.

### Local verification evidence — 2026-07-26

| Check | Observed result |
|---|---|
| Frontend unit tests | 8 passed, 0 failed |
| Focus boundary fixture | 34 enrollments; windows 1–12, 13–24, and 25–34 reconciled |
| Guidance fixture | all seven field types plus starter/no-specific views contain behavior and examples |
| Rust unit/integration tests | 10 passed, 0 failed |
| Rust formatting | passed |
| Clippy with warnings denied | passed |
| Production web bundle | passed |
| Browser visual QA | dynamic field/view guidance and Setup destination inspected; zero console warnings/errors |
| Local Windows installer retry | blocked only because the currently running Gradia process locked the existing release executable; the user process was intentionally left untouched |
| GitHub CI run 30212949814 | completed successfully |
| Native release run 30212956251 | completed successfully; 4/4 jobs |
| macOS Intel | `.dmg` and `.app.tar.gz` uploaded |
| macOS Apple Silicon | `.dmg` and `.app.tar.gz` uploaded |
| Linux x64 | `.AppImage` and `.deb` uploaded |
| Windows x64 | NSIS `.exe` and MSI uploaded |
| Release checksum manifest | all 8 uploaded packages downloaded and SHA-256 hashed |

The published hash manifest is `docs/RELEASE_CHECKSUMS.txt`. Version 0.2.0
remains explicit that Windows and macOS builds are not commercially signed and
that macOS is ad-hoc signed rather than Apple-notarized.
