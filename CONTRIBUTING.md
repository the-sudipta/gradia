# Contributing to Gradia

Thank you for helping make academic assessment safer and easier for teachers.

## Before contributing

1. Read the [Code of Conduct](CODE_OF_CONDUCT.md), [Responsible Use Policy](RESPONSIBLE_USE.md),
   and [license](LICENSE).
2. Search existing issues before opening a duplicate.
3. For substantial product or schema changes, open a feature proposal before
   implementation.
4. Never upload real student data, institutional workbooks, databases, Gradia
   backups, credentials, or private screenshots.

## Development setup

Install Node.js 20+, Rust stable, and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm ci
npm run tauri dev
```

Run the required checks:

```bash
npm test
npm run build

cd src-tauri
cargo test --locked
cargo fmt --all -- --check
```

## Change expectations

- Preserve local-first behavior and avoid runtime network dependencies.
- Preserve Student IDs as text.
- Keep missing, zero, and special mark states distinct.
- Add migration and restart-persistence coverage for schema changes.
- Add hand-reconciled fixtures for calculation changes.
- For Excel changes, prove that only approved cells changed and the output reopens.
- Use fictional records in tests, documentation, and screenshots.
- Update the runbook, memory file, changelog, and user documentation when behavior
  changes.

## Commits

Use a clear conventional type and focused title. An appropriate emoji may precede
the type when it adds useful visual meaning:

```text
🎓 feat: add policy-aware section comparison
```

Use the commit body to explain motivation, behavior, privacy implications,
validation, and any compatibility decision.

## Pull requests

Keep a pull request reviewable. Complete the repository pull-request checklist,
describe manual verification, and disclose any limitation honestly. Maintainer
approval is required before merge.

By contributing, you agree to the contribution terms in Section 5 of [LICENSE](LICENSE).
