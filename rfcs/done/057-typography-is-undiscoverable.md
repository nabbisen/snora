# RFC 057 — The typography vocabulary is complete and undiscoverable

**Status.** Implemented (v0.33.1)
**Tracks.** Documentation and accessibility. Third instance of the
discoverability failure mode RFC-045 and RFC-048 each fixed once.
**Touches.** `docs/src/design/typography.md` (new), `docs/src/SUMMARY.md`,
`docs/src/guides/accessibility.md`, `README.md`,
`docs/src/contributing/accessibility-checklist.md`,
`crates/snora-design/src/typography.rs` (doc comment only). **No code.**
**Release target.** 0.33.1 (patch — documentation only).

## Summary

snora has a six-role typography scale with size **and** line-height. It is
tested, demonstrated in the design workbench, and **fully usable today through
public API with no change to snora**.

A developer following snora's own documentation would never find it. The
README says typography is their decision; the getting-started guide mentions it
only as a limitation; the consumer accessibility guide says nothing about text
at all; and the one place with real guidance is a **contributor** checklist —
which additionally instructs reviewers that line-height is unusable, which is
false.

The observable result is applications with uniform, flat text. That is
primarily a documentation outcome, not a capability gap.

## What exists

`snora_design::Typography` defines six roles, each a `TextRole { size,
line_height }`: `body`, `body_small`, `label`, `title`, `heading`, `display`.
`snora::design::style::text::*` exposes six size helpers.

**Verified: the complete story compiles today**, with nothing added to snora —

```rust,ignore
use iced::widget::text::LineHeight;

iced::widget::text("wrapping prose")
    .size(snora::design::style::text::body_size(&tokens))
    .line_height(LineHeight::Relative(tokens.typography.body.line_height))
```

`LineHeight::Relative(f32)` takes exactly the multiplier `TextRole.line_height`
stores, and `tokens.typography` is public. Compile-checked against the pinned
iced 0.14, not inferred.

## The four documentation failures

**1. No typography page.** `docs/src/design/` has pages for buttons, cards,
chips, chrome geometry, engine surfaces, high contrast, the style bridge,
notices, progress, theme and tokens. There is no `typography.md`, and
typography appears **nowhere in `SUMMARY.md`** — it is absent from the book's
navigation. It exists only as a struct field inside `tokens.md`.

**2. The consumer accessibility guide says nothing about text.**
`docs/src/guides/accessibility.md` was created in v0.27.1 (RFC-045)
*specifically* so consumers could find accessibility answers without reading
`contributing/`. It covers contrast, direction and non-colour status. It does
not mention text size, roles, line-height, or a minimum readable size.

Meanwhile `contributing/accessibility-checklist.md` has a section titled
**"Typography and line-height"**, including a rule that text must not go below
12 logical pixels. **Contributor guidance exists; consumer guidance does not.**
That is exactly the split RFC-045 was raised to fix for focus visibility,
recurring in accessibility content.

**3. The README points the other way.** Its single typography sentence is a
disclaimer — *"the look of your dialog card and header typography are your
decisions by default"* — and the paragraph advertising Snora Design scopes what
design supplies to **chrome colours as of v0.26**. Typography is not listed.

**4. The checklist tells reviewers not to apply line-height, on a false
premise.**

> Line-height multipliers (stored in `TextRole.line_height`) are used where the
> rendering path supports them. **In iced 0.14, line-height is
> vocabulary-only; the limitation is documented.**

It is not vocabulary-only. This is the control that let the gap persist through
several design releases: not a stale comment but a review gate instructing
people to skip it. `snora-design/src/typography.rs:7` compounds it by saying
line-height configuration "happens in `snora-widgets`" — a crate that never did
it and, since v0.33.0, does not hold the style layer at all.

## The honesty constraint

**The new page must not claim snora applies what it does not.**

snora's own prefab widgets use **two** of the six roles — `label` (7 call
sites) and `body` (2). `body_small`, `title`, `heading` and `display` are
unused by the framework. The notice widget renders its *title* at `label_size`,
not `title_size`.

So the page teaches the vocabulary **for the application's own text**, and
states plainly which roles snora's widgets currently use. A page implying
snora renders a full type hierarchy would create precisely the contradiction
RFC-048 was raised to fix — a document promising behaviour the code does not
deliver.

## Scope

Documentation only. Four deliverables, two of them new pages:

1. **`docs/src/design/typography.md`** (Snora Design chapter): the six roles
   and what each is for, the compile-verified size + line-height snippet, and
   an honest statement of which roles snora's own widgets use.
2. **`docs/src/guides/readability.md`** (Guides chapter): the task-oriented
   page — how to pick a role, why line-height matters for prose, the
   12-logical-pixel floor promoted from the contributor checklist. Links the
   typography page; does not restate it. `guides/accessibility.md` gains one
   line pointing here (Q-2).
3. **README** — correct the scoping sentence so typography is not presented as
   outside what Snora Design supplies.
4. **`accessibility-checklist.md`** — replace the false line-height item, and
   fix `typography.rs`'s stale pointer.

## Non-goals

- **No code.** Nothing in `crates/*/src/**` changes except doc comments.
- **No appearance change**, and no change to which roles the prefab widgets
  use. That is deferred — see below.
- **No font weight or family.** A separate design question about framework
  scope, not a documentation fix.
- **No new style helpers.** Line-height is reachable today; adding
  `*_line_height()` helpers is a convenience to consider later, not needed for
  this.

## Deliberately deferred, with a reason

**Role coverage in the prefab widgets** — including the notice's title using
`title_size` — is an **appearance change**, and **orbok is the only consumer
exercising prefab widgets**. Their chrome-geometry (RFC-040) adoption is still
outstanding and their Phase 4 report is the evidence that would tell us whether
current typography reads correctly at their density.

Stacking a second unadopted appearance change on the one team that would see
it, ahead of the evidence, is the wrong order. This RFC ships the half that
costs them nothing.

## Open questions

**Q-1 — how should a change to an already-shipped design surface be packaged?**

**Corrected 2026-08-15. An earlier revision claimed RFC-039 and RFC-040
disagreed on this. They do not, and the claim was wrong.**

Both did the same thing — added a new opt-in surface and left the existing one
untouched:

| RFC | Existing surface | What it added |
|---|---|---|
| 039 | `snora::render`, byte-for-byte unchanged (G-3) | `snora::design::render` |
| 040 | `snora::widget::*`, *"leaving the existing unstyled set exactly as it is"* | `snora::design::widget::*` |

The error came from reading `chrome-geometry.md`'s 16 → 12 sidebar-gap row as
an in-place change. That row compares the **new** styled variant against the
old hardcoded literal — the difference between `snora::widget::app_side_bar`
and `snora::design::widget::app_side_bar`. No existing call site moved.

**What is genuinely unanswered** is narrower: both RFCs *created* surfaces.
Neither faced changing a design surface that **already exists and has
adopters** — which is what the deferred typography work would be, since
`design::widget::*` shipped in v0.27.0 and orbok uses it.

That is an unencountered case, not an inconsistency, and the established
pattern points at the answer: make it opt-in rather than in-place. It needs
deciding when the deferred RFC is written, not before, and it does not need a
separate decision record.

**Q-2 — where does the readability content live?** **Resolved (owner):
accessibility and readability are different things; link rather than absorb.**

Two pages, in the chapters the book already implies by audience:

| Page | Chapter | Answers |
|---|---|---|
| `design/typography.md` | **Snora Design**, beside Tokens / Buttons / Cards | *what the vocabulary is* |
| `guides/readability.md` | **Guides**, beside Accessibility / Direction | *how do I make my text readable* |

`guides/accessibility.md` gains **one line linking to readability**, not a
section absorbing it — which also avoids the duplication RFC-045 warned
against.

**Optional and separate:** grouping Direction, Accessibility and Readability
under one "usable by people" heading in `SUMMARY.md`. That restructures
existing pages, so it should not ride inside a documentation patch.

## Acceptance criteria

1. `docs/src/design/typography.md` and `docs/src/guides/readability.md` exist and are registered in `SUMMARY.md`, in the chapters named in Q-2.
2. It contains the size + line-height snippet, **compile-checked**, not
   transcribed from this RFC.
3. It states which roles snora's own widgets use, without implying more.
4. `guides/accessibility.md` links to the readability guide and does not
   duplicate it.
5. The checklist's line-height item no longer claims the capability is
   unavailable; `typography.rs`'s pointer is corrected.
6. The README no longer scopes typography outside what Snora Design supplies.
7. `git diff --stat -- 'crates/**/*.rs'` shows **doc-comment lines only**;
   `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** Documentation only. No API, no rendering, no gate rows.

**Security.** None.
