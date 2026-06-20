# Recipes and dogfood process

A **recipe** is a reusable app-specific pattern built from Snora Design
primitives. Recipes live in `docs/src/design/recipes/`, not in production
code — they are copy-paste starting points, not stable APIs.

This page defines the recipe format, the initial recipe catalog, and the
dogfood validation requirement that guards promotion to a stable primitive.

---

## Why recipes instead of more primitives

Many useful UI patterns are too domain-specific, too opinion-laden, or too
early to commit to a stable API. Recipes provide the value without the
maintenance burden:

- No breaking-change risk (recipes are docs, not code).
- No scope creep (patterns stay in their lane until proven).
- Adoption evidence accumulates before any API decision.

When a recipe has been used in at least two independent applications and
passes the promotion checklist in `api-governance.md`, it may become an
experimental helper or stable primitive.

---

## Recipe format

Every recipe document must include all nine sections:

### 1. Purpose

One sentence. What the recipe produces and why you would use it.

### 2. When to use

Bullet list of suitable contexts.

### 3. When not to use

Bullet list of contexts where the recipe is a poor fit, and what to use
instead.

### 4. Data the app owns

Explicit list of state the application is responsible for (visibility,
selection, loading status, error state, etc.). Recipes must not smuggle
state into Snora Design.

### 5. Snora primitives used

Which `snora::design::*` functions and types the recipe uses (tokens,
button variants, card variants, style bridge, etc.).

### 6. Accessibility notes

How the recipe meets the requirements in
[Accessibility checklist](accessibility-checklist.md):

- Which controls are native iced widgets.
- Whether focus/keyboard behavior is inherited.
- Any known limitations.

### 7. Code example

A `rust,ignore` fence with a complete, realistic usage snippet. Must
compile against the current snora version when copied into a real project.

### 8. Customization points

Which token values or style bridge overrides apply if the default behavior
does not fit.

### 9. Promotion status

One of:

- **Recipe** — not yet promoted; use freely but do not depend on stability.
- **Candidate** — in active consideration for promotion; feedback welcome.
- **Promoted** — moved to a stable primitive (link to the API docs).

---

## Candidate recipes

The following recipes are planned for v0.22. None are written yet; they are
listed to signal intent and invite early feedback.

| Recipe | Status |
|---|---|
| Result card | Recipe |
| Recent search card | Recipe |
| Setup wizard step card | Recipe |
| Onboarding card | Recipe |
| Empty state | Recipe |
| Background task card | Recipe |
| Friendly error recovery notice | Recipe |

Adding a recipe does not require an RFC. Open a PR with the doc page and
the nine sections filled in. The PR reviewer checks:

- The recipe does not introduce a new stable API.
- The example compiles against the current snora version.
- The accessibility notes are complete.

---

## Dogfood requirement

Before any recipe or experimental helper is promoted to a stable primitive,
it must have been validated in at least one real local-first productivity
application that exercises:

- The `snora::design` token system with at least two presets.
- The primitives the recipe relies on.
- Keyboard and high-contrast behavior.

The dogfood report must use the template below and be filed as a GitHub
issue using the `downstream-feedback` template.

---

## Dogfood feedback template

File this as a GitHub issue after using Snora Design in a production or
serious hobby application.

```text
App:
Snora version:
Patterns used:

What was easy:

What required custom code (and why):

Where the API felt awkward:

Accessibility observations:

Feature requests:

Recipes used:

Should any recipe become a primitive? If so, which one and why:
```

---

## Recipe directory structure

```text
docs/src/design/recipes/
  README.md          ← index of all recipes
  result-card.md
  recent-search.md
  empty-state.md
  background-task.md
  error-recovery.md
  …
```

Each recipe is a single Markdown file following the nine-section format.
The `README.md` index lists all recipes, their promotion status, and the
version they were first published in.
