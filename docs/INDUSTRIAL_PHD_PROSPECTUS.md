# Gradia Industrial PhD & Research Collaboration Prospectus

## Executive proposition

Gradia turns a familiar but fragile academic workflow—marks, attendance,
calculations, analytics, and official spreadsheet submission—into a local-first,
auditable desktop system. It is a working research vehicle rather than only a
concept: the first release includes a versioned database, configurable grading and
calculation logic, teacher-centered entry workflows, descriptive analytics,
fidelity-preserving Excel export, snapshots, and backups.

The project creates an applied research opportunity at the intersection of
human-computer interaction, end-user programming, education technology, data
provenance, privacy engineering, dependable systems, and spreadsheet
interoperability.

## Problem

Teachers often use spreadsheets because they are flexible, visible, and locally
controlled. The same qualities can also produce:

- duplicated and inconsistent formulas;
- accidental replacement of missing values with zero;
- hard-coded institutional policies;
- slow name/ID search and repetitive attendance work;
- limited provenance for grade changes;
- fragile cross-sheet and cross-workbook copying;
- dashboards that are difficult to reproduce; and
- privacy risks from ungoverned files and backups.

Replacing spreadsheets completely is unrealistic because official institutional
templates remain part of the submission boundary. Gradia therefore studies a hybrid
question: how can structured, auditable software preserve teacher flexibility while
remaining exactly compatible with institutional spreadsheets?

## Research themes

### 1. Trustworthy end-user assessment programming

Study how teachers can express weights, conversions, best-of rules, missing-value
semantics, grade ranges, and dependencies without traditional programming, while the
system prevents cycles, invalid ranges, and silent data corruption.

### 2. Human-centered high-frequency entry

Evaluate dynamic student search, focused mark forms, keyboard navigation, and
exception-first attendance against spreadsheet and paper baselines for speed,
errors, cognitive load, and user confidence.

### 3. Provenance-preserving spreadsheet interoperability

Develop and verify methods that inspect Office OpenXML packages, match authoritative
identifiers, change only approved cells, preserve non-target package parts, and
produce human-auditable evidence.

### 4. Privacy-preserving small-cohort analytics

Investigate useful descriptive and combined insights without remote telemetry,
causal overclaiming, or unnecessary exposure of identifiable student data.

### 5. Recoverable local-first academic systems

Study migration, backup integrity, encryption, portable data ownership, and
cross-platform recovery for non-technical teachers operating without a database
server or mandatory cloud account.

## Candidate research questions

- Which visual or declarative rule representation produces the fewest teacher
  configuration errors?
- How should a system explain a computed grade so a teacher can verify it rapidly?
- Does exception-first attendance materially reduce time and error rates?
- What evidence best increases trust that an exported institutional workbook was
  not reformatted or otherwise altered?
- Which analytics reveal actionable patterns while remaining robust for small
  sections and respectful of uncertainty?
- How can local-first tools support institutional governance without becoming
  surveillance systems?

## Available engineering foundation

- Tauri 2 desktop shell;
- Rust domain, calculation, backup, and Excel engines;
- bundled SQLite with migrations and audit records;
- flexible assessment fields and grade policies;
- keyboard-oriented teacher workflows;
- descriptive analytics and heatmaps;
- immutable result snapshots;
- checksummed portable backups; and
- tested Windows packaging.

The authoritative scope, design decisions, and verification baseline are recorded in
[GRADIA_PROJECT_MEMORY.md](project_info/GRADIA_PROJECT_MEMORY.md) and
[GRADIA_RUNBOOK.md](project_info/GRADIA_RUNBOOK.md).

## Proposed industrial PhD method

1. Conduct contextual inquiry with teachers across multiple institutions.
2. Establish spreadsheet and existing-software baselines.
3. Co-design rule, entry, attendance, and explanation interfaces.
4. Develop formal invariants and property-based tests for calculations and exports.
5. Run controlled usability studies using synthetic or properly governed data.
6. Pilot iteratively with ethics approval and data-minimization controls.
7. Evaluate performance, error rate, trust calibration, usability, accessibility,
   and institutional compatibility.
8. Publish reproducible protocols, anonymized aggregate evidence, and technical
   findings without exposing student records.

## Partner value

An industrial or university partner gains a concrete platform for research,
prototyping, controlled deployment, and impact evaluation. Potential outputs include
peer-reviewed papers, validated interaction patterns, dependable spreadsheet
interoperability methods, privacy and governance guidance, reusable test artifacts,
and a deployable teacher product.

## Collaboration requested

- funded industrial PhD appointment or doctoral scholarship abroad;
- supervision spanning HCI/education technology and dependable software;
- responsible access to teacher participants and institutional workflow expertise;
- research ethics, legal, and privacy support;
- usability, accessibility, and security facilities; and
- engineering or commercialization partnership under a separate written agreement.

## Contact

Project owner: **Sudipta Kumar**

- GitHub: <https://github.com/the-sudipta>
- Portfolio: <https://the-sudipta.github.io/portfolio>
- LinkedIn: <https://www.linkedin.com/in/sudiptakumar/>

Prospective partners should describe the institution, research fit, funding model,
country, expected start period, supervision environment, and proposed next meeting.
