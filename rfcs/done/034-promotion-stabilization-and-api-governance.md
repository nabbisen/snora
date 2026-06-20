# RFC 034 — Promotion, Stabilization, and API Governance

**Status.** Implemented (v0.23.0)
**Tracks.** Snora Design System migration; v0.23+.
**Touches.** release governance, API lifecycle docs.

## Summary

This RFC defines how Snora Design APIs become stable or are removed.

## Motivation

Design systems tend to grow. Snora Design needs governance to avoid becoming a broad component catalog.

## API states

```text
recipe
experimental helper
stable primitive
deprecated
removed
```

## Promotion criteria

A recipe or experimental helper may become stable only if:

1. used in at least two real applications; or
2. used in one strong dogfood app plus one external request; and
3. behavior boundary is app-agnostic; and
4. accessibility/semantic review is complete; and
5. high-contrast behavior is documented; and
6. API is small enough to maintain.

## Stable API review checklist

- Is the name final?
- Is the data model app-agnostic?
- Does it preserve minimal path?
- Does it require new dependencies?
- Does it expose iced-specific details unnecessarily?
- Is accessibility scope documented?
- Is migration documented?
- Is there a reason it cannot remain a recipe?

## Deprecation policy

Pre-1.0 breaking changes may occur in minor releases, but must be intentional.

Recommended process:

1. Mark deprecated where practical.
2. Provide migration note.
3. Keep bridge for one or two minor releases if cheap.
4. Remove when stable alternative exists.

## Release review section

Each release should include:

```text
New APIs:
Experimental APIs:
Promoted APIs:
Deprecated APIs:
Removed APIs:
Recipes added:
Recipes promoted:
Scope concerns:
```

## Future 1.0 design-system gates

Before declaring design API stable:

- one iced major upgrade survived;
- minimal path still clean;
- token model stable for two minor releases;
- style bridge stable for two minor releases;
- at least one real app in serious use;
- promotion process proven;
- no broad component bloat.

## Acceptance criteria

- API states documented.
- Promotion criteria documented.
- Release review checklist exists.
- Future 1.0 gates updated.
