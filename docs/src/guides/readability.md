# Readability

How to make text you write on top of snora actually readable — which role to
reach for, and why line-height matters more than most people expect.

For the vocabulary itself — the six roles, their sizes, and which ones
snora's own widgets use — see [Typography](../design/typography.md). This
page does not restate that table; it is about the decision, not the list.

## Picking a role

Match the role to what the text *is*, not to how big you want it to look:

- **`body`** — the default for any sentence or paragraph a user reads to
  understand something: descriptions, help text, notice bodies.
- **`body_small`** — secondary information the user can skim past: metadata,
  timestamps, compact captions.
- **`label`** — short, single-line UI text: button labels, field labels,
  chip text. Not for anything that wraps.
- **`title`** — the name of a card, dialog, or notice. One line, rarely two.
- **`heading`** — marks a section boundary within a screen.
- **`display`** — reserved for the one or two places a screen has a genuine
  headline. If you reach for `display` more than once per screen, it is
  probably the wrong role.

If none of these fit, the answer is almost always `body` or `body_small` —
not a custom size.

## Why line-height matters for prose, and not for labels

A **label** is one line. Line-height has nothing to do — there is no second
line for it to space away from. A **body** paragraph wraps across several
lines, and line-height is what keeps those lines from crowding each other:
too tight, and wrapped prose becomes hard to track from the end of one line
to the start of the next; too loose, and short paragraphs feel disconnected.

This is why `body`'s line-height (`1.4`) is the loosest of the six roles and
`label`'s (`1.2`) is among the tightest — the scale isn't arbitrary, it
tracks how much wrapping each role is expected to do. `title` sits between
the two: usually one line, occasionally two, so it gets a middling `1.3`.

**That scale is calibrated against itself, and separately against iced
0.14's own default line-height, `Relative(1.3)`** — text you never call
`.line_height()` on already renders at 1.3 (see
[Typography](../design/typography.md) for the source citation). Stated
against that baseline, not just against each other: applying `body` or
`body_small` adds air over what you'd get by doing nothing; applying
`title` changes nothing at all, because it *is* 1.3; and applying
`label`, `heading`, or `display` **removes** air relative to doing
nothing — deliberately, because larger or shorter text tolerates (and
usually wants) tighter leading, not because those roles are
under-specified.

**So the practical rule is not "apply line-height to anything that
might wrap" — that reads as uniformly an improvement, and for `title`
and `heading`, the two other roles this guide previously named
alongside `body`/`body_small`, it isn't.** The accurate rule: **apply
`body` or `body_small`'s helper to anything that might wrap** — that is
where it demonstrably helps. Applying `title`'s helper is harmless but
does nothing observable. Applying `heading`'s or `display`'s helper on
wrapping text is a deliberate typographic choice (tighter leading at
larger sizes), not a bug to work around — know that you're trading air
for density there, rather than assuming the helper is free improvement
the way it is for `body`/`body_small`.

Each role has a line-height helper beside its size helper —
`body_line_height`, `title_line_height`, and so on, one per role, in
`snora::design::style::text` — so applying this rule does not mean
reaching through `tokens.typography.<role>.line_height` by hand (RFC-068).
See [Typography § Applying a role to your own text](../design/typography.md#applying-a-role-to-your-own-text)
for the full helper list.

## The floor

**The floor is 12 logical pixels.** Nothing else. Below that, snora's
WCAG-tested contrast pairs stop being a reliable readability guarantee
regardless of how well the contrast ratio itself holds up: size and contrast
both have to clear a floor for text to be legible, and this is the size half of
that pair.

**Separately, and not a second floor: express sizes through a role rather than
a literal.** `body` and `body_small` are the roles for notices, labels and help
content. That is about keeping sizes in one place, not about a minimum — a
role's value is yours to set.

The two combine in the case you may actually want: if your densest surface needs
12px, **redefine `body_small` to 12.0 on your own `Tokens`** and use the role.
The size still comes from the scale, and the floor is met. `Typography` is a
plain struct with every field public, so this is supported rather than a
loophole (see [typography](../design/typography.md)).

An earlier version of this section read *"uses at least `body` or `body_small` —
never a custom size below 12 logical pixels"*, which one consumer reasonably read
as two floors — 14 from the role, 12 from the number — and costed a remediation
at roughly twice what it needed (knotra, 2026-08-19).

**What's actually asserted, and what isn't (RFC-081).**
`text_size_meets_12px_floor_for_every_role` in
`crates/snora-design/src/tests.rs` asserts every role's `size` clears
12.0 in all four **built-in presets** — it proves the presets snora
ships comply, and would catch a future preset edit that dropped a role
below the floor. **It cannot enforce this on your own `Tokens`.**
`Typography`'s fields are public and stay that way (RFC-036's
additive-only covenant freezes that surface), so a custom
`body_small: 8.0` on your own tokens is unreachable by any test snora
ships — the floor above is real guidance, not a guarantee snora
verifies for you once you redefine a role.

## Applying a role

```rust,ignore
{{#include ../../../examples/book_snippets/src/readability.rs:readability_applying_a_role}}
```

Swap `body_size` / `body_line_height` for whichever role's helper pair
fits, per the table on the [Typography](../design/typography.md) page.
