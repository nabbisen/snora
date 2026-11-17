#!/usr/bin/env python3
"""RFC-136 R-3: every Rust snippet in the book, outside the migration guides, is compiled or marked with its reason.

A `rust,ignore` fence in `docs/src` (outside `guides/migration-*.md`, which record the APIs of their time on purpose) passes if:

  * **compiled:** its body is `{{#include ...examples/book_snippets/src/<module>.rs:<anchor>}}`, and that file and anchor
    exist (the crate `examples/book_snippets` is built by the workspace, so the snippet compiles when it does); or
  * **marked:** the line before the fence is `<!-- not compiled: <reason> -->` with a reason a reader can check.

It fails on an unmarked snippet, on a marker with no reason (or the reason "illustrative", which is not one), on a marker that
is not directly above a `rust,ignore` fence, and on an include of a file or an anchor that is not there.

Usage: python3 scripts/check-book-snippets.py [--root DIR]   (exit 0: every snippet is accounted for)
"""
import glob
import os
import re
import sys

ROOT = os.environ.get("BOOK_SNIPPETS_ROOT") or (sys.argv[sys.argv.index("--root") + 1] if "--root" in sys.argv else ".")
MARKER = re.compile(r"^\s*<!--\s*not compiled:(.*?)-->\s*$")
FENCE = re.compile(r"^(\s*)```(\S*)\s*$")
INCLUDE = re.compile(r"\{\{#include\s+(\S+?):(\S+?)\}\}")
NOT_REASONS = {"illustrative", "illustration", "example", "an example", "for illustration", "n/a", "todo", "tbd"}


def reasons_ok(reason: str) -> bool:
    r = reason.strip().strip(".").lower()
    return bool(r) and r not in NOT_REASONS


def anchors_of(path: str) -> set:
    try:
        text = open(path, encoding="utf-8").read()
    except OSError:
        return None
    return {m.group(1) for m in re.finditer(r"//\s*ANCHOR:\s*(\S+)", text)}


def main() -> int:
    problems = []
    compiled = marked = 0
    pages = sorted(glob.glob(os.path.join(ROOT, "docs/src/**/*.md"), recursive=True))
    assert pages, f"no pages under {ROOT}/docs/src: the glob stopped reading"
    for page in pages:
        rel = os.path.relpath(page, os.path.join(ROOT, "docs/src"))
        in_scope = not (rel.startswith("guides/") and os.path.basename(rel).startswith("migration-"))
        lines = open(page, encoding="utf-8").read().split("\n")
        i = 0
        while i < len(lines):
            m = FENCE.match(lines[i])
            if not m:
                # A marker that is not directly above a fence it can be about.
                if in_scope and MARKER.match(lines[i]):
                    nxt = FENCE.match(lines[i + 1]) if i + 1 < len(lines) else None
                    if not nxt or nxt.group(2) not in ("rust,ignore", "ignore"):
                        problems.append(f"{rel}:{i + 1}: a `not compiled` marker that is not directly above a `rust,ignore` fence")
                i += 1
                continue
            tag = m.group(2)
            j = i + 1
            while j < len(lines) and not re.match(r"^\s*```\s*$", lines[j]):
                j += 1
            body = lines[i + 1 : j]
            if in_scope and tag in ("rust,ignore", "ignore"):
                where = f"{rel}:{i + 1}"
                marker = MARKER.match(lines[i - 1]) if i > 0 else None
                inc = INCLUDE.search("\n".join(body))
                if inc:
                    target = os.path.normpath(os.path.join(os.path.dirname(page), inc.group(1)))
                    anchors = anchors_of(target)
                    if "examples/book_snippets/src/" not in target.replace(os.sep, "/"):
                        problems.append(f"{where}: an include that is not of `examples/book_snippets/src/` ({inc.group(1)}): only that crate is built")
                    elif anchors is None:
                        problems.append(f"{where}: includes {inc.group(1)}, which is not a file")
                    elif inc.group(2) not in anchors:
                        problems.append(f"{where}: includes anchor `{inc.group(2)}`, which {os.path.basename(target)} does not have")
                    else:
                        compiled += 1
                elif marker:
                    if reasons_ok(marker.group(1)):
                        marked += 1
                    else:
                        problems.append(f"{where}: a `not compiled` marker with no reason a reader can check ({marker.group(1).strip()!r})")
                else:
                    problems.append(f"{where}: a `rust,ignore` snippet that is neither compiled (`{{{{#include}}}}` from examples/book_snippets) nor marked (`<!-- not compiled: <reason> -->` on the line before the fence)")
            i = j + 1
    for p in problems:
        print("check-book-snippets:", p, file=sys.stderr)
    print(f"check-book-snippets: {compiled} compiled, {marked} marked, {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
