# RFC-016-C — Downstream Adoption and Feedback Program

Status: Proposed  
Target release: v0.16+ and pre-1.0  
Priority: Medium  
Type: Product maturity / evidence gathering

## 1. Summary

Create a structured way to gather evidence from real Snora applications before 1.0, without allowing every app request to expand framework scope.

## 2. Motivation

One existing 1.0 gate is real application use outside examples and maintainer-only demos. That gate needs a process: what feedback to collect, how to evaluate feature requests, and how to avoid scope creep. Snora should learn from downstream apps while preserving its narrow identity.

## 3. Goals

- Define what counts as meaningful downstream adoption.
- Collect feedback in a structured template.
- Separate framework-level needs from app-specific wishes.
- Use adoption evidence to validate 1.0 readiness.
- Create reconsideration triggers for deferred features.

## 4. Non-Goals

- Do not promise to implement every downstream request.
- Do not accept broad widget-library requests by popularity alone.
- Do not require users to disclose private app details.
- Do not create telemetry.
- Do not block all releases on downstream feedback.

## 5. External Design

Adoption evidence template:

```markdown
# Snora Downstream Feedback

## App category
Desktop/local-first, internal tool, editor, media app, etc.

## Surfaces used
Header, sidebar, footer, menus, dialog, sheet, toasts, tabs, breadcrumb, direction.

## Feature flags used
widgets, lucide-icons, svg-icons, no-default-features, etc.

## What worked well

## What was awkward

## Missing framework-level concepts

## App-specific needs that should stay outside Snora

## Binary-size / compile-time concerns

## iced upgrade concerns
```

Feature-request triage:

| Request | Default response |
|---|---|
| Skeleton/overlay/direction issue | consider RFC |
| Form/table/chart/decorative widget | reject or redirect |
| Theming system | reject unless boundary changes intentionally |
| i18n beyond direction | redirect to application/other crates |
| Testing helper | consider internal tests before public crate |
| Popover | consider if multiple apps need it |

## 6. Internal Design

Repository/process changes:

- Add issue template: `.github/ISSUE_TEMPLATE/downstream-feedback.md`.
- Add feature request template with scope checklist.
- Add docs page: `docs/src/contributing/feedback-and-scope.md`.
- Link from README and roadmap.

Maintainer triage labels:

- `scope:core-layout`
- `scope:overlay`
- `scope:abdd`
- `scope:widgets-prefab`
- `scope:out-of-scope`
- `needs-rfc`
- `evidence-needed`

These labels are optional but useful once issue volume grows.

## 7. Testing and Acceptance

Acceptance criteria:

- Feedback template exists.
- Feature request template asks whether the request belongs in Snora or app code.
- Roadmap explains how downstream evidence affects 1.0 readiness.
- At least one real or dogfood app feedback record is collected before 1.0.

## 8. Documentation Updates

Update:

- README contribution section
- roadmap
- design decisions / scope boundary docs
- issue templates

Docs should be welcoming but firm about non-goals.

## 9. Compatibility and Migration

Compatible.

No runtime impact.

## 10. Open Questions

- What qualifies as a "third-party production app" for the 1.0 gate?
- Should maintainer-owned production apps count, or only external users?
- Should feedback records be public when app details are private?
