# RFC 068 — The typography scale is half-tooled: six size helpers, no line-height helper

**Status.** Proposed
**Tracks.** Design vocabulary / readability.
**Found by** the architect, auditing after **knotra** reported measuring **no
line-height set anywhere** across their three crates (2026-08-19).
**Touches.** `crates/snora-style/src/text.rs`,
`docs/src/design/typography.md`, `docs/src/guides/readability.md`,
`CHANGELOG.md`.
**Release target.** 0.38.0 (minor — `snora-style` gains public functions).

## Summary

`TextRole` carries two fields — `size` and `line_height`. `snora-style::text`
provides **six helpers for the first and none for the second**.

An application wanting the size writes `text::body_size(&tokens)`. An
application wanting the line-height reaches into
`tokens.typography.body.line_height` and constructs
`iced::widget::text::LineHeight::Relative` itself. We tooled one half of a
two-field vocabulary and left the other as raw field access.

## Why this is a defect rather than a scope choice

`text.rs`'s own module doc frames the omission as deliberate:

> Line-height **is** configurable through `iced::widget::text` in iced 0.14 …
> applying it is the application's own call.

**The size is equally the application's own call**, and we provide six helpers
for it. The stated reasoning does not distinguish the two halves; it justifies
providing neither or both.

The doc's second sentence is honest and stays true either way: *"Snora's own
prefab widgets do not yet apply it to the text they render internally."* That is
a separate gap (see Non-goals) and RFC-057 already made it visible.

## The evidence that it matters

**knotra measured zero line-height occurrences across three crates**, alongside
38 call sites below the 12px floor. Neither was known to them before reading
`readability.md`. They found the size half because it has a floor and a helper;
they found the line-height half only as an absence.

**orbok is in the same position** — their typography question has been deferred
pending Phase 4, and nothing in their reports indicates line-height is applied.

Two of five adopters, with no line-height applied anywhere, while we publish a
readability guide that treats leading as load-bearing. A vocabulary that is
harder to use than to ignore gets ignored.

## Scope

Add six line-height helpers to `snora-style::text`, one per role, mirroring the
existing size helpers exactly in name and shape.

The values they return are already fixed by `Typography::default_roles()` and
are not changed by this RFC:

| role | size | line_height |
|---|---|---|
| `body` | 16.0 | 1.4 |
| `body_small` | 14.0 | 1.35 |
| `label` | 14.0 | 1.2 |
| `title` | 18.0 | 1.3 |
| `heading` | 24.0 | 1.25 |
| `display` | 32.0 | 1.2 |

## Non-goals

- **snora's own widgets still do not apply line-height.** That is a separate,
  larger question — it is a rendered change to shipped primitives, it interacts
  with orbok's deferred typography assessment, and it needs their Phase 4
  evidence. **Do not fold it in.** This RFC makes the vocabulary usable by
  applications; it does not change a pixel snora renders.
- **No change to any `line_height` value.**
- **No new `TextRole` field**, and no change to `Typography`. RFC-036's covenant
  freezes both; this RFC needs neither.
- **No line-height floor.** `readability.md` has a size floor and no leading
  floor, and inventing one here without evidence would be the shape of defect
  RFC-058 found.

## Open questions

**Q-1 — return `LineHeight` or the raw `f32` multiplier?**

- **`iced::widget::text::LineHeight`** is directly usable —
  `.line_height(text::body_line_height(&tokens))` — and matches how the size
  helpers return `iced::Pixels` rather than `f32`. Consistent with the module's
  existing shape.
- **`f32`** is renderer-agnostic and leaves the caller to choose
  `Relative` vs `Absolute`.

**Suggest `LineHeight::Relative`**, on the consistency argument: the size
helpers already return an iced type, and the multiplier stored in `TextRole` is
*defined* as relative. Returning `f32` would make the caller re-state a fact the
token already fixed.

**Q-2 — does adding these oblige snora's widgets to apply line-height?**
No, and the RFC should say so explicitly, because the obvious next question
after "we have helpers" is "why don't we use them?" The answer is that applying
them changes rendering of shipped primitives and is gated on orbok's evidence —
not that nobody noticed.

**Q-3 — should `readability.md` gain a leading section?** It currently covers
size and contrast. Leading is the third variable and now has tooling. Suggest a
short section pointing at the helpers, with **no floor asserted** — describing
what the roles provide, not mandating a minimum we have no evidence for.

## Acceptance criteria

1. Six line-height helpers exist in `snora-style::text`, named consistently with
   the size helpers, one per role.
2. Q-1 answered, with the reasoning recorded.
3. `text.rs`'s module doc no longer implies line-height is untooled, and keeps
   its accurate statement that snora's own widgets do not apply it.
4. `typography.md` and `readability.md` point at the helpers.
5. Q-2's answer is written down where a reader will hit the question.
6. `render_semantics` passes unmodified; **no rendered output changes** —
   `git diff` shows no change to any widget's rendering path.

## Compatibility and security

**Compatibility.** Purely additive — six new public functions in `snora-style`,
which RFC-036's covenant permits without reopening a gate ("Adding new functions
to the style bridge"). No existing signature changes. **Nothing snora renders
changes.**

**Security.** None.
