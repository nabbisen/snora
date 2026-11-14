# Handoff — 0.63: non-text contrast, Unit 0 (measure only)

**Ruled.** The owner, 2026-11-14, in scoping 0.63: "the known gap: non-text contrast at 3:1,
measured on drawn pixels, in every state". The roadmap has listed a glyph sweep since RFC-128 Unit 1.
**What it produces.** Measurements and a proposed measure, in one review request. **No product code
and no test committed.** The architect then raises an RFC from them, for the owner.
**Starts after** audit-0-63 request A has been reviewed. Its `non-text-inventory.md` is this unit's
list of subjects.
**Where.** On `main`. The probe is a scratch test, described in the request, not committed.

## Why

WCAG 1.4.11 asks 3:1 for the parts of a control a reader needs to see it, and for graphics that
carry meaning.
- **Today's guards cover only some of snora's non-text elements:**
  - the focus ring (RFC-116);
  - the `border` role against the surfaces, as a token pair;
  - `secondary`'s border, which RFC-129 holds at 3:1 in every state.
- **Nothing measures a glyph:** a notice's dismiss mark, a removable chip's close mark, a
  button's icon.
- **Nothing measures most borders, indicators and tracks as drawn.**
- orbok's conformance record cites snora for 1.4.11.

## What to measure

**For every subject in the inventory, on the pixels drawn:**
- every preset, at rest, hovered, pressed and focused; disabled is exempt, so record it but do not
  count it;
- each of the three neutral surfaces it can sit on;
- **tiny-skia,** and say whether the parity job covers wgpu for the same frames.

**Report, per subject:** the lowest contrast found, where it was found, and the cell. List every
cell under 3:1.

## The measure (the open design question, and the reason for a Unit 0)

A glyph is a thin anti-aliased stroke. Its edge pixels are blends that read low against any
background. A naive "every ink pixel ≥ 3:1" false-fails a correct icon, and a naive "the darkest
pixel ≥ 3:1" passes a glyph a reader cannot see.

**Prototype at least two measures, and propose one:**
- **A. The stroke's core:** the pixels whose coverage is at or above a fixed fraction, against the
  background beside the glyph.
- **B. The glyph's colour as declared** (the fill the SVG or the style function asks for), against
  the background. Then show that the drawn core matches the declared colour within a stated
  tolerance, so the declared colour is what a reader sees.

**For each measure, report:**
1. what it says about every subject;
2. **a plant it catches:** a glyph's colour lowered under 3:1;
3. **a plant it must not fail:** a correct thin icon at its smallest size;
4. its cost per frame.

**Choose by the owner's "robust" and "nobody misunderstands",** not by which passes more:
- the measure must fail when a reader cannot see the glyph;
- it must pass when they can;
- it must be stated in one sentence the book can print.

## Acceptance criteria

1. Every inventory subject measured, every cell under 3:1 listed.
2. Two measures prototyped, each with its plant caught and its correct icon passed, and one
   proposed with the reason.
3. Named `cp` backups and `sha256sum -c` for any plant; never `git checkout` or `git restore`.
4. Nothing committed.
