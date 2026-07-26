---
name: github-repository-manager
description: Prepare, audit, publish, and maintain professional GitHub repositories. Use when creating or restructuring repository documentation, README branding and badges, screenshots, CITATION.cff, contribution and conduct policies, security and support guidance, funding material, licenses, issue and pull-request templates, CI workflows, releases, or when validating git hygiene and publishing with GitHub CLI.
---

# GitHub Repository Manager

Turn a local project into a professional, truthful, safe, and maintainable GitHub repository.

## Workflow

1. Inspect before changing anything.
   - Read repository instructions such as `AGENTS.md`, project runbooks, and memory files.
   - Inspect the technology stack, test commands, build outputs, existing documentation, branding assets, git status, remotes, and recent history.
   - Check `gh --version` and `gh auth status` before planning remote actions.

2. Establish publication facts.
   - Determine the project name, repository owner, version, visibility, support channel, and publication goal from local evidence or the authenticated GitHub account.
   - Never invent a legal name, email address, funding account, citation author, repository URL, test result, security guarantee, or project capability.
   - If legal identity is unavailable, use the verified GitHub handle and repository issue/discussion channel, and flag that counsel should review custom terms.

3. Protect private and regulated data.
   - Audit tracked and untracked files for secrets, tokens, local databases, backups, exports, logs, build artifacts, absolute workstation paths, and personal or student data.
   - Never publish real student records, marks, attendance, uploaded institutional spreadsheets, `.db` files, `.gradia` backups, credentials, or private keys.
   - Preserve excluded local files; add precise ignore rules instead of deleting user data.
   - Replace demo personal data with clearly synthetic fixtures before public publication.

4. Build the repository surface.
   - Use `references/repository-files.md` to select the smallest complete set of repository files.
   - Make the README visually coherent, evidence-based, accessible, and useful before promotional.
   - Use centered HTML only for the hero area and elements that materially benefit from centering.
   - For intentionally full-width layouts, use an HTML table with `width="100%"`; do not claim Markdown tables have controllable width.
   - Only add badges for real technologies, workflows, releases, licenses, or policies.
   - Store reusable branding and screenshots under `docs/assets/`.

5. Handle licensing honestly.
   - Distinguish OSI-approved open-source licenses from proprietary or source-available terms.
   - Never describe a non-commercial license as open source.
   - Copyright protects expression, not abstract ideas or independently developed feature concepts. Do not claim a repository license automatically controls independent implementations.
   - A custom commercial-royalty license is a draft for legal review, not a guarantee of enforceability.
   - Put detailed commercial negotiation steps in `COMMERCIAL-LICENSING.md`, separate from community conduct rules.

6. Validate.
   - Run the project’s relevant tests and builds.
   - Run `python scripts/repo_audit.py <repository-path>` from this skill.
   - Inspect `git diff --check`, `git status --short`, and the exact staged diff.
   - Verify image paths, links, workflow commands, citation metadata, version numbers, and release artifact checksums.

7. Publish deliberately.
   - Create or use the intended branch and remote without overwriting unrelated history.
   - Stage only reviewed files.
   - Write a concise conventional commit subject and a detailed body that explains product, engineering, documentation, validation, privacy, and licensing changes.
   - For a new repository, create it with the verified owner and intended visibility, set the default branch, push, and verify the remote commit.
   - Create releases only from validated artifacts, and include checksums and accurate release notes.

8. Report the outcome.
   - Provide the repository URL, branch, commit hash, release URL when applicable, validation results, intentionally excluded private files, and any legal or funding follow-up.

## Non-negotiable safeguards

- Do not publish student or institutional data.
- Do not commit `.env` files, credentials, database files, private backups, or signing keys.
- Do not use fabricated testimonials, usage statistics, sponsors, citations, or funding claims.
- Do not imply that badges are certifications.
- Do not promise that a custom license can prevent lawful independent development.
- Do not delete local source material merely to keep it out of Git; ignore it precisely.
- Do not rewrite or force-push shared history unless the user explicitly requests it and the exact target has been verified.
