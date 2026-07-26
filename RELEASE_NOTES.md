# Gradia 0.1.0

Gradia’s first public release turns the complete core teacher workflow into a
private, local-first Windows desktop application.

## Highlights

- Build semesters, courses, sections, rosters, policies, and assessment structures.
- Record marks in a flexible gradebook or dynamic-search form.
- Take attendance by marking everyone Present and changing only exceptions.
- Configure institute-independent grade ranges and calculation/conversion rules.
- Explore distributions, grade frequencies, heatmaps, boundary cases, and
  descriptive observations.
- Track evaluation, recording, and portal-upload progress independently.
- Fill official institutional Excel templates by exact Student ID while preserving
  non-target workbook content and never overwriting the source.
- Finalize auditable result snapshots and save checksummed portable backups.
- Launch the Windows application without an accompanying terminal.

## Windows downloads

- `Gradia_0.1.0_x64-setup.exe` — recommended interactive installer.
- `Gradia_0.1.0_x64_en-US.msi` — Windows Installer package for managed deployment.
- `gradia.exe` — standalone application binary.
- `RELEASE_CHECKSUMS.txt` — SHA-256 verification values.

Windows may display an unfamiliar-publisher warning because version 0.1.0 is not
code-signed. Verify the SHA-256 checksum before installation.

## Privacy

Normal operation is account-free and local. The release contains fictional demo
records only. Store `gradia.db`, exported workbooks, and `.gradia` backups as
sensitive academic data.

## Verification

- Frontend tests: 6 passed.
- Rust tests: 10 passed.
- Rust formatting check: passed.
- Production frontend build: passed.
- Native Windows release, MSI, and NSIS packaging: passed.
- Repository publication audit: 0 errors and 0 warnings.

## License

Gradia is source-available, not open source. Non-commercial use is governed by
`LICENSE`; commercial use requires a separate written agreement described in
`COMMERCIAL-LICENSING.md`.
