#!/usr/bin/env python3
"""Audit a repository before public GitHub publication."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path


REQUIRED_FILES = (
    "README.md",
    "LICENSE",
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    ".gitignore",
)
SENSITIVE_SUFFIXES = {".db", ".sqlite", ".sqlite3", ".gradia", ".pem", ".key", ".p12", ".pfx"}
PRIVATE_SPREADSHEET_SUFFIXES = {".xlsx", ".xls", ".ods"}
IGNORED_PARTS = {".git", "node_modules", "target", "dist", "build", ".idea", ".vscode"}
TEXT_SUFFIXES = {".md", ".txt", ".yml", ".yaml", ".json", ".toml", ".js", ".ts", ".rs", ".html", ".css"}
PLACEHOLDER_RE = re.compile(r"\b(TODO|TBD|YOUR[_ -]?(?:NAME|EMAIL|USERNAME)|REPLACE[_ -]?ME)\b", re.I)
LOCAL_PATH_RE = re.compile(r"(?:[A-Za-z]:\\Users\\[^\\\s]+|/Users/[^/\s]+|/home/[^/\s]+)")
SECRET_RE = re.compile(
    r"(?:ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|"
    r"AKIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----)"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository", nargs="?", default=".", help="Repository path")
    parser.add_argument("--json", action="store_true", dest="as_json", help="Emit JSON")
    return parser.parse_args()


def should_skip(path: Path, root: Path) -> bool:
    return any(part in IGNORED_PARTS for part in path.relative_to(root).parts)


def add(findings: list[dict[str, str]], level: str, code: str, path: str, message: str) -> None:
    findings.append({"level": level, "code": code, "path": path, "message": message})


def repository_files(root: Path) -> list[Path]:
    try:
        result = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            cwd=root,
            check=True,
            capture_output=True,
        )
        return [root / item.decode("utf-8") for item in result.stdout.split(b"\0") if item]
    except (FileNotFoundError, subprocess.CalledProcessError, UnicodeDecodeError):
        return [path for path in root.rglob("*") if path.is_file()]


def audit(root: Path) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    for required in REQUIRED_FILES:
        if not (root / required).is_file():
            add(findings, "error", "missing-file", required, "Required repository file is missing.")

    license_path = root / "LICENSE"
    readme_path = root / "README.md"
    license_text = license_path.read_text(encoding="utf-8", errors="ignore") if license_path.is_file() else ""
    readme_text = readme_path.read_text(encoding="utf-8", errors="ignore") if readme_path.is_file() else ""
    restrictive = any(term in license_text.lower() for term in ("non-commercial", "noncommercial", "commercial use is prohibited"))
    open_source_claim = re.search(r"(?<!not )\bopen[ -]source\b", readme_text, re.I)
    if restrictive and open_source_claim:
        add(
            findings,
            "error",
            "license-mislabel",
            "README.md",
            "Restrictive licensing is described as open source; use source-available instead.",
        )

    for path in repository_files(root):
        if not path.is_file() or should_skip(path, root):
            continue
        relative = path.relative_to(root).as_posix()
        lower_name = path.name.lower()
        if lower_name in {".env", ".env.local", ".env.production"}:
            add(findings, "error", "credential-file", relative, "Environment credential file must not be published.")
        if path.suffix.lower() in SENSITIVE_SUFFIXES:
            add(findings, "error", "sensitive-binary", relative, "Sensitive database, backup, or key file is present.")
        if path.suffix.lower() in PRIVATE_SPREADSHEET_SUFFIXES:
            add(
                findings,
                "warning",
                "spreadsheet-review",
                relative,
                "Spreadsheet requires explicit privacy review before publication.",
            )
        if path.stat().st_size > 25 * 1024 * 1024:
            add(findings, "warning", "large-file", relative, "File exceeds 25 MiB; use a release artifact or Git LFS.")
        if path.suffix.lower() not in TEXT_SUFFIXES and path.name != "LICENSE":
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if SECRET_RE.search(text):
            add(findings, "error", "secret-pattern", relative, "Possible credential or private key detected.")
        if PLACEHOLDER_RE.search(text):
            add(findings, "warning", "placeholder", relative, "Unresolved placeholder text detected.")
        if LOCAL_PATH_RE.search(text):
            add(findings, "warning", "local-path", relative, "Absolute workstation path detected.")
    return findings


def main() -> int:
    args = parse_args()
    root = Path(args.repository).resolve()
    if not root.is_dir():
        print(f"Repository not found: {root}")
        return 2
    findings = audit(root)
    errors = sum(item["level"] == "error" for item in findings)
    warnings = sum(item["level"] == "warning" for item in findings)
    if args.as_json:
        print(json.dumps({"root": str(root), "errors": errors, "warnings": warnings, "findings": findings}, indent=2))
    else:
        for item in findings:
            print(f"[{item['level'].upper()}] {item['code']}: {item['path']} — {item['message']}")
        print(f"Audit complete: {errors} error(s), {warnings} warning(s).")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
