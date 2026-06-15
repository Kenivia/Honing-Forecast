#!/usr/bin/env python3
"""
generate_release_notes.py

Reads NEXT_PUBLISH_VERSION.txt, generates a release notes markdown file
(NEXT_PUBLISH_VERSION.md) containing commits not previously published,
and updates published_commits.json with the newly included hashes.

published_commits.json schema:
{
  "v1.0.0": ["hash1", "hash2", ...],
  "v1.1.0": ["hash3", ...],
  ...
}

Re-running for the same version replaces that version's entry, so you can
regenerate release notes (e.g. after a formatting change) without those
commits being considered "already published" to a different version.
"""

import json
import subprocess
import sys
from datetime import datetime
from pathlib import Path

# ── Config ────────────────────────────────────────────────────────────────────

GITHUB_COMMIT_BASE = "https://github.com/Kenivia/Honing-Forecast/commit/"
VERSION_FILE       = "./scripts/change_log_helper/NEXT_PUBLISH_VERSION.txt"
PUBLISHED_JSON     = "./scripts/change_log_helper/published_commits.json"

# ── Helpers ───────────────────────────────────────────────────────────────────

def read_version() -> str:
    path = Path(VERSION_FILE)
    if not path.exists():
        sys.exit(f"[error] '{VERSION_FILE}' not found.")
    version = path.read_text(encoding="utf-8").strip()
    if not version:
        sys.exit(f"[error] '{VERSION_FILE}' is empty.")
    return version


def load_published_data() -> dict[str, list[str]]:
    """Returns the full version→hashes map from disk."""
    path = Path(PUBLISHED_JSON)
    if not path.exists():
        print(f"[info] '{PUBLISHED_JSON}' not found — starting fresh.")
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            sys.exit(f"[error] '{PUBLISHED_JSON}' must contain a JSON object.")
        return data
    except json.JSONDecodeError as exc:
        sys.exit(f"[error] Could not parse '{PUBLISHED_JSON}': {exc}")


def hashes_from_other_versions(published_data: dict[str, list[str]], current_version: str) -> set[str]:
    """
    Returns only the hashes that were published under a *different* version.
    Hashes belonging to the current version are intentionally excluded so
    that re-running the script regenerates the current release's notes.
    """
    return {
        h
        for version, hashes in published_data.items()
        if version != current_version
        for h in hashes
    }


def save_published_data(published_data: dict[str, list[str]]) -> None:
    Path(PUBLISHED_JSON).write_text(
        json.dumps(published_data, indent=2),
        encoding="utf-8",
    )
    total = sum(len(v) for v in published_data.values())
    print(f"[info] Updated '{PUBLISHED_JSON}' ({len(published_data)} versions, {total} total hashes).")


def get_git_commits() -> list[dict]:
    """
    Returns all commits from git log, newest first.
    Each entry: { hash, author, subject, timestamp }
    """
    sep = "SeparatorThatDefinitelyWontAppearInTheMessage"
    fmt = sep.join(["%H", "%an", "%s", "%ct"])  # %ct = unix timestamp

    result = subprocess.run(
        ["git", "log", f"--pretty=format:{fmt}"],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        sys.exit(f"[error] git log failed:\n{result.stderr.strip()}")

    commits = []
    for line in result.stdout.splitlines():
        parts = line.split(sep)
        if len(parts) != 4:
            continue  # skip malformed lines
        full_hash, author, subject, ts_str = parts
        commits.append({
            "hash":      full_hash.strip(),
            "author":    author.strip(),
            "subject":   subject.strip(),
            "timestamp": int(ts_str.strip()),
        })

    # git log already outputs newest-first; sort explicitly to be safe
    commits.sort(key=lambda c: c["timestamp"], reverse=True)
    return commits


def format_commit_row(commit: dict) -> str:
    short_hash = commit["hash"][:6]
    url        = f"{GITHUB_COMMIT_BASE}{commit['hash']}"
    author     = commit["author"].replace(" ", "\u00a0")  # non-breaking spaces in names look cleaner
    if author == "Kenivia":
        author = ""
    else:
        author = "@" + author
    subject    = commit["subject"]
    date_str   = datetime.fromtimestamp(commit["timestamp"]).strftime("%Y-%m-%d")

    return f"| {date_str}|{subject} |  {author} | [{short_hash}]({url})  |"


def build_markdown(version: str, new_commits: list[dict]) -> str:
    lines = [
        f"# {version}\n",
               "## New features",         
               "## Fixes & improvements",
                      "## Commits ",
        "| Date | Commit Message |  Contributor | Commit  |",
        "|--------|--------|---------|------|",
    ]
    for commit in new_commits:
        lines.append(format_commit_row(commit))

    if not new_commits:
        lines.append("| — | — | _No new commits since last release_ | — |")

    return "\n".join(lines) + "\n"


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    version       = read_version()
    output_file   = Path("./public/change-logs/" + version + ".md")  # e.g. v1.2.3.md

    published_data        = load_published_data()
    excluded_hashes       = hashes_from_other_versions(published_data, version)

    all_commits  = get_git_commits()
    new_commits  = [c for c in all_commits if c["hash"] not in excluded_hashes]

    print(f"[info] Version       : {version}")
    print(f"[info] Total commits : {len(all_commits)}")
    print(f"[info] New commits   : {len(new_commits)}")
    if version in published_data:
        print(f"[info] Re-generating existing version '{version}' (previous entry will be replaced).")

    markdown = build_markdown(version, new_commits)
    output_file.write_text(markdown, encoding="utf-8")
    print(f"[info] Written '{output_file}'.")

    # Overwrite this version's entry; all other versions are left untouched
    published_data[version] = [c["hash"] for c in new_commits]
    save_published_data(published_data)


if __name__ == "__main__":
    main()