# Security Policy

## Supported versions

| Version | Security support |
|---|---|
| 0.1.x | Supported |
| Earlier or unreleased snapshots | Not supported |

## Reporting a vulnerability

Do not open a public issue for a vulnerability that could expose marks, identities,
backups, local files, workbook content, or command execution.

Use GitHub’s private vulnerability reporting page:

<https://github.com/the-sudipta/gradia/security/advisories/new>

If that route is unavailable, contact the maintainer through a private method listed
on <https://github.com/the-sudipta>. Include:

- affected Gradia version and operating system;
- clear reproduction steps using fictional data;
- impact and required preconditions;
- relevant logs with identities and local paths removed; and
- any safe mitigation already tested.

Never send a real `gradia.db`, `.gradia` backup, institutional workbook, access token,
or student screenshot.

## Response goals

The maintainer aims to acknowledge a complete report within seven days, assess
severity and reproduction, coordinate a fix and release where feasible, and credit
the reporter if requested. These are good-faith targets, not guaranteed service
levels.

## Security posture

Gradia minimizes exposure through local SQLite storage, no normal runtime telemetry,
versioned migrations, strict command validation, transactional writes, non-overwrite
Excel export, result snapshots, and checksummed backups. Checksums detect accidental
corruption; they do not encrypt a backup. Store backups as sensitive academic data.
