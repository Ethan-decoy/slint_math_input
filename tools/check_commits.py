#!/usr/bin/env python3
"""Validate project commit messages using only Python's standard library."""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

HEADER = re.compile(
    r"(?:build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)"
    r"(?:\([a-z0-9]+(?:[-/.][a-z0-9]+)*\))?!?: (\S(?:.*\S)?)$"
)
SKIP_CI = re.compile(
    r"\[(?:skip (?:ci|actions)|(?:ci|actions) skip|no ci)\]|^skip-checks:\s*true\b",
    re.IGNORECASE | re.MULTILINE,
)


def validate(message):
    """Return actionable errors without requiring a particular body language."""
    lines = message.replace("\r\n", "\n").splitlines()
    if not lines or not lines[0]:
        return ["A non-empty Conventional Commit header is required"]
    header = lines[0]
    errors = []
    if not HEADER.fullmatch(header):
        errors.append("Use type(scope): summary; scope is optional and type is lowercase")
    if len(header) > 72:
        errors.append("Keep the header within 72 characters")
    if header.endswith((".", "。")):
        errors.append("Do not end the summary with a period")
    if len(lines) > 1 and lines[1]:
        errors.append("Separate the header and body with a blank line")
    if SKIP_CI.search(message):
        errors.append("CI-skip directives are not allowed in project commits")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--all", action="store_true", help="Check non-merge history reachable from HEAD")
    mode.add_argument("--range", dest="revision_range", help="Check a Git revision range")
    mode.add_argument("--message-file", type=Path, help="Check a proposed commit message")
    mode.add_argument("--title-env", help="Check a PR title from the named environment variable")
    args = parser.parse_args()
    if args.message_file:
        messages = [(str(args.message_file), args.message_file.read_text(encoding="utf-8-sig"))]
    elif args.title_env:
        messages = [("PR title", os.environ.get(args.title_env, ""))]
    else:
        command = ["git", "log", "--no-merges", "--format=%H%x00%B%x00"]
        if not args.all and not args.revision_range:
            command.append("-1")
        command.extend(["--end-of-options", args.revision_range or "HEAD"])
        result = subprocess.run(command, capture_output=True, check=False)
        if result.returncode:
            print(result.stderr.decode("utf-8", errors="replace"), file=sys.stderr)
            return 2
        records = result.stdout.decode("utf-8").split("\0")
        messages = [(records[i].strip()[:12], records[i + 1]) for i in range(0, len(records) - 1, 2)]
    failed = False
    for source, message in messages:
        for error in validate(message):
            print(f"{source}: {error}", file=sys.stderr)
            failed = True
    if not failed:
        print(f"Checked {len(messages)} commit message(s): passed")
    return int(failed)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    raise SystemExit(main())
