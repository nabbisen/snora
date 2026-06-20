# API governance — Snora Design

This page defines how Snora Design APIs move between states — from recipe to
stable primitive to deprecated to removed — and the governance process that
keeps the design system from becoming a broad component catalog.

---

## API states

Every Snora Design API is in exactly one state at any time:

| State | Meaning |
|---|---|
| **Recipe** | A copy-paste doc pattern in `docs/src/design/recipes/`. Not a public API; no stability guarantee. |
| **Experimental helper** | A public function in `snora::design::*` marked `#[doc(alias = "experimental")]` or noted as experimental in its doc comment. Breaking changes allowed with a migration note. |
| **Stable primitive** | A public function in `snora::design::*` covered by snora's versioning policy (no breaking change without a minor bump + migration guide). |
| **Deprecated** | Marked `#[deprecated]` with a migration note. Present for at least one minor release before removal. |
| **Removed** | Gone. The removal is in the CHANGELOG and any migration guide for the corresponding minor release. |

---

## Promotion criteria

A recipe or experimental helper may become a **stable primitive** only when
all of the following are true:

1. **Evidence of use.** Used in at least two real applications *or* one strong
   dogfood app plus one documented external request.
2. **App-agnostic boundary.** The behavior the helper encodes belongs in the
   framework, not in the calling application. Domain-specific behavior stays
   as a recipe.
3. **Accessibility review complete.** The five RFC-027 primitive questions are
   answered in the promotion PR:
   - What native iced primitive is used?
   - Is it keyboard reachable?
   - How is focus visible?
   - What semantic limitation remains?
   - What example demonstrates usage?
4. **High-contrast behavior documented.** The primitive's visual-fit checklist
   is complete for all four built-in token presets.
5. **API is small enough to maintain.** If adding the primitive requires more
   than ~100 lines of non-test, non-doc code, revisit the scope boundary.
6. **Reason a recipe cannot serve the need.** If a recipe with the nine-section
   format covers the use case adequately, prefer the recipe.

---

## Stable API review checklist

Complete this checklist in the PR that promotes an API to stable:

```text
[ ] Is the function/type name final and unambiguous?
[ ] Does the data model avoid encoding app-specific state?
[ ] Does `snora --no-default-features` still compile? (minimal path)
[ ] Does `snora --no-default-features --features widgets` still compile?
[ ] Does adding this API require any new external dependencies?
[ ] Does the public API expose iced-specific types unnecessarily?
[ ] Are accessibility scope and limitations documented in the API docs?
[ ] Is there a migration note if this replaces a recipe or changes an
    experimental API?
[ ] Is there a reason this cannot remain a recipe or experimental helper?
[ ] Have the five RFC-027 primitive questions been answered?
[ ] Has the high-contrast visual-fit been verified in the workbench?
[ ] Is this covered by a test (contrast, compile-time, or render-semantics)?
```

---

## Deprecation policy

Snora is pre-1.0 on the design track: breaking changes may occur in minor
releases, but must be intentional, announced, and bridged where practical.

Recommended deprecation process:

1. Add `#[deprecated(since = "0.X.0", note = "Use Y instead.")]`.
2. Add a migration note in `CHANGELOG.md` and the relevant guide page.
3. Keep the deprecated item for **one or two minor releases** if the
   migration cost is low. Remove earlier only if the item is harmful to
   keep (e.g. incorrect accessibility behavior).
4. Remove in a minor release; record in the release's "Removed APIs"
   section.

Do not deprecate an API that has no stable replacement yet. Mark it
experimental instead and keep it until the replacement is ready.

---

## Release review section

Include this section in the CHANGELOG entry for every design-track minor
release (v0.20, v0.21, …):

```text
### Design API changes

New APIs:        (list new stable primitives, helpers, or style bridge additions)
Experimental:    (list items introduced as experimental)
Promoted:        (list items promoted from recipe/experimental to stable)
Deprecated:      (list items newly marked deprecated, with migration target)
Removed:         (list items removed, with version they were deprecated in)
Recipes added:   (list new recipe doc pages)
Recipes promoted:(list recipes that moved to experimental/stable)
Scope concerns:  (note any out-of-scope requests received and disposition)
```

For releases with no design API changes (e.g. a patch release), omit this
section from the CHANGELOG.

---

## Future 1.0 design-system gates

The design-system track has its own 1.0 gates, separate from snora core's
ten 1.0 gates. Before declaring Snora Design APIs stable at a 1.0 level:

| Gate | Condition |
|---|---|
| D-1 | One iced major upgrade survived with the design feature enabled |
| D-2 | Minimal path clean (`--no-default-features`) after iced upgrade |
| D-3 | Token model stable for two consecutive minor releases without vocabulary churn |
| D-4 | Style bridge stable for two consecutive minor releases |
| D-5 | At least one real application in serious production use of the design tokens |
| D-6 | Promotion process used at least once (recipe → primitive) with evidence |
| D-7 | No broad component catalog creep (scope review complete) |
| D-8 | `snora-design` published (`publish = false` flipped; binary-size/build-cost measured) |

Gates D-1 and D-2 are coupled to snora core's Gate 1 (iced major upgrade).
Gates D-3 and D-4 can only be satisfied after the v0.20 release and at least
two subsequent minors. Gate D-5 couples to snora core's Gate 3.

These gates are tracked separately from `api-freeze-review.md`, which
covers the core snora 1.0 gates.

---

## Scope boundary

Snora Design is **not** a generic UI component library. The scope boundary
from RFC-020 is permanent:

- Helpers ship only if they are **layout-direction-aware** and
  **semantics-light** (they do not encode app logic).
- Forms, data grids, charts, routing, workflow engines, and
  domain-specific cards are outside scope forever.
- When in doubt, write a recipe first.

See [Feedback and scope](feedback-and-scope.md) for the full reasoning.
