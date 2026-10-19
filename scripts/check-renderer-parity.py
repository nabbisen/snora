#!/usr/bin/env python3
"""Compare two renderers' drawn elements (RFC-108 R-4).

The two renderers cannot be compared inside one test process:
``iced_test`` fixes its backend from ``ICED_TEST_BACKEND`` when the
first simulator is built, and reaching for ``std::env::set_var`` to get
around that is unsound in a threaded test binary and dishonest about
what was measured. So each rendered CI job runs the same tests with
``SNORA_PARITY_OUT`` set, writes its own file, and this compares the
two across jobs.

Each line is::

    element <TAB> kind <TAB> renderer <TAB> core colour <TAB> x y w h

``kind`` is ``mark`` or ``fill``, and it says which statistic the core
colour is — because a thin mark and a filled box need different ones.

A **mark** is a few pixels on a background, so its core is the pixel
the renderer covered fully; a region average would encode the
antialiasing, which is exactly where the two renderers differ by
design (tiny-skia spreads a thin element over partial rows, wgpu does
not), and would fail on every thin element.

A **fill** is mostly its own colour with a label on top, so its core is
the mode. Reading a fill with the mark statistic returns its *label*,
which is how RFC-102's reintroduced defect first looked like no change
at all: tiny-skia was filling the whole tab and wgpu was not, and the
most-contrasting pixel in both was the black label either way.

Either way the edge pixels are excluded before the statistic is taken.

Fails when:

* the two element sets differ;
* a ``mark`` row's core colour differs at all;
* a ``fill`` row's core colour differs by more than one level in any
  channel (see ``FILL_COLOUR_TOLERANCE``);
* any bound differs by more than ``BOUNDS_TOLERANCE`` physical pixels.

Usage: scripts/check-renderer-parity.py <file-a> <file-b>
"""

import sys
from pathlib import Path

# One physical pixel. Layout is identical across renderers — it is the
# same tree measured by the same code — so this is not absorbing a
# difference in where things are, only the rounding of a fractional
# edge to the physical grid.
BOUNDS_TOLERANCE = 1

# **A deviation from RFC-108 R-4's "exact", forced by measurement, and
# confined to `fill` rows.**
#
# A mark's core is a fully covered pixel of an opaque style colour:
# both renderers write the same bytes, and 45 mark rows agreed exactly
# across three CI runs. A fill's core can be a *composite* — a
# translucent fill over the page — and the two renderers round that
# composite independently. Measured on CI's lavapipe, one row of 92
# differs: `chip-box/chip-sel-dis-design dark`, #35568b on wgpu against
# #36568b on tiny-skia, one level in red. On local hardware wgpu four
# rows do.
#
# One level in 255 is not a difference a user can see, and it is three
# orders below what this check exists to catch: RFC-102's reintroduced
# defect moves the active tab's fill from #ffffff to #5865f2. Marks
# stay exact, so the looser rule reaches only the rows that need it.
#
# **"Exact" is a property of the pair this runs on, not of wgpu.** In
# CI both jobs use the same runner and wgpu is lavapipe, where the 45
# mark rows agree to the byte. Run against a file from a *hardware*
# wgpu and three stock-dark marks come out one level apart
# (#e6e6e6 against #e5e5e5) — so a local two-file comparison may report
# mark failures that CI does not. That is a difference between wgpu
# implementations, not a regression.
FILL_COLOUR_TOLERANCE = 1


def _channels(colour):
    """``#rrggbb`` to three integers."""
    return tuple(int(colour[i : i + 2], 16) for i in (1, 3, 5))


def load(path):
    rows = {}
    renderers = set()
    for number, raw in enumerate(Path(path).read_text().splitlines(), start=1):
        if not raw.strip():
            continue
        parts = raw.split("\t")
        if len(parts) != 5:
            sys.exit(f"{path}:{number}: expected 5 tab-separated fields, got {len(parts)}")
        element, kind, renderer, colour, bounds = parts
        if kind not in ("mark", "fill"):
            sys.exit(f"{path}:{number}: kind {kind!r} is neither 'mark' nor 'fill'")
        try:
            box = tuple(int(v) for v in bounds.split())
        except ValueError:
            sys.exit(f"{path}:{number}: bounds {bounds!r} are not four integers")
        if len(box) != 4:
            sys.exit(f"{path}:{number}: bounds {bounds!r} are not four integers")
        if element in rows:
            sys.exit(f"{path}:{number}: {element!r} recorded twice")
        rows[element] = (kind, colour, box)
        renderers.add(renderer)
    if not rows:
        sys.exit(f"{path}: no parity rows — the tests did not run, or did not record any")
    if len(renderers) != 1:
        sys.exit(f"{path}: expected one renderer per file, found {sorted(renderers)}")
    return renderers.pop(), rows


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: scripts/check-renderer-parity.py <file-a> <file-b>")

    name_a, a = load(sys.argv[1])
    name_b, b = load(sys.argv[2])

    if name_a == name_b:
        sys.exit(
            f"both files were written by {name_a}. Parity needs two different "
            "renderers; one of the jobs fell back."
        )

    failures = []

    only_a = sorted(set(a) - set(b))
    only_b = sorted(set(b) - set(a))
    for element in only_a:
        failures.append(f"{element}: drawn by {name_a} and missing from {name_b}")
    for element in only_b:
        failures.append(f"{element}: drawn by {name_b} and missing from {name_a}")

    for element in sorted(set(a) & set(b)):
        kind_a, colour_a, box_a = a[element]
        kind_b, colour_b, box_b = b[element]
        if kind_a != kind_b:
            failures.append(
                f"{element}: recorded as {kind_a} on {name_a} and {kind_b} on "
                f"{name_b} — the two are not comparable"
            )
            continue
        if colour_a != colour_b:
            allowance = FILL_COLOUR_TOLERANCE if kind_a == "fill" else 0
            try:
                drift_rgb = [
                    abs(x - y) for x, y in zip(_channels(colour_a), _channels(colour_b))
                ]
            except ValueError:
                sys.exit(f"{element}: {colour_a!r} or {colour_b!r} is not #rrggbb")
            if max(drift_rgb) > allowance:
                failures.append(
                    f"{element}: {kind_a} core colour {colour_a} on {name_a} against "
                    f"{colour_b} on {name_b}"
                    + (f" — off by {drift_rgb}, over the {allowance}-level allowance"
                       if allowance else "")
                )
        drift = [abs(x - y) for x, y in zip(box_a, box_b)]
        if max(drift) > BOUNDS_TOLERANCE:
            failures.append(
                f"{element}: bounds {box_a} on {name_a} against {box_b} on "
                f"{name_b} — off by {drift}, over the {BOUNDS_TOLERANCE}px allowance"
            )

    if failures:
        print(
            f"{len(failures)} parity failure(s) between {name_a} and {name_b}:",
            file=sys.stderr,
        )
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        sys.exit(1)

    print(f"OK: {len(a)} elements agree between {name_a} and {name_b}.")


if __name__ == "__main__":
    main()
