# RFC 045 — Assistive technology: stated position, and bounding the ABDD claim

**Status.** Proposed
**Tracks.** Accessibility governance. Answers a downstream question raised
by the apimokka team (2026-08-04) and corrects a claim snora makes about
itself.
**Touches.** `README.md`, `docs/src/introduction.md`,
`docs/src/getting-started/05-when-to-use.md`,
`docs/src/contributing/semantic-accessibility.md`, `docs/src/SUMMARY.md`,
`docs/src/contributing/design-decisions.md`. **No code.**

## Summary

snora presents itself around **ABDD — "Accessible By Default and by
Design"** and ships two accessibility documents. Everything implemented is
*visual* accessibility: contrast-tested presets, logical layout direction,
non-colour status encoding. There is **no accessibility tree, no AccessKit
integration, and no semantic identifiers anywhere in the crates** — a
screen reader sees nothing.

This RFC does two things, neither of which is a feature:

1. **States snora's position** on assistive technology, so downstream teams
   can cite an intent rather than infer one from silence.
2. **Bounds the ABDD claim**, because the name invites a broader reading
   than the implementation supports.

## Motivation

A downstream team preparing UX acceptance sessions asked directly:

> does snora have a position on AccessKit, which iced has discussed
> integrating? Would snora adopt it when iced exposes it, or is assistive
> technology considered out of scope in favour of visual accessibility?
> … a framework built around accessible design has a distance between that
> claim and what assistive technology can reach, and downstream teams
> inherit it.

That is a fair characterisation and this RFC does not soften it. They were
about to record "focus visibility could not be verified" in acceptance
evidence and would rather cite a known upstream position than a silence.

**Verified before writing:** `grep -rniE "accesskit|accessibility_tree|
widget::Id|semantic_id" crates/*/src/` returns nothing. The claim holds.

## The honest problem is the name, not the gap

The gap itself is defensible. An accessibility tree is not something a
layout framework can supply on its own — it requires the widget toolkit to
expose one, and iced 0.14 does not.

What is **not** defensible is that snora's own framing invites a reading
its implementation does not support. "Accessible By Default and by Design"
is heard as covering assistive technology. ABDD's actual definition —
logical edges and layout direction, plus visual accessibility — is narrower
and is stated correctly *inside* the documentation. But a reader meets the
name first, in the README, and the name does the claiming.

This is snora's to fix, not iced's.

## Position (for adoption)

> **snora will integrate an accessibility tree when iced exposes one.**
> Until then, ABDD means layout-direction correctness and visual
> accessibility — contrast, logical edges, non-colour status encoding — and
> snora states that boundary plainly rather than implying more.
>
> snora will **not** build a parallel accessibility abstraction of its own
> in the interim.

The last clause matters. Building a snora-owned accessibility layer ahead
of iced would repeat DEC-02's original mistake — a parallel abstraction
duplicating what the toolkit will provide — in a domain where getting it
wrong is worse than in theming. When iced exposes an API, snora integrates
it; that is the same relationship snora has to iced's `Theme`.

## Goals

- G-1. A stated, citable position on assistive technology.
- G-2. The ABDD claim bounded wherever it is made user-facing.
- G-3. Accessibility documentation reachable by a *consumer*, not only by
  a contributor.
- G-4. A recorded trigger, so the position is revisited rather than
  forgotten.

## Non-goals

- **N-1. No accessibility tree, no AccessKit integration.** Blocked on
  iced; this RFC states intent, not implementation.
- **N-2. No snora-owned interim accessibility abstraction.** See above.
- **N-3. No weakening of the existing visual-accessibility work.** The
  contrast tests, presets and ABDD checklist stand unchanged.
- **N-4. No change to the term ABDD itself.** It is bounded, not renamed —
  renaming would invalidate every existing reference for no gain.

## Proposed changes

### C-1 — State the position

Add the position above to `docs/src/contributing/semantic-accessibility.md`
as a top-level section, and record it in `design-decisions.md` as a
decision with a **reconsideration trigger: iced exposes an accessibility
API**. It belongs in the decision register because it is exactly the kind
of thing a future contributor would otherwise re-litigate.

### C-2 — Bound the ABDD claim where it is made

ABDD appears user-facing in `README.md`, `docs/src/introduction.md`, and
`docs/src/getting-started/05-when-to-use.md`. Each should state, briefly
and without apology, what ABDD covers **and what it does not**: layout
direction and visual accessibility, not assistive-technology support.

One clause each. This is not a disclaimer campaign — it is making the name
honest at the point a reader meets it.

### C-3 — Make the accessibility docs reachable by consumers

Both accessibility documents live under `docs/src/contributing/`, which
reads as "for people changing snora" rather than "for people depending on
it." A downstream team auditing accessibility, looking specifically for the
focus-state limitation, **did not find it** — although it is documented
there in a dedicated section.

The content is not the problem; its location is. Surface it from the
consumer-facing part of the book — a guide-level accessibility page that
links the existing material, or a cross-reference from the getting-started
path. Do not duplicate the content; duplication drifts.

## Compatibility, security, testing

**Compatibility.** Documentation only. No API, no behaviour, no dependency
change.

**Security.** Not affected.

**Testing.** `mdbook build docs` and `mdbook test docs`. There is nothing
else to test — that is inherent to a governance RFC and should not be
disguised.

## Alternatives considered

- **Declare assistive technology permanently out of scope.** Defensible for
  a layout framework, and it would at least be honest. Rejected because it
  conflicts with how snora presents itself — a framework named for
  accessible design declaring AT out of scope would be a stranger position
  than the one proposed, and would foreclose an integration that costs
  little once iced provides the hook.
- **Build a snora-owned interim.** Rejected — see the position statement.
- **Say nothing and wait for iced.** This is the status quo, and it is what
  produced the question. Silence is itself a position; it is just an
  unstated one that downstream teams inherit without being able to plan
  around it.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The position reads as a commitment with a date | Medium | Medium | Word it as conditional on iced, with no timeline — because there is none to give |
| Bounding the claim reads as weakening accessibility | Medium | Low | The visual work is unchanged and strong; the change is to stop implying more than is there |
| iced exposes an API and nobody notices | Medium | Medium | Recorded as a reconsideration trigger in the decision register, where the project already tracks such things |

## Acceptance criteria

1. The position is stated in `semantic-accessibility.md` and recorded in
   `design-decisions.md` with a reconsideration trigger.
2. ABDD is bounded in all three user-facing locations.
3. Accessibility documentation is reachable from the consumer-facing part
   of the book without duplicating content.
4. No code, no API, no gate row changes.
5. `mdbook build docs` and `mdbook test docs` pass.

## Release implications

Ships as **0.27.1**, a documentation patch. It is deliberately first in the
sequence: it is the downstream team's most time-sensitive need (acceptance
evidence), and it costs a day.
