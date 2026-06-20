# RFC 027 — Accessibility and Semantic Construction Policy

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** accessibility docs, primitive review policy.

## Summary

This RFC defines accessibility expectations for Snora Design primitives, examples, and documentation.

## Goals

- Scope accessibility claims honestly.
- Require high-contrast support.
- Require visible focus where iced exposes focus styling.
- Require semantic construction review.
- Require keyboard guidance.
- Document limitations.

## Non-goals

- No guarantee arbitrary app content is accessible.
- No complete screen-reader abstraction.
- No OS accessibility setting synchronization in v0.20.
- No custom accessibility tree framework.

## Allowed claim

> Snora Design provides accessibility-oriented defaults and ABDD layout discipline.

## Disallowed claim

> Apps using Snora are automatically accessible.

## Checklist

Required checklist sections:

- contrast;
- high contrast;
- focus visibility;
- keyboard reachability;
- semantic construction;
- pointer target size;
- typography line-height/readability;
- RTL/LTR;
- reduced motion consideration;
- disabled state readability;
- loading/empty/error clarity;
- destructive action wording;
- non-technical wording.

## Semantic construction rule

Prefer native iced interactive widgets.

Avoid visual-only controls built from generic containers plus mouse handlers.

## Focus visibility (scoped)

Visible focus is required **only where the pinned iced widget APIs expose
enough state to style it**. In iced 0.14, standard `button` and `container`
do not expose focus state (see RFC-025), so their helpers cannot render a
custom focus ring. Where focus cannot be expressed, the limitation must be
documented and must **not** be treated as a QA regression. This replaces any
blanket "visible focus required" reading.

## Primitive rules

### Buttons

Use iced button.

### Interactive cards

Use semantic control where possible or document limitation.

### Notices

Dismiss/action controls must be real controls.

### Chips

Interactive chips should be button-like where possible.

### Progress

Use iced progress primitive where possible and document limitations.

## Keyboard ownership

| Behavior | Owner |
|---|---|
| Basic iced button activation | iced |
| Snora visual focus style | Snora where possible |
| App shortcuts | application |
| Escape handling | application or later RFC |
| Focus trap | out of v0.20 unless separately designed |

## Data lifecycle

```text
token design
  -> contrast tests
  -> style helper design
  -> primitive semantic review
  -> example visual-fit review
  -> docs checklist
  -> dogfood feedback
```

## Process handling

Every primitive RFC must answer:

1. What native iced primitive is used?
2. Is it keyboard reachable?
3. How is focus visible?
4. What semantic limitation remains?
5. What example demonstrates usage?

## Acceptance criteria

- Accessibility checklist exists.
- Semantic construction policy exists.
- Primitive RFCs use the policy.
- Docs avoid false guarantees.
