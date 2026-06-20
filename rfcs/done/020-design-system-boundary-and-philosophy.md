# RFC 020 — Design System Boundary and Philosophy Amendment

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `README`, `docs/`, RFC governance text.

## Summary

This RFC amends Snora's philosophy to allow an optional design-system layer while preserving the original identity of Snora as a small layout and overlay framework.

The new boundary is:

> Snora positions and stacks. Optionally, Snora Design provides a small, coherent desktop application design system for local-first productivity tools. Applications still own content, domain behavior, complex widgets, workflow logic, and final brand identity.

## Motivation

The original Snora philosophy was intentionally narrow: Snora positions and stacks; applications supply content and styling.

That remains correct for the minimal path. However, real Snora-based iced applications face repeated design-system work. They either remain visually close to default iced, or they invest large effort building a custom accessible design system.

Snora Design fills the middle ground.

## Goals

- Introduce Snora Design as an optional design-system layer.
- Preserve minimal Snora usage.
- Preserve Snora's non-widget-framework identity.
- Define what is changed and what is preserved.
- Establish a recipe-first policy.
- Establish scope gates for future components.

## Non-goals

- No full component catalog.
- No form validation framework.
- No table/data grid.
- No charting framework.
- No router.
- No full localization framework.
- No replacement for iced.
- No guarantee that arbitrary app content is accessible.
- No forced design-system adoption.

## Boundary statement

The project documentation should include:

> Snora is not becoming a general widget component framework. Snora remains a small layout and overlay framework for iced-based desktop applications. Snora Design is an optional layer that provides design tokens, accessible visual defaults, iced-facing style helpers, and shallow productivity-oriented primitives. Applications still own their domain behavior, complex widgets, validation, data presentation, navigation, and final brand identity.

## Scope classification

| Candidate | Classification | Rationale |
|---|---|---|
| Palette tokens | In scope | Design foundation |
| Typography roles | In scope | Readability foundation |
| Focus tokens | In scope | Accessibility baseline |
| Button helper | In scope | Shallow generic primitive |
| Basic card helper | In scope | Shallow app-surface primitive |
| Notice primitive | Later | Useful but should follow foundation |
| Filter chip | Later | Needs semantic care |
| Progress primitive | Later | Needs semantic/progress behavior review |
| Result card | Recipe first | App/workflow-specific |
| Recent search card | Recipe first | App/workflow-specific |
| Setup wizard card | Recipe first | Workflow-specific |
| Form validation | Out of scope | App behavior |
| Table/data grid | Out of scope | Broad component framework |
| Chart | Out of scope | Specialized display |

## Process handling

Every future addition must answer:

> Is this a generic visual/accessibility primitive for local-first desktop productivity apps, or is it domain/application behavior?

If generic, it may be considered for Snora Design.

If domain-specific, it must remain a recipe or application code.

## Promotion lifecycle

```text
idea
  -> recipe
  -> experimental helper
  -> stable primitive
```

Promotion requires real use.

Minimum evidence:

- two real applications; or
- one strong dogfood app plus one external request.

## Compatibility

This RFC does not change public API by itself. It authorizes later RFCs to introduce optional features.

## Risks

### Risk: framework bloat

Mitigation:

- explicit non-goals;
- recipe-first policy;
- promotion criteria;
- scope gate.

### Risk: existing users feel Snora changed identity

Mitigation:

- minimal path remains supported;
- design features are optional;
- docs explain old vs new philosophy.

## Acceptance criteria

- Boundary statement is added to docs.
- Non-goals are updated.
- Future RFCs classify additions as token, style helper, primitive, recipe, or out of scope.
