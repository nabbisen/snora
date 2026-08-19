# RFC 067 — Withdrawing a claim does not retract it from consumers who acted on it

**Status.** Proposed
**Tracks.** Documentation / release process. Sixth instance of the RFC-059
class, and the first with a measurable propagation rate.
**Reported by** **knotra** (2026-08-19), with corroborating instances from
**aaai**, **orbok** and **apimokka**.
**Touches.** `docs/src/contributing/release-process.md`,
`docs/src/contributing/feature-gating-criteria.md` (the documentation-scope
rule), `CHANGELOG.md`. **No code.**
**Release target.** 0.37.2.

## Summary

snora has withdrawn **two** consumer-facing claims. **Both propagated into
downstream code or conformance records before withdrawal, and neither
withdrawal reached the consumers who had acted on it.** Every one of the five
known instances was found by the consumer, not by us, and two of them only
while reading a seven-release migration bundle.

We announce corrections thoroughly. We have never named the **action**.

## The evidence

### Withdrawal 1 — `text_muted` is "exempt from mandatory contrast checks" (0.34.0)

An exemption we invented; WCAG grants none. Withdrawn in 0.34.0 with a full
explanation of the defect, the measurement, and the repair.

| consumer | what it did to them |
|---|---|
| **orbok** | WCAG conformance record cited the exemption as justification |
| **knotra** | excluded the role from their WCAG AA suite, **naming our doc comment as the authority**; they render it as select-widget placeholder text |
| **aaai** | excluded it from their contrast test for the same reason, across **28 call sites** — diff line numbers, a "Not selected" status, onboarding steps |

Two of the three render the role as text a user reads to make a decision.

### Withdrawal 2 — a focus ring "cannot be rendered" on iced 0.14 (0.34.0)

Over-scoped; the accurate constraint is narrower. Withdrawn at five sites.

| consumer | what it did to them |
|---|---|
| **apimokka** | written into **RFC MK-023**, their accessibility contract, as a reason full Tab traversal might be unachievable — *"which made a scope decision look like a framework limitation"* |
| **orbok** | carried the same statement *"almost verbatim"* |

**Two withdrawals, five propagations, four consumers.** That is not an anecdote.

## The diagnosis is precise, and it is not "we did not say"

We said. The 0.34.0 CHANGELOG entry runs to a paragraph on the exemption,
including *"that exemption was ours, invented, and it is withdrawn."*

What it never says is what a reader who **relied on it** should now do.

The asymmetry is visible inside that same release. Its *rendered* change —
the border repair — carries an action:

> Re-check any screenshot tests or visual regression baselines that include
> card or dialog borders…

Its *documentary* withdrawal carries none. We have a re-check convention for
changes to pixels and no convention for changes to claims — and a claim is the
thing a consumer is most likely to have copied into their own tree, because it
is the thing that reads like authority.

knotra's formulation, which is the RFC in one line:

> **Withdrawing a claim from your docs does not retract it from consumers who
> already acted on it.**

## Scope

1. **A release-note rule**: when a release withdraws or narrows a claim
   consumers may have relied on, the note must name **the action**, not only the
   correction — *"if you relied on X, re-check Y."*
2. **A release-checklist line** that fires it, in the RFC-059 pattern: the rule
   without the trigger is what produced five instances.
3. **A retroactive re-check line** for both withdrawals, in the next release
   note. Three of six known consumers acted on the `text_muted` exemption and
   two on the focus claim; we know only because four of them told us, so the
   unknown set is not empty.

## Non-goals

- **No code.**
- **No new RFC for each future withdrawal.** The point is a convention cheap
  enough to apply every time.
- **No retraction of the corrections themselves.** Both withdrawals were right.
- **No claim that this catches everything.** A consumer who copied a claim and
  never reads our notes is unreachable, and this RFC does not pretend otherwise.

## Open questions

**Q-1 — how does a withdrawal get detected at release time?** There is no grep
for "we stopped asserting something." Suggest a checklist *question* rather than
a check: *did this release withdraw, narrow, or correct anything we previously
told consumers?* — answered by whoever writes the release notes, who is the
person who knows. Mechanising the question is achievable; mechanising the answer
is not, and pretending otherwise would produce a check that passes vacuously.

**Q-2 — how far back does the retroactive line go?** Both known withdrawals are
from 0.34.0. Suggest naming exactly those two and stopping, rather than auditing
the full history — an audit would be a large, speculative sweep for a class we
have only two confirmed members of.

**Q-3 — does this belong beside the existing documentation-scope rule?**
`feature-gating-criteria.md` already carries a rule covering capabilities that
arrive, leave, or exist as invisible standing answers (RFC-048, widened by
RFC-056 and RFC-059). A withdrawn *claim* is a fourth case of the same shape.
Suggest widening that rule a third time rather than writing a fifth — the
precedent for not adding rules is RFC-059's own Q-1.

## Acceptance criteria

1. The release-note rule is recorded, with the action-not-only-correction
   distinction explicit.
2. A release-checklist line fires it (Q-1).
3. A retroactive re-check line for both 0.34.0 withdrawals appears in the next
   release note, naming what a consumer should verify.
4. Q-3 answered: the rule is widened in place, or a reason recorded for not
   doing so.
5. The propagation evidence is recorded where the rule lives — five instances
   across four consumers is the argument, and a rule without it will read as
   fussiness.

## Compatibility and security

**Compatibility.** Documentation and process only.

**Security.** None directly. Worth noting the shape, though: two consumers
narrowed an **accessibility gate** on our authority, and one of them was
excluding 28 call sites of user-facing text. A withdrawn claim that stays
in force downstream is a defect that outlives its own fix.

## Credit

knotra, who found it while planning a migration and generalised it past their
own instance; aaai, orbok and apimokka, whose reports made it a rate rather than
a story.
