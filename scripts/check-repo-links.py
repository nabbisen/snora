"""Repo-wide relative Markdown link check (RFC-084 follow-up, 2026-09-01).

Complements `check-built-links.py`, which reads the *built book* under
`docs/book/` and therefore cannot see `rfcs/` at all — where fourteen
handoff back-links had gone stale unnoticed because every RFC moves
between `accepted/`, `done/` and `archive/` as it ships.

Checks every relative `.md` link in every tracked Markdown file outside
`target/`, `.git/`, `docs/book/` and `.git-exclude/`.

Manual for now, per RFC-064's precedent: the count is zero today and
should be stable for a release before a gate is pointed at it. RFC-087
covers wiring the existing manual checks into CI; this one joins that
question rather than pre-empting it.
"""

import os
import re
import sys

SKIP = ("/target", "/.git", "/docs/book", ".git-exclude")
LINK = re.compile(r"\]\((\.[^)#:]+\.md)\)")

broken, checked = [], 0
for root, _, files in os.walk("."):
    if any(s in root for s in SKIP):
        continue
    for name in files:
        if not name.endswith(".md"):
            continue
        path = os.path.join(root, name)
        with open(path, encoding="utf-8", errors="replace") as fh:
            for target in LINK.findall(fh.read()):
                checked += 1
                if not os.path.exists(os.path.normpath(os.path.join(root, target))):
                    broken.append((path, target))

print(f"Relative .md links checked: {checked}")
print(f"Broken: {len(broken)}")
for path, target in broken:
    print(f"  {path} -> {target}")
sys.exit(1 if broken else 0)
