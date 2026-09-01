# Developer Handoff — RFC-067 withdrawn claims

**Governing RFC.** **RFC-067** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-067 — Accepted (owner, 2026-08-19).
**Release target.** 0.37.2. **Documentation and process only** — no code.
**Implementation units.** One.

---

## 1. Task title

Widen the documentation-scope rule to cover withdrawn claims, add the
release-checklist question that fires it, and issue the retroactive re-check
line for the two withdrawals already known to have propagated.

## 2. Purpose

Two consumer-facing claims have been withdrawn. **Both had already propagated
downstream, and neither withdrawal reached the consumers who acted on it.** Five
instances across four consumers — every one found by the consumer, two of them
only while reading a seven-release migration bundle.

We announce corrections thoroughly. We have never named the **action**.

## 3. The distinction the whole task turns on

Not *"tell people"* — we told them. The 0.34.0 entry runs to a paragraph
including *"that exemption was ours, invented, and it is withdrawn."*

**Announcing a correction is not prompting an action.** The same release proves
we already know the difference: its *rendered* change carries

> Re-check any screenshot tests or visual regression baselines that include
> card or dialog borders…

and its *documentary* withdrawal carries nothing.

So the rule is narrow and concrete: **when a release withdraws or narrows a
claim consumers may have relied on, the note names what to re-check** — in the
same voice, and in the same place, as the rendered-change re-check line that
already exists.

If your implementation produces a general exhortation to communicate clearly,
it has missed the target.

## 4. Q-3 is decided: widen the existing rule, do not write a fifth

`feature-gating-criteria.md:182` § *"Documentation scope when a capability
arrives, leaves, or a standing answer is invisible"* already covers three cases
and carries a five-row instance table.

A withdrawn claim is a **fourth case of the same shape**. Widen the rule in
place and retitle the section accordingly. Do not create a new rule page —
RFC-059's own Q-1 settled that a fifth rule would land in `contributing/` and
reproduce the defect it describes.

**Add the five propagation instances to the existing table**, which is the
argument. A rule without them reads as fussiness; with them it reads as a rate.

| withdrawal | consumer | what it did to them |
|---|---|---|
| `text_muted` exemption (0.34.0) | orbok | WCAG conformance record cited it as justification |
| | knotra | excluded the role from their WCAG AA suite, **naming our doc comment as the authority**; renders it as select-widget placeholder text |
| | aaai | excluded it from their contrast test — **28 call sites**: diff line numbers, a "Not selected" status, onboarding steps |
| focus ring "cannot be rendered" (0.34.0) | apimokka | written into **RFC MK-023**, their accessibility contract, as a reason full Tab traversal might be unachievable |
| | orbok | carried the same statement *"almost verbatim"* |

## 5. Q-1 is decided: mechanise the question, not the answer

There is no grep for *"we stopped asserting something."* Do not invent one — a
check that cannot detect its subject passes vacuously, which is the failure mode
RFC-062 and RFC-064 each found.

Add a **checklist question** to `release-process.md`, answered by whoever writes
the release notes, because that is the person who knows:

> Did this release withdraw, narrow, or correct anything we previously told
> consumers? If so, the note names **what to re-check**, not only what changed.

Place it next to the existing documentation-scope checklist line, so the two
are read together.

## 6. Q-2 is decided: retroactive line names exactly the two known withdrawals

Both are from 0.34.0. Name those two and stop.

**Do not audit the full history.** That would be a large speculative sweep for a
class with two confirmed members, and its results would be unverifiable — we
know about these five instances only because four consumers told us.

The retroactive line goes in **0.37.2's release note**, and should say plainly
what a consumer should check:

- **If you excluded `text_muted` from a contrast or accessibility suite on the
  strength of snora's documentation, that exemption was invented and is
  withdrawn — re-check the role.** It is asserted here at `AA_TEXT` against all
  three surfaces as of 0.34.0.
- **If you recorded that a focus ring cannot be rendered on iced 0.14, that
  statement was over-scoped** — an application owning focus as its own state can
  style it today. Re-check any scope decision that cited it.

State that we know of five instances and cannot know the full set. Do not imply
the list is complete.

## 7. Change scope

| File | Purpose |
|---|---|
| `docs/src/contributing/feature-gating-criteria.md` | widen the rule; add the five instances (§4) |
| `docs/src/contributing/release-process.md` | the checklist question (§5) |
| `CHANGELOG.md` | **Changed**, plus the retroactive re-check line (§6) |

## 8. Explicit non-change scope

Do **not**:

- **Write a fifth rule page** (§4).
- **Build a detector** for withdrawn claims (§5).
- **Audit the history** for further withdrawals (§6).
- **Retract either correction.** Both withdrawals were right; this is about
  reaching the people who had acted on the old claim.
- **Imply the propagation list is complete.** Four consumers told us; the others
  were not asked.
- **Change any code**, `render_semantics`, or any preset value.

## 9. Required tests

```bash
mdbook build docs && mdbook test docs
git diff --stat -- 'crates/**'   # MUST be empty
```

No code changes, so the compile gates are unaffected — the release checklist
runs them regardless.

## 10. Required evidence

- The widened rule, with its retitled heading and the five new instance rows.
- The release-checklist question, sited next to the existing documentation-scope
  line.
- The retroactive re-check entry, with both withdrawals named and the
  incompleteness stated.
- `git diff --stat -- 'crates/**'` empty.

## 11. Acceptance criteria

RFC-067 §Acceptance criteria 1–5. The two that carry the task:

- **1** — the action-not-only-correction distinction must be explicit in the
  rule's own wording (§3). This is the criterion a vague implementation fails.
- **5** — the propagation evidence recorded *where the rule lives*, not only in
  the RFC.

## 12. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/067-withdrawing-a-claim-does-not-retract-it/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the retroactive re-check line (§6). It is the part a
consumer will actually act on, and the failure mode is writing it as a
description of what changed in 0.34.0 rather than as an instruction to someone
who copied a claim eleven releases ago and has not thought about it since.
