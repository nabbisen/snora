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

## Additive-only covenant (design surface)

D-3 and D-4 closed at v0.25 on a freeze review spanning six consecutive
minors (v0.20 → v0.25; see RFC-036). Closing a stability gate immediately
before a milestone that extends the surface it covers would hollow the gate
out, so this covenant binds what may be done to the frozen surface *next*.
It does not freeze the surface against all future change — it defines
which changes remain permitted without reopening the gate.

### The frozen surface

Anything not listed below is outside the covenant and not constrained by
it.

**Token surface** — all public items of `snora-design`:

- `Tokens` and its constructors: `light`, `dark`, `high_contrast_light`,
  `high_contrast_dark`.
- `Palette` and its 18 role fields.
- `Color`.
- `Spacing`.
- `Typography` and `TextRole`.
- `Radius`.
- `FocusTokens`.
- `Tone`, `Emphasis`, `Size`, `Density`.
- The `contrast` module's three functions: `relative_luminance`,
  `contrast_ratio`, `composite_over`.

**Style-bridge surface** — all public functions of `snora_style`
(RFC-055; formerly `snora_widgets::design::style`, removed in RFC-056):

- `color::to_iced_color`.
- `button::{primary, secondary, ghost, danger}`.
- `container::{card_surface, card_raised, card_selected}`.
- `progress::toned`.
- `text::{body_size, body_small_size, label_size, title_size,
  heading_size, display_size}`.

**Not frozen.** The design **primitives** — `button`, `card`, `notice`,
`chip`, `progress` helper modules — are deliberately excluded from the
frozen surface. They remain governed by the promotion lifecycle above
(Recipe → Experimental helper → Stable primitive → Deprecated → Removed),
which is a different and more permissive regime than this covenant.

### Permitted without reopening the gate

- Adding new items to `snora-design` (new token types, new fields on
  `#[non_exhaustive]` types, new presets, new contrast helpers).
- Adding new functions to the style bridge.
- Adding new modules, entry points, and primitives elsewhere in the design
  layer.
- Changing values *inside* a preset **only** where a contrast test proves
  the change fixes an accessibility defect, recorded as **Fixed** in the
  CHANGELOG.

**Contrast thresholds are floors, and that is a commitment, not
currently-true trivia (RFC-072).** Every contrast assertion snora ships
is `>=` — a role's ratio against its declared surfaces is guaranteed to
be *at least* its threshold. **No maximum is guaranteed, now or
later.** The bullet above is the reason: the only preset value change
this covenant permits is one a contrast test proves fixes a defect —
raising a ratio that was failing — so the only direction a value can
move under this covenant is up. **We will not commit to keeping any
snora colour insufficiently contrasty**, because a design system that
promises accessibility repairs cannot also promise a ceiling those
repairs might cross.

**The practical consequence: do not assert that a snora colour stays
below a threshold.** A repair can raise a ratio you were relying on
staying low, at any time. What it carries when it does: **at minimum, a
CHANGELOG entry under Fixed** — every permitted preset-value change
requires one, per the bullet above — **and where the change is
visible, an explicit appearance-change statement and a migration
guide**, as 0.34.0's `border` repair carried — its CHANGELOG entry
states plainly *"This is an appearance change, not a silent fix"*
(`CHANGELOG.md`), and [the paired migration
guide](../guides/migration-0.33-to-0.34.md) names the affected
rendering regions and tells readers what to re-check — that is the
only precedent of this kind, and it is the bar future repairs are held
to, not a best-effort courtesy. None of that makes it
a breaking change by this covenant's own definition, because nothing
frozen changed shape or meaning — but it is not a silent one either. If
a decision in your own application depends on a colour being illegible
against something, assert that against **your own** colour, which you
control, not against ours.

**And the limit of this guarantee, stated as plainly as the guarantee
itself: a repair is judged only on the pair that was failing, and
preserves nothing else.** Changing `border`, for instance, moves its
contrast against `background`, `surface`, and `surface_raised`
simultaneously — the repair is correct if the failing pair now clears
its floor; nothing about the covenant commits to any other pair's
ratio staying where it was, moving by a bounded amount, or moving in a
particular direction. The floor is the promise. No other ratio is.

**Adding a `Palette` role requires declaring where it renders, and the
compiler enforces it (RFC-063).** `Palette::usages`
(`crates/snora-design/src/palette.rs`) destructures `Palette`
exhaustively and declares each role's intended surfaces and threshold
class; `mandatory_pairs` in `tests.rs` is derived from that declaration,
not maintained as a separate list. A role added to `Palette` without a
matching entry in `usages` fails to compile (`E0027: pattern does not
mention field ...`) — this is what closed the class of defect RFC-058
found twice (`border`, `text_muted`: values existed, contrast was never
asserted, because nothing forced anyone to add them to the old
hand-written pair list). The declaration is crate-private and
`#[cfg(test)]`, matching this covenant's own framing that role additions
are otherwise unconstrained — the enforcement is about *measuring* a new
role, not about permission to add one.

**The enforcement is crate-local by construction, and consumers do not need it.** `#[non_exhaustive]` permits exhaustive destructuring only *inside* the crate that defines the type. From a consuming crate the compiler requires `..` in the pattern (`E0638: `..` required with struct marked as non-exhaustive`) — and `..` is exactly what defeats the mechanism. A downstream team hit this trying to copy the pattern onto snora's `Palette` and reported that `E0638` alone reads like a mistake rather than a boundary (orbok, 2026-08-18).

This is not a gap to close. A consumer maintaining a pair list over **snora's** roles is duplicating a check snora already runs, and [`design/stability.md`](../design/stability.md) states the token surface is contractually frozen. The mechanism *is* portable — to a consumer's **own** palette type, in their own crate, where the same `E0027` enforcement applies.

**Adding a composited or derived surface carries the same obligation,
beside the role rule above (RFC-065).** `Palette::usages` can only
declare where a *role* renders; it has no way to express a surface that
is not a `Palette` field at all — the modal dim, composited at render
time from `background`'s own darkness and an alpha constant, is such a
surface, and RFC-063's mechanism could not see it. RFC-065 measured it
at 2.85:1 in `light` against SC 1.4.11's 3:1, unmeasured for as long as
the surface existed. The rule this establishes: a new composited or
derived surface must be expressed as a pure function over `Tokens` in
`snora-design` — not reimplemented at the render site that consumes it
— and asserted in `snora-design`'s own test suite
(`crates/snora-design/src/surfaces.rs`, `crates/snora-design/src/tests.rs`),
the same one contrast suite the role rule keeps single. The role axis
and the surface axis are declared independently; a primitive can
violate either one without violating the other, so neither compiler
enforcement nor a written rule for one substitutes for the other.

### Forbidden without reopening the gate

- Removing, renaming, or retyping any item in the frozen surface.
- Changing the signature of any frozen style-bridge function.
- Adding, removing, renaming, or retyping a `Palette` role.
- Changing the *meaning* of an existing token (e.g. redefining what
  `Spacing::md` denotes) even where the type itself is untouched.

### Reopening obligation

If work requires a forbidden change, the implementing RFC must say so
explicitly and **reset D-3 and D-4 to open (⬜) in `api-freeze-review.md`
in the same change**. It must not proceed and rationalise afterward. The
gate's value is entirely in its being expensive to reverse — quietly
absorbing a forbidden change without reopening the gate defeats the point
of having recorded the freeze review at all.

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
