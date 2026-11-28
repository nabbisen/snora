# RFC-142 — A feature turned on elsewhere breaks an exhaustive `match` on `Icon`

**Status.** Accepted 2026-11-28 (owner), every open question ruled as suggested (see Rulings).
**Raised.** 2026-11-28, the architect, while checking the 0.67 audit's request D (item 5, the exhaustive enums;
`.git-exclude/reviewed/audit-0-67-d/review-result.md`). The owner asked for this RFC for 0.67 on 2026-11-28.
**Release target.** 0.67.0. **A breaking change**, small and stated.
**Priority.** A latent compile break that someone else's feature choice can trigger at any time.

---

## The finding

```rust
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}
```

- **`Icon` is exhaustive, and two of its variants exist only under a feature.**
- An application built without `svg-icons` may match it exhaustively:
  `match icon { Icon::Text(s) => …, Icon::Lucide(l) => … }`.
- **If any crate in its dependency graph turns on `snora/svg-icons`** (Cargo unifies features), `Icon::Svg` comes
  into existence, and **the application's match stops compiling** (E0004, a non-exhaustive pattern). Its authors
  changed nothing.
- It is RFC-139's class, in the type system: a feature that changes what *someone else's* build does. RFC-139 set the
  rule that a feature may only make an API available. Here the feature also makes existing code invalid.
- **The book invites the match.** `guides/icons.md` says: "When a feature is disabled the variant does not exist in
  the enum. No runtime 'unknown icon kind' branch is reachable."

## Proposal

**R-1. `#[non_exhaustive]` on `Icon`.** Code outside `snora-core` that matches it must have a `_` arm. That arm stays
correct whichever features the graph turns on.

**R-2. `snora-widgets` keeps its exhaustiveness by a test, not by the compiler.** Its two renderers (`icon_element`,
`icon_element_sized`) are now outside the defining crate, so they need a `_` arm too, and the compiler will no longer
fail them when a variant is added.
- **A test fails** if a variant renders through the `_` arm. Under `--all-features`, every variant is constructed and
  drawn, and the test asserts that none takes the fallback.
- **The `_` arm draws nothing, and is never reached in a build where the workspace is consistent.** Say so in a
  comment. It draws nothing rather than panicking, because an `Icon` that cannot be drawn must not take the window
  down.

**R-3. The book and the migration guide say it plainly.** The icons guide's sentence goes, and is replaced by: "match
with a `_` arm: a feature turned on by another crate adds a variant". The 0.66 → 0.67 guide names the break and the
one-line fix.

## Open questions, with suggested rulings

**Q-1. Make the variants always exist instead?** **Suggest: no.** `Svg(PathBuf)` could exist without its feature,
but what would it draw without iced's `svg`? A fallback that silently draws nothing is worse than a type that tells
you. `Lucide(lucide_icons::Icon)` cannot exist without the `lucide-icons` dependency at all. `#[non_exhaustive]` is
the language's own answer to "more variants may appear".

**Q-2. The other eleven exhaustive enums?** **Suggest: not here.** None has a feature-gated variant, so none has this
hazard. Whether they stay exhaustive before 1.0 is part of the owner's 1.0 review, and `api-freeze-review.md`
records the list.

**Q-3. When.** **Suggest: 0.67.0.** It is breaking, and 0.67 is a minor that may break, with its guide.

## Rulings, 2026-11-28 (owner)

"Yes." Every question as suggested:
- **Q-1:** `#[non_exhaustive]`, not variants that always exist.
- **Q-2:** the other eleven exhaustive enums are left to the owner's 1.0 review.
- **Q-3:** 0.67.0, breaking, named in the migration guide.

## Review against snora's principles

- **Safe and robust.** No feature choice elsewhere can break an application's build.
- **Nobody is confused.** The type now says what is true: more variants may appear. The book stops inviting the match
  that breaks.
- **Clean.** One attribute and one test. No new type.
- **Correspondence cost.** One line in 0.67's migration guide. A letter only to a team known to match `Icon`; no team
  is known to.

## Units

**Unit 0, measured (nothing committed):**
- A scratch crate that matches `Icon` exhaustively without `svg-icons`, shown to stop compiling when a second
  dependency enables `snora/svg-icons`. That is the defect, demonstrated.
- The same crate with R-1 and a `_` arm, compiling in both graphs.
- Every `match` on `Icon` in the workspace, listed.

**Unit 1:**
- R-1, R-2 and the test, with a breakage row: a variant routed to the `_` arm fails the test.
- CHANGELOG `[Unreleased]`: **Changed** (breaking), with the fix.
- **The book and the migration guide are the architect's** (R-3).

## Acceptance criteria

1. The defect demonstrated in a scratch crate, and gone with R-1.
2. Unit 1's test shown able to fail, and every `ci.yaml` command run locally.
3. The icons guide and the 0.66 → 0.67 guide corrected.
