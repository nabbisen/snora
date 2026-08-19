# RFC 072 — Contrast values are bounded below and never above, and we have never said so

**Status.** Accepted (owner, 2026-08-19). Handoff written — see
[`handoffs/072-…`](../handoffs/072-contrast-values-are-bounded-below-and-never-above/implementation-handoff.md).
**Tracks.** API governance / consumer-facing standing answers.
**Found by** the architect, from a **knotra** assertion (2026-08-19) that
depends on the opposite being true.
**Touches.** `docs/src/contributing/api-governance.md`,
`docs/src/guides/accessibility.md`, `docs/src/design/tokens.md`,
`docs/src/contributing/feature-gating-criteria.md` (instance table).
**Release target.** 0.38.1 — documentation only. Can ride with RFC-070/071.

## Summary

Every contrast assertion snora ships is a **floor**. Verified across the whole
suite: `crates/snora-design/src/tests.rs:107` (`r >= min`, the derived mandatory
pairs) and `:302` (`r >= 7.0`, primary text at AAA) are the only two forms, and
both are `>=`. `AA_TEXT`, `FOCUS_MIN` and `NON_TEXT_MIN` are all minimums.
**There is no upper bound on any contrast ratio anywhere in the codebase.**

And RFC-036's covenant permits changing a preset value **only where a contrast
test proves the change fixes an accessibility defect** — so the only change the
covenant allows is one that raises a ratio that was too low.

**We have never told consumers either fact.** `ThresholdClass` is
`pub(crate)`; the covenant page states what may change without saying which
direction; and no consumer-facing page says that a value may become *more*
contrasty at any time.

## The instance

knotra asserts `border` against `surface` **stays below AA (4.5)**, to justify
excluding a neutral notice tone — *"this colour would not be legible as text,
so we do not offer that tone."* The figure (3.50) is right and the reasoning is
sound.

It is an upper bound on a value we bound only from below. Current headroom is
**1.0 ratio point**, and **this has already moved once**: 0.34.0 took `dark`'s
border from 1.32 to 3.50 against `surface`, and `light`'s from 1.28 to 3.12. A
second repair of that shape takes them past 4.5.

**The failure mode is the bad one.** Their test would break *because we fixed
something* — so it presents as a regression and is actually a repair, which is
the hardest kind of breakage for a downstream team to diagnose.

Nothing is broken today, and they are not being asked to act. This RFC exists
so the next consumer does not build the same thing.

## Why documentation and not a letter

Six teams; **each letter costs a hand relay** by our own maintainer, and one
team (tekstide) closed their channel over exactly that cost. A statement in the
book reaches all six at zero relay cost and is present when they look, which a
letter sent today is not when they write the assertion next year.

This is also RFC-059's finding restated: an answer filed where consumers do not
read reaches nobody. **The covenant page is not where a consumer looks before
writing a contrast assertion.**

## What to say

Two facts, stated as a standing answer:

1. **Contrast thresholds are floors.** A role's ratio against its declared
   surfaces is guaranteed to be *at least* its threshold. No maximum is
   guaranteed, now or later.
2. **The only permitted value change raises a ratio that was failing.** So the
   direction a value can move is up, and **we will not commit to keeping any
   colour insufficiently contrasty** — that is not a commitment a design system
   can make.

With the practical consequence spelled out: **do not assert that a snora colour
stays below a threshold.** If your decision depends on a colour being
illegible, assert it against your own colour, which you control.

**State the limit of the guarantee honestly.** The covenant does not promise
that a repair preserves every *other* ratio: changing `border` moves it against
`background`, `surface` and `surface_raised` at once, and only the failing pair
is what the repair is judged on. The floor is the promise; nothing else is.

## Open questions

**Q-1 — where does it go?** `api-governance.md` is the covenant's home and is a
**contributing** page. Consumers writing contrast assertions read
`guides/accessibility.md` and `design/tokens.md`. **Suggest: the full statement
in `api-governance.md`, and a short pointer in `guides/accessibility.md`** —
one canonical text, one place a consumer will actually hit it. Duplicating the
whole thing invites the two copies to drift, which is a defect this project has
shipped twice this month.

**Q-2 — how strongly to phrase it?** As a positive commitment ("we will not
promise a ceiling") rather than an absence ("no ceiling is currently
specified"). An absence reads as an oversight a future release might correct;
the commitment is the actual position and closes the question.

**Q-3 — does the documentation-scope rule's instance table get a row?**
`feature-gating-criteria.md`'s table records claims that reached consumers
wrongly. This is a *standing answer that was invisible* — its third case — with
one known instance. **Suggest yes**, since the table is the record of how this
class keeps recurring, and a case with no instances logged reads as theoretical.

## Acceptance criteria

1. Both facts stated in `api-governance.md`, including the honest limit — a
   repair is judged on the failing pair and preserves nothing else.
2. A pointer from `guides/accessibility.md`, not a second copy.
3. The practical consequence — *do not assert a snora colour stays below a
   threshold; assert against your own* — stated where a consumer reads it.
4. The claim verified before it is written: **every contrast assertion in the
   suite is `>=`**. Re-derive it (`grep` the assert forms) rather than
   inheriting it from this RFC.
5. Q-3's instance row added or explicitly declined with a reason.
6. No code change, no assertion change, no value change.

## Compatibility and security

**Compatibility.** Documentation only. It documents a property the codebase
already has; it does not create one.

**Security.** None.
