# Developer Handoff — RFC-082 three statements a reader cannot trust

**Governing RFC.** [RFC-082](../../accepted/082-three-keyboard-and-focus-statements-a-reader-cannot-trust.md)
**Status.** Inherited from RFC-082 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.2 — documentation only. **No code.**
**Implementation units.** Two now, **one held.**

---

## 1. Held — §3 (menus) is NOT in this handoff

The owner ruled in-menu traversal is the application's own. **Recording that
ruling is blocked on a scope decision that has not been made**, and the RFC
states why: `TabBar` carries `active: TabId` so an application can drive tab
selection; `Menu` and `MenuItem` carry **no field** for a highlighted item, so
an application can track arrow keys and has no way to make snora render it.

As written, "the application's own" would read as available and be impossible.

**Do not write anything into `menus.md`.** Not the ruling, not a caveat, not a
"deferred" note. Three options are with the owner and one of them is a breaking
minor. **If you have an opinion, send it to the architect; do not implement it.**

## 2. Unit 1 — the decision register is wrong about who is asking

`docs/src/contributing/design-decisions.md:32`:

> Focus trapping deferred | Deferred — trigger fired, Q-1 (RFC-060) now the
> blocker | **Concrete app: met (tekstide)**

**tekstide withdrew as a demand signal on 2026-08-18** — *"we would not switch
to trapping even if you shipped it"* — and our own 0.36.1 note recorded the
consequence: *"the demand column for modal focus trapping is empty."*

Correct the row so it reflects that **no consumer is a demand signal**, citing
tekstide's withdrawal.

**Two things it must not do.** It must **not** claim to resolve RFC-060 Q-1 —
the owner ruled that is answered by measurement (RFC-078), which is open and
unstarted. And it must **not** silently drop the row; the decision is still
"deferred", only the evidence changed.

**Then check the rest of the register the same way.** RFC-075 fixed one register
being wrong about itself and never looked at this one. **Read every row's
evidence column and say which ones you verified**, including the ones that were
already right.

## 3. Unit 2 — a mandatory bullet that cannot pass

`docs/src/design/high-contrast.md:59-61`:

> - The focus ring at `3.0` width must be visible when tabbing through controls.
>   (In iced 0.14 the focus ring is not rendered through `button::Style` — see
>   Semantic accessibility.)

**A mandatory checklist item whose own parenthetical says it cannot be
satisfied.** Every other statement of that limitation in our docs carries a
marker — `BLOCKED`, `INFO`, or a documented exception.

Either mark it the way the others are, or reword it so it can be checked as
written. **Match the existing convention rather than inventing a fourth
marker** — find how the others are marked first, and say which you followed.

## 4. Unit 1's second half — Q-2 ruled: date the evidence

The register's demand column depends on what consumers say, which **no check can
derive** — do not try to mechanise it.

**Give each row's evidence the date it was last confirmed.** Staleness then
shows without pretending it is derivable, which is the honest version of the
guarantee RFC-079's check gives for guides.

## 5. Explicit non-change scope

- **Nothing in `menus.md`** (§1).
- **No resolution of RFC-060 Q-1**, and no touching RFC-078.
- **No new marker vocabulary** (§3).
- **No code.** `git diff -- crates/` must be empty.

## 6. Required evidence

- The corrected register row, quoted before and after
- **Every register row's evidence column, listed, with whether you verified it**
  — including rows that needed nothing
- The `high-contrast.md` bullet before and after, plus the convention you matched
  and where you found it
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py`
- `git diff --stat -- crates/` — **expected empty**

## 7. Acceptance criteria

1. The focus-trapping row says no consumer is a demand signal, cites tekstide's
   withdrawal, keeps the decision as deferred, and does not resolve Q-1.
2. Every register row carries the date its evidence was last confirmed.
3. Every row's evidence reported as verified or not — a null result stated, not
   omitted.
4. The `high-contrast.md` bullet is marked or reworded, matching an existing
   convention you name.
5. **`menus.md` is untouched.**
6. `CHANGELOG.md` `[Unreleased]`, crediting tekstide.

## 8. Required review-request format

`.git-exclude/review-request/082-three-keyboard-and-focus-statements-a-reader-cannot-trust/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: §2's second half** — the rest of the register. One row
was wrong because a consumer told us. The others have had nobody checking them
at all.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
