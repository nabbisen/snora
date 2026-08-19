# RFC 070 — The typography scale was never stated against iced's default, and half of it is tighter

**Status.** Accepted (owner, 2026-08-19). Handoff written — see
[`handoffs/070-…`](../handoffs/070-the-scale-is-uncalibrated-against-iceds-default/implementation-handoff.md).
**Tracks.** Design vocabulary / readability.
**Found by** **orbok**, answering RFC-068 Q-2's evidence request (2026-08-19).
Verified against the pinned iced 0.14 by the architect before acceptance.
**Touches.** `docs/src/guides/readability.md`, `docs/src/design/typography.md`,
`crates/snora-style/src/text.rs`, `crates/snora-design/src/typography.rs`.
**Release target.** 0.39.0 if any value changes; 0.38.1 if documentation only.

## Summary

**iced 0.14's default line-height is `Relative(1.3)`.** Text that never calls
`.line_height()` is already rendered at 1.3 — `Format::default()` sets
`line_height: LineHeight::default()` (`iced_core-0.14.0/src/widget/text.rs:290`),
and `impl Default for LineHeight` returns `Relative(1.3)`
(`iced_core-0.14.0/src/text.rs:215-219`).

Our six roles were calibrated **against each other** and never against that
baseline. Stated against it:

| role | ours | vs iced's 1.3 | effect of applying our helper |
|---|---|---|---|
| `body` | 1.4 | **+0.10 looser** | more air — the intended benefit |
| `body_small` | 1.35 | **+0.05 looser** | slightly more air |
| `title` | 1.3 | **identical** | **literally nothing** |
| `heading` | 1.25 | **−0.05 tighter** | **less air than doing nothing** |
| `label` | 1.2 | −0.10 tighter | less air |
| `display` | 1.2 | −0.10 tighter | less air |

**Two of six roles are looser than the default. One is a no-op. Three are
tighter.**

## Why this is a defect and not a curiosity

`readability.md` tells applications:

> The practical rule: **apply line-height to anything that might wrap**
> (`body`, `body_small`, and any `title`/`heading` text that isn't guaranteed
> short), and don't worry about it for text you know is always one line.

Of the four roles that sentence names for wrapping text: `body` helps,
`body_small` helps slightly, **`title` does nothing at all**, and **`heading`
makes wrapped prose tighter than if the application had ignored our advice.**

The guidance is written as though applying a role's line-height is always an
improvement over not applying it. Against the actual renderer default, that is
true for two roles out of four named, and backwards for one.

RFC-068 shipped six helpers, one of which (`title_line_height`) has **no
observable effect on any surface** — a helper whose entire behaviour is to
restate the default. That is worth saying out loud rather than leaving a
consumer to discover it.

## The evidence, and how it arrived

orbok adopted line-height at **28 of 125 candidate call sites**, and adopted
**only `body` and `body_small`** — explicitly because those are the only two
roles looser than iced's default, so adopting the full scale "would have made
most of our text more crowded, which was the opposite of the goal."

They reached that by measurement, from outside, while we shipped a readability
guide that never mentions the baseline. **A consumer had to discover our own
scale's relationship to the renderer we are built on.**

## Scope

**In scope, certainly:**

1. State iced's default (`Relative(1.3)`) in `typography.md`'s role table as a
   column or an adjacent note, so every role's value is readable against it.
2. Fix `readability.md`'s practical rule. It must not imply that applying a
   role's line-height is uniformly an improvement.
3. Say plainly that `title_line_height` restates the default and changes
   nothing — in the helper's own doc comment, where a caller will read it.

**In scope, open (see Q-1):** whether any *value* changes.

## Non-goals

- **No change to how the roles relate to each other.** The internal scale is
  coherent and was designed deliberately; this RFC is about the missing
  baseline, not about re-tuning the ladder.
- **No new role, no new field.** RFC-036's covenant freezes both.
- **No change to `snora-style`'s function set.** Six helpers stay six helpers
  even if `title_line_height` is a no-op — removing it would break the
  symmetry the two-axis contract in `text.rs` enforces, and symmetry is worth
  more than deleting one redundant call.

## Open questions

**Q-1 — do any values change?** RFC-036 permits changing a preset value **only
where a test proves an accessibility defect**, recorded as *Fixed*. Is
`heading` at 1.25 — tighter than the renderer default, on text our own guidance
says may wrap — such a defect?

**Argument for yes:** we tell applications to apply it to wrapping text, and
doing so demonstrably reduces leading below what they would get for free.

**Argument for no:** 1.25 on a 24px heading is 30px of line box, and large text
tolerates tighter leading — that is why the ladder tightens as size grows, and
it is standard typographic practice. The defect may be entirely in the
**guidance**, which tells people to apply it to wrapping text without saying
what they are trading.

**No recommendation yet.** This needs a contrast-style demonstration, not an
opinion: measure a wrapped heading at 1.25 against 1.3 and state what changes.
If nothing legibility-relevant changes, the fix is documentation only and the
covenant is not touched.

**Q-2 — does `readability.md` need a baseline-relative rule?** Something like
*"applying `body` or `body_small` adds air; applying `heading`, `label` or
`display` removes it; `title` changes nothing"* is accurate and immediately
actionable, but it is also six facts where the current text has one rule.
Suggest stating the baseline once and letting the table carry the rest.

**Q-3 — should `Tokens`' own docs carry the baseline?** `typography.rs`'s
`TextRole` doc says *"Line-height multiplier (e.g. `1.4`), relative to `size`"*
— true, and silent on what a reader gets by not setting it at all.

## Acceptance criteria

1. iced's `Relative(1.3)` default is stated in `typography.md`, cited to the
   source file, and every role's value is readable against it.
2. `readability.md` no longer implies applying a role's line-height is
   uniformly an improvement.
3. `title_line_height`'s doc comment says it restates the renderer default.
4. Q-1 answered with a demonstration, not an assertion — and if any value
   changes, it is recorded as **Fixed** per RFC-036 and the migration note
   names what to re-check (RFC-067).
5. The six helpers still exist and the two-axis contract in `text.rs` is
   unchanged.

## Compatibility and security

**Compatibility.** Documentation-only unless Q-1 changes a value, in which case
it is a rendered change for any application applying that role, and a minor.

**Security.** None.
