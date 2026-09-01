# Developer Handoff — RFC-072 contrast values are floors

**Governing RFC.** **RFC-072** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-072 — Accepted (owner, 2026-08-19).
**Release target.** 0.38.1 — **patch, documentation only.** No code, no
assertion, no value.
**Implementation units.** One. Small. **Resist growing it.**
**Sequence:** after RFC-071, which touches the same subject matter from the
assertion side. Either order works, but 071 first keeps the figures settled.

---

## 1. Task title

State, where a consumer will read it, that snora's contrast thresholds are
floors and that the only permitted value change raises a failing ratio.

## 2. Purpose

Every contrast assertion in the suite is `>=`. **Verify this yourself before
writing it** — it is the entire claim:

```
grep -rn "assert" crates/snora-design/src/tests.rs crates/snora/src/design/render/tests.rs | grep -E "contrast|ratio|>=|<="
```

Expect two forms and only two: `r >= min` (`snora-design/src/tests.rs:107`, the
derived mandatory pairs) and `r >= 7.0` (`:302`, primary text at AAA).
`AA_TEXT`, `FOCUS_MIN`, `NON_TEXT_MIN` are all minimums. **If you find any
upper-bound assertion, stop — the RFC is wrong and this work does not proceed.**

Neither this nor the covenant's direction has ever been told to consumers:
`ThresholdClass` is `pub(crate)`, and `api-governance.md` states what may change
without stating which way it moves.

**knotra asserts `border` against `surface` stays *below* 4.5**, to justify
excluding a neutral notice tone. Their reasoning is sound and their figure
(3.50) is right; the bound is simply not one we hold. It has already moved once —
0.34.0 took `dark` from 1.32 to 3.50. Nothing is broken today, and **they are
not to be contacted about it**; this RFC is the response.

## 3. The three questions are decided

**Q-1 — one canonical text, one pointer.** Full statement in
`docs/src/contributing/api-governance.md`, beside the covenant's "Permitted
without reopening the gate" list. A **short pointer** from
`docs/src/guides/accessibility.md`, which is where a consumer writing a contrast
assertion actually looks. **Do not write the statement twice** — two copies of
one fact drifting apart is a defect this project shipped twice this month.

**Q-2 — phrase it as a commitment, not an absence.** *"We will not promise a
ceiling"* closes the question; *"no ceiling is currently specified"* reads as an
oversight a future release might correct, and invites exactly the assertion
this RFC exists to prevent.

**Q-3 — yes, add the instance row.** `feature-gating-criteria.md`'s table, under
the **standing answer, undiscoverable** case — the same case as tekstide's
RFC-059 (2). Three columns; the third names what carried the gap. A case with no
instance logged reads as theoretical, and this one has a real consumer behind it.

## 4. What the text must contain

Three things, and no more:

1. **Thresholds are floors.** A role's ratio against its declared surfaces is at
   least its threshold. **No maximum is guaranteed, now or later.**
2. **The only permitted value change raises a failing ratio** (RFC-036), so the
   direction a value can move is up — and **we will not commit to keeping any
   colour insufficiently contrasty**, because a design system cannot promise
   that.
3. **The consequence a consumer needs:** *do not assert that a snora colour
   stays below a threshold.* If a decision depends on a colour being illegible,
   assert it against your own colour, which you control.

**And the limit of the guarantee, stated honestly.** A repair is judged on the
*failing pair* and preserves nothing else: changing `border` moves it against
`background`, `surface` and `surface_raised` simultaneously. **The floor is the
promise; no other ratio is.** Leave this out and we have replaced one
over-broad reading with another.

## 5. Explicit non-change scope

- **No code. No assertion. No value.** If this touches `crates/`, it is wrong.
- **Do not add an upper-bound assertion anywhere.** The RFC's point is that we
  decline to have one; adding one to be helpful inverts it.
- **Do not write to knotra or any other team.** The book is the delivery
  mechanism, deliberately — see the RFC's "Why documentation and not a letter."
- **Do not extend this to non-contrast values** (spacing, radius, sizes). The
  claim is verified for contrast only; asserting it more broadly would be the
  over-scoping RFC-059 is about.
- **Do not restate the covenant** in the accessibility guide. Pointer only.

## 6. Required evidence

- The §2 grep output, showing the two assertion forms and no upper bound
- `mdbook build docs && mdbook test docs`
- `git diff --stat -- crates/` — **expected empty**

## 7. Acceptance criteria

1. Both facts in `api-governance.md`, with the honest limit from §4.
2. A pointer — not a copy — in `guides/accessibility.md`.
3. The "assert against your own colour" consequence stated where a consumer
   reads it.
4. The claim re-derived by §2's grep, not inherited from the RFC.
5. Instance row added under **standing answer, undiscoverable**.
6. `git diff -- crates/` empty; `CHANGELOG.md` `[Unreleased]` records it.

## 8. Required review-request format

`.git-exclude/review-request/072-contrast-values-are-bounded-below-and-never-above/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus:** the wording of the honest limit (§4, last
paragraph). Everything else is placement. That sentence is the one that can be
got wrong in a way that creates the next over-broad claim.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
