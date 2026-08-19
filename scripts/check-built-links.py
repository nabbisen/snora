#!/usr/bin/env python3
"""Built-output link check for the mdBook site (RFC-073).

Parses every `<a href="...">` under `docs/book/` (mdBook's built HTML,
not the `.md` source) and confirms each internal link resolves to a
file that actually exists in the built output.

Why built-output, not source-level: a source-level check confirms every
relative `.md` link in `docs/src` resolves to another `.md` file on
disk. That misses a page that exists on disk but was never wired into
`SUMMARY.md` — mdBook silently never builds it, so a reader gets a 404
on the published site while every source-level check stays green. This
is the check that caught RFC-073's Unit 1 (three migration guides
linked from `guides/migrations.md` but absent from `SUMMARY.md`) and a
second, independent instance in `contributing/recipes.md` (an
`mdBook`-rewrite asymmetry: `README.md` -> `index.html` happens for
`SUMMARY.md`'s own nav entries but not for an ordinary prose link).

Usage:
    mdbook build docs
    python3 scripts/check-built-links.py

Manual only — not wired into CI. Whether it should become a gate is
RFC-071 Q-4's open question and stays deliberately deferred; this
script exists so the check is repeatable by hand, not to answer that
question.

Known limits, both acceptable for a first version:
- URL fragments (`#anchor`) are stripped and not verified to exist on
  the target page.
- Site-root-absolute links (leading `/`, e.g. `book.toml`'s
  `site-url` prefix as seen in `404.html`) are skipped — they resolve
  against the deployed site root, not the local `docs/book/` tree.
"""

import os
import re
import sys
from urllib.parse import unquote, urlparse

BOOK_DIR = "docs/book"
HREF_RE = re.compile(r'href="([^"]+)"')


def main() -> int:
    if not os.path.isdir(BOOK_DIR):
        print(f"error: {BOOK_DIR} not found — run `mdbook build docs` first", file=sys.stderr)
        return 2

    total_links = 0
    internal_links = 0
    broken = []

    for root, _dirs, files in os.walk(BOOK_DIR):
        for fname in files:
            if not fname.endswith(".html"):
                continue
            page_path = os.path.join(root, fname)
            with open(page_path, encoding="utf-8", errors="replace") as f:
                content = f.read()
            for href in HREF_RE.findall(content):
                total_links += 1
                if href.startswith("#"):
                    continue
                if href.startswith("/"):
                    continue  # site-root-absolute; see module docstring
                parsed = urlparse(href)
                if parsed.scheme in ("http", "https", "mailto", "javascript"):
                    continue
                if not parsed.path:
                    continue
                internal_links += 1
                target = unquote(parsed.path)
                resolved = os.path.normpath(os.path.join(root, target))
                if not os.path.isfile(resolved):
                    broken.append((page_path, href, resolved))

    print(f"Total <a href> occurrences scanned: {total_links}")
    print(f"Internal (local-file) links checked: {internal_links}")
    print(f"Broken: {len(broken)}")
    if broken:
        print()
        for page, href, resolved in broken:
            print(f"  {page} -> {href!r} (resolved: {resolved})")
        return 1
    print("No broken internal links found.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
