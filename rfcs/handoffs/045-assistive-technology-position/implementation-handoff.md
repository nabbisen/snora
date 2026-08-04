# Developer Handoff — RFC-045 assistive technology position

**Governing RFC.** [RFC-045](../../proposed/045-assistive-technology-position.md)
**Status.** Inherited from RFC-045 (Proposed; accepted by the owner).
**Release target.** 0.27.1 (patch). Ships alone and first.
**Implementation units.** One. Documentation and governance only — no code.

---

## 1. Task title

State snora's position on assistive technology, bound the ABDD claim where
it is made user-facing, and make the accessibility documentation reachable
by consumers.

## 2. Purpose

A downstream team preparing UX acceptance sessions asked whether snora
intends to adopt an accessibility API when iced exposes one, or considers
assistive technology out of scope. They were about to record *"focus
visibility could not be verified"* in acceptance evidence and would rather
cite a known position than a silence.

The gap itself is defensible — an accessibility tree needs iced to expose
one. What is not defensible is that snora's own framing invites a broader
reading than its implementation supports.

## 3. Background — read first

- `rfcs/proposed/045-assistive-technology-position.md` in full.
- `docs/src/contributing/semantic-accessibility.md` — the existing document,
  which is **good** and is not being replaced.
- `.git-exclude/tmp/apimokka-to-snora-feedback-2026-08-04.md` — the field
  report, for the downstream framing.

Conventions: English only. No code, so no `cargo` gates; do **not** run
workspace-wide `cargo fmt` regardless (~152 hunks of pre-existing drift).

## 4. Ship this text

The owner has accepted this position. Use it as written — do not
paraphrase, soften, or expand it:

> **snora will integrate an accessibility tree when iced exposes one.**
> Until then, ABDD means layout-direction correctness and visual
> accessibility — contrast, logical edges, non-colour status encoding — and
> snora states that boundary plainly rather than implying more.
>
> snora will **not** build a parallel accessibility abstraction of its own
> in the interim.

## 5. Change scope

| File | Purpose |
|---|---|
| `docs/src/contributing/semantic-accessibility.md` | the position, as a top-level section |
| `docs/src/contributing/design-decisions.md` | decision entry + reconsideration trigger |
| `README.md` | bound the claim (2 sites — §7) |
| `docs/src/getting-started/05-when-to-use.md` | bound the claim (§7) |
| new consumer-facing accessibility page under `docs/src/guides/` | discoverability (§8) |
| `docs/src/SUMMARY.md` | register the new page |
| `CHANGELOG.md` | `[Unreleased]` **Changed** |

## 6. Explicit non-change scope

Do **not**:

- Change any source file. This handoff carries no code.
- Weaken, hedge, or re-scope the existing visual-accessibility work. The
  contrast tests, the four presets, and the ABDD checklist are unchanged
  and are genuinely strong — this RFC bounds a *claim*, it does not retreat
  from a capability.
- Rename or redefine the term **ABDD**. It is bounded, not replaced.
  Renaming would invalidate every existing reference for no gain.
- Duplicate `semantic-accessibility.md`'s content into the new page.
  Duplication drifts; link instead.
- Add an accessibility abstraction, a stub, or a placeholder API.
- Touch `docs/src/introduction.md` — it carries no accessibility claim
  (verified: its only match is a navigation link).

## 7. Bounding the claim — the exact sites

Verified locations. **Confirm against the source yourself; line numbers
drift.**

| Site | Current text | Assessment |
|---|---|---|
| `README.md:28` | "an app that needs **accessibility correct from day one** — RTL layout, theme-aware colors, logical edges baked into the API" | **The main overclaim.** Unqualified "accessibility correct" reads as covering assistive technology. |
| `README.md:105` | "*Accessible by Default and by Design.* Layout is described in logical edges…" | Already self-qualifying — it says what it means. Lightest touch, possibly none. |
| `docs/src/getting-started/05-when-to-use.md:11` | "…want accessibility correct from day one" (wraps across lines) | Same phrase as README:28. |
| `05-when-to-use.md:19` | "**Apps with ABDD as a hard requirement.** RTL support, logical edge…" | Enumerates what ABDD means. Probably fine; judge it. |

**One clause each is enough.** State what ABDD covers *and* what it does
not: layout direction and visual accessibility, not assistive-technology
support.

This is **not a disclaimer campaign.** Do not add warnings throughout the
documentation. The goal is that the name is honest at the point a reader
meets it — nothing more. If you find yourself adding a fourth or fifth
caveat, stop; you have gone past the intent.

## 8. Discoverability — the finding behind it

A downstream team auditing accessibility, **looking specifically for the
focus-state limitation**, did not find it — although
`semantic-accessibility.md` documents it in a dedicated section opening
*"This is a hard constraint of the pinned iced version, not a design
choice."*

The content is not the problem. Its **location** is: both accessibility
documents live under `docs/src/contributing/`, which reads as "for people
changing snora" rather than "for people depending on it."

Add a consumer-facing accessibility page under `docs/src/guides/` that:

- says what snora does provide (layout direction, contrast-tested presets,
  non-colour status);
- says what it does not (assistive technology, focus rings on stock
  controls — with the iced constraint named);
- **links** the two `contributing/` documents rather than restating them.

Register it in `SUMMARY.md` so it appears in the book's navigation.

Test the outcome, not the edit: **could a reader looking for "does snora
support screen readers?" find the answer from the guides section without
opening `contributing/`?** If not, the page has not done its job.

## 9. Decision register entry

Add to `docs/src/contributing/design-decisions.md`, matching the existing
format (index row + section):

- **Status:** Accepted.
- **Reconsideration trigger:** *iced exposes an accessibility API.*

The trigger is the point of the entry. Without it this position is a
statement that quietly expires; with it, the project revisits deliberately.
This is how the register already handles focus trapping and anchored
popover.

## 10. Required tests

```bash
mdbook build docs
mdbook test docs
```

Both must exit 0. There is nothing else to test — that is inherent to a
governance change and should be stated plainly in the review request rather
than padded with irrelevant gates.

## 11. Acceptance criteria

RFC-045 §Acceptance criteria 1–5:

1. The position is stated in `semantic-accessibility.md` **verbatim** and
   recorded in `design-decisions.md` with the reconsideration trigger.
2. The claim is bounded at the sites in §7.
3. Accessibility documentation is reachable from the consumer-facing part
   of the book, without duplicated content.
4. No code, no API, no gate row changes.
5. `mdbook build docs` and `mdbook test docs` pass.

## 12. Prohibited shortcuts

- Do not paraphrase the position text.
- Do not turn bounding into hedging. "snora is not accessible" is as wrong
  as the current overclaim, in the other direction — the visual
  accessibility work is real, tested, and unusual for a framework this size.
- Do not solve discoverability by moving the `contributing/` documents.
  Contributors need them where they are; consumers need a route to them.

## 13. Required evidence

- Diffs of every changed file.
- The new page, in full.
- `mdbook build docs` / `mdbook test docs` output.
- Explicit confirmation that no source file changed.
- A short statement of how you judged §8's outcome test.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/045-assistive-technology-position/`. **State
the single entry-point path to hand to the reviewer** in the completion
summary.

**Requested review focus:** whether the bounded claim is honest without
being self-deprecating — and whether §8's outcome test actually passes for
a reader who does not already know where to look.
