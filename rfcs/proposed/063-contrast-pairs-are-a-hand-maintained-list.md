# RFC 063 — The contrast pair list is hand-maintained, so the nineteenth role repeats the defect

**Status.** Proposed
**Tracks.** Accessibility / measurement integrity. Reported by **tekstide**
(2026-08-18), who identified the class after RFC-058 fixed two instances.
**Touches.** `crates/snora-design/src/palette.rs`,
`crates/snora-design/src/tests.rs`,
`docs/src/contributing/accessibility-checklist.md`,
`docs/src/contributing/api-governance.md`, `CHANGELOG.md`.
**Release target.** 0.36.0.

## Summary

RFC-058 added `border` against three surfaces, `text_muted` against three, and
`text_secondary/surface_raised`. **Every one of those is a hand-written entry in
a hand-maintained list.**

tekstide's diagnosis, and it is correct:

> The list did not fail because it was short; it failed because **nothing about
> adding a role forces anyone to measure it.** The nineteenth role will be added
> by someone who has no reason to think about `mandatory_pairs` at all, and the
> failure mode reproduces exactly.

RFC-058 fixed two instances. This RFC closes the class.

## The defect exists at two levels, both verified

**1. `mandatory_pairs` is a literal list.** Adding a `Palette` field and never
mentioning it there compiles, passes, and ships — exactly how `border` shipped
at 1.28:1 for the role's entire life.

**2. `Palette::roles()` is the same defect one level down.** It is
`#[cfg(test)] pub(crate) fn roles(&self) -> [Color; 18]` returning eighteen
hand-written field accesses. A nineteenth field does not break it; the array
stays valid at eighteen and the new role is silently absent from every check
built on it.

## What the compiler already enforces, and what it does not

Measured by adding a nineteenth field to `Palette` and compiling
(probe run 2026-08-18, files restored, tree verified clean):

| Mechanism | Adding a role... |
|---|---|
| The four preset initializers | **fails to compile** — `E0063: missing field` ×4 |
| `Palette::roles()` | compiles; role silently omitted |
| `mandatory_pairs` | compiles; role silently unasserted |

So the compiler **already** forces an author to supply a *value* in all four
presets. What nothing forces is declaring **where the role renders** and
therefore **what it must be measured against**. That gap is the whole defect,
and it is narrower than "the list is unmaintained" — the value side is
mechanised, the assertion side is not.

## The mechanism, verified rather than assumed

Exhaustive struct destructuring inside the defining crate fails to compile when
a field is added, **despite `Palette` being `#[non_exhaustive]`** — that
attribute constrains other crates, not `snora-design` itself.

Probed by adding `probe_role` and compiling:

```text
error[E0027]: pattern does not mention field `probe_role`
  --> crates/snora-design/src/palette.rs:77:13
   | missing field `probe_role`
```

That is the enforcement point. A function that destructures `Palette`
exhaustively and returns each role's intended surfaces cannot be left stale: the
nineteenth field breaks the build until someone answers *where does this
render?* — the question nobody asked about `text_muted` for its entire life.

The pair list is then **derived** from that declaration rather than maintained
beside it.

## Not the cross-product

tekstide anticipated the wrong fix and they are right to:

> We are not suggesting the cross-product. `accent_text` on `background` is
> meaningless — `accent_text` exists to sit on `accent` — so 18 × 3 would be
> mostly noise, and **noise in an accessibility gate is how gates get ignored.**

The declaration is *intended usage*, not all usage. `accent_text` declares
`accent`; `text_primary` declares all three surfaces; `focus` declares the
surfaces a ring is drawn over. Each role also declares its threshold class —
body text at `AA_TEXT`, non-text boundary at `NON_TEXT_MIN` — so the derived
list carries the right bar automatically rather than by a second hand-written
decision.

## Scope

1. **A total, compiler-enforced declaration** of each `Palette` role's intended
   surfaces and threshold class, using exhaustive destructuring so a new field
   cannot be added without answering it.
2. **Derive `mandatory_pairs` from it.** The assertions become a consequence of
   the declaration, not a parallel list.
3. **Fix `Palette::roles()` the same way**, or delete it if the declaration
   subsumes it.
4. **Record the rule** in the accessibility checklist and in
   `api-governance.md` beside the additive-only covenant: adding a `Palette`
   role requires declaring its surfaces, and the compiler enforces it.

## Non-goals

- **No new or renamed `Palette` roles.** RFC-036's covenant forbids it without
  reopening D-3/D-4, and this RFC does not need one.
- **No preset value changes.** If deriving the list surfaces a *new* failing
  pair, that is a finding to **report before repairing** — RFC-058's carve-out
  and its failing-first order would apply, and this RFC is not the place to
  exercise them silently.
- **No cross-product.**
- **No change to `AA_TEXT`, `FOCUS_MIN` or `NON_TEXT_MIN`.**
- **No public API change.** `roles()` is already `pub(crate)` and `#[cfg(test)]`;
  the declaration should be too.

## Open questions

**Q-1 — does deriving the list change what is asserted today?** It should
produce a superset: the twelve original pairs, RFC-058's seven additions, and
any pair implied by a declaration nobody had written an assertion for.
**Enumerate the difference before landing it.** Any newly-covered pair that
*fails* is a defect this RFC found, and must be reported rather than repaired in
passing.

**Q-2 — where does the declaration live?** `palette.rs` keeps it beside the
fields it constrains, which is where a contributor adding a field will be. A
separate module is tidier but reintroduces the distance that caused the problem.
Suggest `palette.rs`.

**Q-3 — should the threshold class be part of the declaration or inferred from
the role name?** Inference (`*_text` → `AA_TEXT`) is concise and brittle;
`focus` and `border` are both non-text and share no naming convention. Suggest
explicit, on the grounds that this RFC exists because something was implicit.

## Acceptance criteria

1. Adding a field to `Palette` **fails to compile** until its surfaces and
   threshold class are declared — demonstrated by a probe, added and reverted,
   with the compiler error captured.
2. `mandatory_pairs` is derived from the declaration; no hand-written pair list
   remains.
3. `roles()` is derived or removed.
4. Q-1's before/after pair set is enumerated, and any newly-failing pair is
   **reported, not repaired**.
5. The rule is recorded in the checklist and `api-governance.md`.
6. `cargo test -p snora-design` passes; `render_semantics` unmodified.

## Compatibility and security

**Compatibility.** No public API change and no preset value change. The derived
list may assert more pairs than today; if all pass, nothing observable changes.

**Security.** None.

## Credit

tekstide identified the class, declined to propose the cross-product, and
disclosed the same defect in their own theme tests — *"complete by coincidence
of our being small… we were not more careful than you; we were luckier."*
