# Repository File Guide

Select files based on the project and publication goal. Do not create empty ceremony.

## Core files

- `README.md`: identity, value proposition, screenshots, key capabilities, architecture, installation, quick start, privacy posture, documentation links, support, citation, funding, and license.
- `LICENSE`: exact legal grant and restrictions. Label custom restrictive terms as proprietary or source-available.
- `CITATION.cff`: machine-readable project citation with verified author, version, date, and repository URL.
- `CHANGELOG.md`: release history following a Keep a Changelog-style structure.
- `.gitignore`: precise exclusions for dependencies, builds, credentials, databases, backups, local exports, and private fixtures.
- `.gitattributes`: text normalization and binary declarations.

## Collaboration and governance

- `CONTRIBUTING.md`: setup, branches, tests, commit expectations, privacy requirements, and pull-request process.
- `CODE_OF_CONDUCT.md`: community behavior and enforcement contact. Keep product-use restrictions elsewhere.
- `RESPONSIBLE_USE.md`: illegal, abusive, discriminatory, privacy-invasive, or academically dishonest uses that are forbidden.
- `SECURITY.md`: supported versions and private vulnerability reporting route.
- `SUPPORT.md`: usage questions, bug reports, and commercial inquiry routes.
- `GOVERNANCE.md`: maintainer authority, decision model, and licensing ownership when useful.
- `.github/ISSUE_TEMPLATE/*.yml`: structured bug and feature requests with privacy warnings.
- `.github/PULL_REQUEST_TEMPLATE.md`: validation and data-safety checklist.

## Automation

- `.github/workflows/ci.yml`: real lint, test, and build commands on supported platforms.
- `.github/dependabot.yml`: dependency update policy when the ecosystem is supported.
- Avoid workflows that require nonexistent secrets or publish artifacts without explicit authorization.

## Funding and research positioning

- `.github/FUNDING.yml`: only verified sponsor handles or real custom links.
- `FUNDING.md`: transparent use of funds, sponsorship routes, and research ambitions.
- `docs/INDUSTRIAL_PHD_PROSPECTUS.md`: research problem, industrial relevance, research directions, evidence, collaboration sought, and contact route.
- Never promise outcomes, imply endorsements, or list unconfirmed sponsors.

## README composition

1. Centered logo, project name, tagline, and compact badge rows.
2. A short, concrete explanation of who the product is for and what it replaces.
3. One strong hero screenshot followed by a small gallery.
4. A full-width capability table when comparison is useful:

```html
<table width="100%">
  <tr>
    <td width="50%">Capability A</td>
    <td width="50%">Capability B</td>
  </tr>
</table>
```

5. Privacy and data ownership.
6. Installation and quick start.
7. Architecture and development commands.
8. Roadmap and contribution links.
9. Citation, funding, commercial licensing, and legal status.

Use descriptive image alt text. Keep screenshots free of personal data and workstation chrome.

## Badge rules

- Prefer stable badges from GitHub Actions, Releases, Shields.io, and official technology ecosystems.
- Link each badge to a useful target.
- Use repository-specific URLs only after the owner and repository name are verified.
- A `source--available` badge must not say `open source`.
- Do not add download counts, coverage percentages, build status, or platform support claims without real backing.

## Release checklist

- Version agrees across application metadata, citation, changelog, and tag.
- Tests and production build pass from the committed tree.
- Release artifacts come from that exact tree.
- SHA-256 checksums are included.
- Release notes identify supported platforms and known limitations.
- Database, backups, logs, and private spreadsheet fixtures are absent.
