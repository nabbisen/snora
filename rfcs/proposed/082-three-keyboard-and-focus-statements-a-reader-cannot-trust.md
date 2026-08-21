# RFC 082 — Three keyboard-and-focus statements a reader cannot trust: one stale, one uncheckable, one absent

**Status.** Proposed
**Tracks.** Decision register / accessibility documentation.
**Found by** **tekstide**, 2026-08-20. All three verified against source.
**Touches.** `docs/src/contributing/design-decisions.md`,
`docs/src/design/high-contrast.md`, `docs/src/guides/menus.md`.
**Release target.** 0.39.2 — documentation only.

## 1. Stale — the decision register says tekstide is a live trigger, and tekstide says it is not

`design-decisions.md:32`:

> Focus trapping deferred | Deferred — trigger fired, Q-1 (RFC-060) now the
> blocker | **Concrete app: met (tekstide)**

**tekstide has explicitly withdrawn as a demand signal**, on 2026-08-18:
*"keystroke suppression is stronger than trapping for our threat model and we
would not switch to trapping even if you shipped it."* Our own 0.36.1 note
recorded the consequence — *"the demand column for modal focus trapping is
empty"* — alongside arama out and apimokka declined.

**So the answer exists and the register is behind it.** A reader of 0.39.1
concludes tekstide is a live concrete-app trigger. They are not one, and they
are the ones who had to tell us.

This is RFC-075's defect — a register wrong about its own subject — in the
register RFC-075 did not look at. Note what tekstide is careful about and we
should be too: **this does not resolve RFC-060 Q-1**, which the owner has ruled
is answered by measurement (RFC-078). It corrects who is listed as asking.

## 2. Uncheckable — a mandatory checklist item that cannot pass on the default path

`high-contrast.md:59-61`:

> - The focus ring at `3.0` width must be visible when tabbing through controls.
>   (In iced 0.14 the focus ring is not rendered through `button::Style` — see
>   Semantic accessibility.)

**The bullet is mandatory and its own parenthetical says it cannot be satisfied.**
Every other statement of that limitation in our docs carries a marker —
`BLOCKED`, `INFO`, or a documented exception. This one reads as a requirement a
reviewer is supposed to tick.

## 3. Absent — in-menu keyboard navigation is neither promised nor denied

`guides/menus.md` contains **zero** occurrences of "arrow", "keyboard" or "Tab"
— verified. It describes header and context menus entirely in pointer terms.

Arrow-key traversal *within* an open menu is a different axis from both concerns
we have scoped: `next_zone` moves **between** skeleton regions,
`dismiss_on_escape` **closes**. Neither moves between items in an open menu,
which is the conventional desktop pattern and the first thing a keyboard-only
user reaches for.

tekstide's framing: items may already be Tab-reachable individually if they are
native controls, **they could not tell from the docs, and that ambiguity is the
finding.** They are explicit that this is *not* a request to build it.

Our own "two keyboard concerns, two owners" framing reads like it wants a third
entry — **owned, or deliberately deferred and said so.**

## Open questions

**Q-1 — what does §3 become?** Three options: state that in-menu traversal is the
application's own (consistent with "intra-pane structure snora does not own");
state it is deferred with a trigger; or scope it. **Suggest the first** — it
matches where we already drew the line for tabs and breadcrumbs in RFC-060 — but
this is a scope decision and belongs to the owner, not to this RFC.

**Q-2 — should §1's correction be mechanised?** The register's demand column
depends on what consumers say, which no check can derive. **Suggest not.** State
the date each row's evidence was last confirmed, so staleness is visible without
pretending it is derivable.

## Acceptance criteria

1. `design-decisions.md`'s focus-trapping row reflects that **no consumer is a
   demand signal**, citing tekstide's withdrawal — and does **not** claim to
   resolve RFC-060 Q-1.
2. The `high-contrast.md` bullet is marked the way every other statement of that
   limitation is, or reworded so it can be checked as written.
3. Q-1 ruled by the owner and `menus.md` says the answer, whichever it is.
4. Q-2: each register row carries the date its evidence was last confirmed.
5. No code.

## Compatibility and security

**Compatibility.** Documentation only. **Security.** None.
