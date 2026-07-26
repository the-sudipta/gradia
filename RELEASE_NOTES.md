# Gradia 0.2.2

Gradia 0.2.2 makes calculated-assessment setup self-explanatory. Every
calculation operation now tells the teacher exactly what it does and shows a
concrete numerical example before the field is saved.

## Calculation guidance

The Add/Edit assessment dialog now displays a live
**“What this operation does”** card for:

- **Sum selected fields** — for totals such as OBE + viva + written exam.
- **Average selected fields** — for the arithmetic mean of several marks.
- **Best single field** — for choosing the highest selected mark.
- **Best N (sum)** — for keeping and adding the highest N marks.
- **Drop lowest (sum)** — for removing the lowest marks and adding the rest.
- **Multiply one field** — for rules such as doubling one score.
- **Convert mark from one maximum to another** — for percentage-preserving
  conversion such as 75/100 → 30/40.
- **Weighted combination** — for results such as 40% midterm + 60% final.
- **Subtract later fields from the first** — for applying one or more
  deductions to a starting mark.

The explanation and example change immediately with the selected operation. The
same guidance appears when creating a field and when editing an existing
calculated field. Every description also states how missing marks are handled.

## Included from v0.2.1

- Quick Entry follows every 12-student boundary through the actual roster.
- Assessment types and gradebook views have live explanations and examples.
- Complete `.gradia` database export/import with checksum and SQLite integrity
  validation.
- Edit workflows for semesters, courses and colors, sections, students,
  gradebook views, assessments/calculations, policies, and attendance sessions.
- Deliberate permanent semester/course/section deletion with cascade preview and
  exact typed confirmation.
- Versioned Windows standalone executable with no terminal window.

## Downloads

- **Windows x64:** standalone `.exe`, NSIS setup `.exe`, or MSI.
- **macOS Apple Silicon:** `aarch64` `.dmg`.
- **macOS Intel:** `x64` `.dmg`.
- **Linux x64:** portable `.AppImage` or Debian/Ubuntu `.deb`.
- **Integrity:** `RELEASE_CHECKSUMS.txt` contains SHA-256 values for every native
  package.

Packages are built on native GitHub-hosted runners. They are not commercially
code-signed or Apple-notarized, so Windows/macOS may show an
unfamiliar-developer warning.

## Verification

- Frontend tests cover all nine calculation-operation guides.
- Rust database, transfer, calculation, deletion, snapshot, backup, and
  Excel-fidelity tests.
- Rust formatting and Clippy with warnings denied.
- Production frontend build.
- Native Windows, macOS Apple Silicon, macOS Intel, and Linux release builds.
- Independent metadata, checksum, and launch verification of the downloaded
  Windows standalone before publication.

## License

Gradia is source-available, not open source. Non-commercial use is governed by
`LICENSE`; commercial use requires a separate written agreement described in
`COMMERCIAL-LICENSING.md`.
