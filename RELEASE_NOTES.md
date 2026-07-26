# Gradia 0.2.0

Gradia 0.2.0 makes assessment setup self-explanatory, keeps focused entry
synchronized with the full roster, and introduces native packages for Windows,
macOS, and Linux.

## Highlights

- **Focused entry follows the student.** Moving from student 12 to 13 now advances
  the left finder to the next roster window; the selected student never disappears
  from the visible list.
- **Every assessment type explains itself.** Score, Calculated, Attendance, Bonus,
  Penalty, Text, and Note include behavior guidance and a concrete example.
- **Every starter gradebook view explains itself.** The dialog clarifies No
  specific view, Midterm, Final, Semester Result, and Attendance, including the
  difference between a Term and a View.
- **Setup shows the active destination.** A visible Semester → Course → Section
  path makes it clear where courses, sections, and students will be added.
- **Text means text.** Text and Note fields now accept and persist written values
  in the main gradebook and focused-entry form.
- **Penalties can be deducted.** The visual calculation builder now exposes
  subtraction using the first selected source minus the later sources.

## Downloads

- **Windows x64:** NSIS setup `.exe` or Windows Installer `.msi`.
- **macOS Apple Silicon:** `aarch64` `.dmg`.
- **macOS Intel:** `x64` `.dmg`.
- **Linux x64:** portable `.AppImage` or Debian/Ubuntu `.deb`.
- **Integrity:** `RELEASE_CHECKSUMS.txt` lists SHA-256 values for every downloadable
  package.

These packages are built on native GitHub-hosted operating-system runners. They
are not commercially code-signed or Apple-notarized. macOS builds use an ad-hoc
signature, but macOS may still require approval under **System Settings → Privacy
& Security**. Windows may show an unfamiliar-publisher warning.

## Privacy and compatibility

All editions use the same local SQLite model and `.gradia` backup format. Normal
operation remains account-free and makes no required runtime network requests.
Back up `gradia.db` before moving between versions or computers.

## Verification

- Frontend behavioral and rendering tests.
- Rust unit, calculation, database, snapshot, backup, and Excel-fidelity tests.
- Rust formatting and Clippy checks.
- Production frontend build.
- Native Windows, macOS Apple Silicon, macOS Intel, and Linux release builds.

## License

Gradia is source-available, not open source. Non-commercial use is governed by
`LICENSE`; commercial use requires a separate written agreement described in
`COMMERCIAL-LICENSING.md`.
