# RFC 068 — The typography scale is half-tooled: six size helpers, no line-height helper

**Status.** Accepted (owner, 2026-08-19). Handoff written — see
[`handoffs/068-…`](../handoffs/068-line-height-has-no-helper/implementation-handoff.md).
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

## Resolved questions

**Q-1 — return `LineHeight` or the raw `f32` multiplier? → `LineHeight::Relative`.**
**Ruled by the architect, 2026-08-19.**

The two candidates were `iced::widget::text::LineHeight`, directly usable as
`.line_height(text::body_line_height(&tokens))`, and a raw `f32`, which is
renderer-agnostic and leaves the caller to choose `Relative` vs `Absolute`.

**The ruling is `LineHeight::Relative`**, on consistency: the size helpers
already return an iced type (`body_size` returns `Pixels`, not `f32`), and the
multiplier stored in `TextRole` is *defined* as relative. Returning `f32` would
make the caller re-state a fact the token already fixed, and would leave every
call site free to reinterpret a relative multiplier as an absolute pixel height
— a mistake the token's own meaning forbids.

Implementation notes, verified against iced 0.14:

- `LineHeight` is `iced_core::text::LineHeight`, re-exported at
  `iced::widget::text::LineHeight`. Its variants are `Relative(f32)` and
  `Absolute(Pixels)`. **`snora-style` already depends on `iced`** — no new
  dependency, and no change to the crate's dependency graph.
- The helpers therefore read
  `LineHeight::Relative(tokens.typography.body.line_height)`, mirroring
  `body_size`'s `tokens.typography.body.size.into()` one-liner.
- **`snora-design` is unaffected.** The `f32` stays in the token; only the
  bridge converts. The iced-free constraint on `snora-design` is untouched.

**Q-2 — does adding these oblige snora's widgets to apply line-height? → No.**
**Ruled by the architect, 2026-08-19.**

It does not, and the RFC requires that to be written down where a reader hits
the question — because the obvious next thought after "we have helpers" is "why
don't we use them?", and the answer must not read as "nobody noticed." Applying
them changes the rendering of shipped primitives, and is gated on orbok's Phase
4 evidence.

**Q-3 — a contract, or documentation? → Both, and they cover different halves.**
**Ruled by the architect, 2026-08-19.**

The question is *who gets noticed*, and the answer splits sharply:

| Audience | What can fire | Mechanism |
|---|---|---|
| A snora developer adding a role or a `TextRole` field | **A compile error** | The exhaustive test below |
| A snora developer wiring a helper to the wrong role | **A test failure** | The same test |
| An application developer who never sets line-height | **Nothing we ship** | Docs and API shape only |

### The contract — two axes, both compile-enforced

`Typography` and `TextRole` are **plain structs — neither is
`#[non_exhaustive]`**, unlike `Tokens` and `Palette`. That is the enabling fact:
exhaustive destructuring binds from *any* crate, so the enforcement can live in
`snora-style`, next to the helpers it constrains. `Palette::usages()` had to be
pushed into `snora-design` for exactly the opposite reason (RFC-063); this one
does not.

One test in `crates/snora-style/src/text.rs` destructures both:

```rust
// Exhaustive on the ROLE axis. A seventh role fails to compile (E0027)
// until it is listed here with both helpers. Do NOT add `..`.
let Typography { body, body_small, label, title, heading, display } = t.typography;

for (role, size, line_height) in [
    (body, body_size(&t), body_line_height(&t)),
    // ... one row per role
] {
    // Exhaustive on the FIELD axis. A third TextRole field fails to compile
    // until it is either tooled or explicitly declined here. Do NOT add `..`.
    let TextRole { size: want_size, line_height: want_line_height } = role;
    assert_eq!(size.0, want_size);
    assert_eq!(line_height, LineHeight::Relative(want_line_height));
}
```

The role axis catches a new role arriving untooled. **The field axis is the one
that catches this RFC's own defect class** — a token field growing while the
bridge silently does not follow. It could not have caught the present instance
(both fields have existed since v0.20 and the test is written after the fact);
it fires on the next one.

The equality assertions are not ceremony. Six near-identical one-liners is a
copy-paste bug waiting to happen — `title_line_height` returning `body`'s
multiplier would pass any "the helper exists" check, and is not visible by
reading.

**It also repairs a live instance of the same defect.** `text.rs`'s existing
`sizes_are_positive_and_monotonic` builds a **hand-written array of six**.
Adding a seventh role does not break it. That is the RFC-063 hand-maintained-list
shape, sitting in the file this RFC edits; fold it into the exhaustive test.

### The value floor — declined, deliberately

**No leading floor.** WCAG 2.1 SC 1.4.12 (Text Spacing) is the number that would
be reached for, and it does not say what it appears to: it requires that content
survive *a user setting* line height to 1.5, not that a design system ship 1.5.
Deriving a shipped floor from it would fabricate a threshold from a
misread standard — RFC-058's defect inverted, and this project has made that
exact error once already.

What is defensible is a **definitional sanity bound**: `line_height > 1.0`,
below which lines overlap. `typography.rs`'s doctest already asserts it for
`body` alone; extend it to all six inside the same exhaustive test, **labelled a
sanity bound and not an accessibility threshold.**

### The documentation half, which is not a fallback

Nothing snora ships can fire in an application's source — no lint, no macro, no
build script reaches it. **knotra's zero-occurrences finding would not have been
caught by any contract available to us.** For that half, two levers exist and
both are weak:

1. **API shape.** A `body_line_height` sitting beside `body_size`, same module,
   same naming, is discoverable at the point of use — autocomplete is the closest
   thing to "noticed while implementing" an application developer will get from
   us. This is the main reason the helpers are worth adding at all.
2. **`readability.md` gains a leading section** — short, pointing at the helpers,
   describing what the roles provide. **No floor asserted**, per the above.

## Acceptance criteria

1. Six line-height helpers exist in `snora-style::text`, named consistently with
   the size helpers, one per role.
2. The helpers return `iced::widget::text::LineHeight::Relative` (Q-1, ruled)
   — not `f32`, and not `Absolute`.
3. `text.rs`'s module doc no longer implies line-height is untooled, and keeps
   its accurate statement that snora's own widgets do not apply it.
4. `typography.md` and `readability.md` point at the helpers.
5. Q-2's answer is written down where a reader will hit the question.
6. **The exhaustive test exists in `crates/snora-style/src/text.rs`**,
   destructuring `Typography` on the role axis and `TextRole` on the field
   axis, with no `..` in either pattern, and asserting each helper returns its
   own role's value.
7. **A perturbation demo, not a green run.** Show the compile error from adding
   a seventh role, and from adding a third `TextRole` field, then restore.
   A guard that has never fired is unproven.
8. `sizes_are_positive_and_monotonic`'s hand-written six-element array is gone,
   folded into the exhaustive test.
9. All six roles assert `line_height > 1.0`, commented as a **sanity bound, not
   an accessibility threshold**, and no leading floor is introduced anywhere.
10. `render_semantics` passes unmodified; **no rendered output changes** —
    `git diff` shows no change to any widget's rendering path.

## Compatibility and security

**Compatibility.** Purely additive — six new public functions in `snora-style`,
which RFC-036's covenant permits without reopening a gate ("Adding new functions
to the style bridge"). No existing signature changes. **Nothing snora renders
changes.**

**Security.** None.
