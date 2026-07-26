# Gradia 0.2.1

Gradia 0.2.1 is a corrective release for focused-entry navigation and Windows
standalone distribution. It also turns the existing verified backup mechanism
into a clear database export/import workflow for moving an entire workspace to
another device.

## What is fixed

- **The left student list follows Next student.** With a 40-student roster,
  moving from position 12 to 13 changes the visible list from 1–12 to 13–24.
  The same behavior continues at positions 24→25 and 36→37 through the actual
  end of the roster.
- **The finder states exactly what is visible.** It displays ranges such as
  `Showing 13–24 of 40`, and the selected row is brought into view.
- **Guidance is unmistakable.** The Add assessment dialog labels its live help
  as “What this type does” and “What this view means”, with behavior and an
  example for every selectable assessment type and gradebook view.
- **You can confirm the build.** The sidebar displays `v0.2.1`.

## Database transfer

Settings now contains **Export database** and **Import database**:

1. Export creates one `.gradia` file containing the complete local workspace.
2. Move that file to another Windows, macOS, or Linux device.
3. Install Gradia there and use Import database.
4. Gradia validates the format, embedded SHA-256 checksum, SQLite integrity, and
   migrations before replacing the destination device’s current database.

The custom extension prevents accidental selection of unrelated files, but it is
not encryption. Store `.gradia` files securely because they contain academic
records. Password-encrypted transfer is a separate roadmap item.

## Edit what you created

- **Setup:** edit semester, course code/name/export name/accent color/policy,
  section label, student identity/email/roster position/status, grading policy,
  and gradebook views.
- **Gradebook:** select any assessment header to edit its label, stable key, term,
  type, maximum, contribution, view, calculation recipe, final-result role, or
  archived state without discarding existing student entries.
- **Attendance:** edit a session’s date, title, or note without changing its
  saved attendance statuses.
- **Existing values:** marks, written fields, attendance statuses, and pipeline
  stages remain directly editable as before.

Every structural update stores old/new audit values. Gradia rejects an assessment
maximum below an already saved mark.

## Deliberate permanent deletion

Setup can permanently delete the selected semester, course, or section. Before
enabling deletion, Gradia displays the cascade impact across child structures,
rosters, marks, attendance, and snapshots and requires the exact displayed phrase.
If the active semester is deleted, another remaining semester is activated
automatically.

## Downloads

- **Windows x64:** standalone `.exe`, NSIS setup `.exe`, or Windows Installer
  `.msi`.
- **macOS Apple Silicon:** `aarch64` `.dmg`.
- **macOS Intel:** `x64` `.dmg`.
- **Linux x64:** portable `.AppImage` or Debian/Ubuntu `.deb`.
- **Integrity:** `RELEASE_CHECKSUMS.txt` lists SHA-256 values for every
  downloadable package.

The packages are built on native GitHub-hosted runners. They are not commercially
code-signed or Apple-notarized. Windows/macOS may therefore show an
unfamiliar-developer warning.

## Verification

- Frontend regression tests, including every 12-record boundary of a 40-student
  roster.
- Rust database-transfer, calculation, database, snapshot, backup, and
  Excel-fidelity tests.
- Rust formatting and Clippy checks.
- Production frontend build.
- Native Windows, macOS Apple Silicon, macOS Intel, and Linux release builds.
- Independent download and launch check of the published Windows standalone
  executable before the draft release is made public.

## License

Gradia is source-available, not open source. Non-commercial use is governed by
`LICENSE`; commercial use requires a separate written agreement described in
`COMMERCIAL-LICENSING.md`.
