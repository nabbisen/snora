# RFC 086 — The engine's toast colours fail their own thresholds

**Status.** Proposed
**Tracks.** Accessibility. **Severity: High.**
**Found by** the external audit, 2026-09-01 (F-05, F-06).
**Touches.** `crates/snora/src/toast.rs`.
**Release target.** 0.41.0 — **minor.** Rendered appearance changes.

## The two

**F-05 — `ToastIntent::Warning` fails WCAG AA.** `toast.rs:204` pairs a
hard-coded `WARNING_COLOR` fill with `Color::WHITE` text. Audited at **3.18:1**
against a 4.5 requirement.

**F-06 — the `Debug` toast's dismiss `×` is invisible.** `toast.rs:199` uses
`ep.background.strong` for *both* the fill and the mark, audited at **1.58:1**
in light theme.

**Re-measure both before changing anything.** These figures are the auditor's;
this project's rule is that a figure you did not derive is a figure you cannot
defend, and the last audit's arithmetic differed from ours once already.

## Why this is separate from RFC-085

RFC-085 is the **widget** layer (`snora-widgets`) and its unreachable-by-tests
problem. This is the **engine** (`snora`), whose toasts are on the default path
with no features at all. Different crate, different cause — a hard-coded colour
and a same-tier pairing, not a cross-family one — and a different fix.

Grouping them would produce one RFC whose acceptance criteria could be half-met.

## Non-goals

- **No new intent.** The five `ToastIntent` values stay.
- **No token change.** `WARNING_COLOR` is a literal in the engine, not a token;
  fixing it does not touch `snora-design`.
- **Do not route the engine path through `snora-design`.** The engine is
  deliberately token-free — that is the whole `design`-feature boundary.

## Open questions

**Q-1 — does `WARNING_COLOR` change, or does the text on it?** Darkening the
fill keeps the intent's visual identity; switching the text to near-black
changes it and may collide with `Success`/`Danger` at a glance. **Suggest
measuring both and choosing on the number**, stating which identity property was
traded.

**Q-2 — is `Debug`'s `×` a colour fix or a pairing fix?** `background.strong`
for fill and mark is the same category error F-13 is. **Suggest the pairing
fix** — `background.strong.text` — since the tier already carries its own
foreground.

**Q-3 — do the other three intents pass?** The audit measured two.
**Measure all five**, and report the passes as results.

## Acceptance criteria

1. All five intents measured, both themes, figures stated.
2. `Warning` clears 4.5:1 for its text; the `Debug` dismiss mark clears the
   non-text floor.
3. An assertion covers the intent/text pairs, so the next one cannot regress
   silently — this is the engine's first contrast assertion and should be
   derived from the intent enum, not hand-listed.
4. Q-1's trade recorded.
5. Appearance change noted; reference images invalidated.

## Compatibility and security

**Compatibility.** Rendered appearance change on the default path. **Minor**,
with a migration note. **Security.** None.
