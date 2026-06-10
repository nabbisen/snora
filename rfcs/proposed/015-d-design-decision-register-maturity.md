# RFC-015-D — Design Decision Register Maturity

Status: Proposed  
Target release: v0.15  
Priority: Medium  
Type: Project governance / documentation quality

## 1. Summary

Mature Snora’s design-decision documentation into a maintained decision register with statuses, supersession links, and review triggers.

## 2. Motivation

The handoff emphasizes preserving the "why" in design decisions. This is essential for a small framework with strong non-goals. However, design-decision pages can become stale if they do not record status and supersession. A lightweight decision register will help future contributors understand which decisions are firm, which are provisional, and which were rejected.

## 3. Goals

- Preserve rationale behind scope boundaries.
- Record decision status clearly.
- Track superseded decisions.
- Link RFCs to decision records.
- Keep the process lightweight.

## 4. Non-Goals

- Do not create heavyweight governance.
- Do not require RFCs for every small patch.
- Do not turn docs into bureaucracy.
- Do not duplicate the changelog.
- Do not rewrite all historical decisions unless needed.

## 5. External Design

Decision record format:

```markdown
# Decision: No Snora-owned theming layer

Status: Accepted / Firm boundary
Date: YYYY-MM-DD
Related RFCs: RFC-014-C
Supersedes: none
Superseded by: none

## Context
## Decision
## Consequences
## Reconsideration Trigger
```

Recommended statuses:

- Proposed
- Accepted
- Firm boundary
- Deferred
- Rejected
- Superseded

Every non-goal should have a reconsideration trigger, even if the trigger is intentionally strict.

## 6. Internal Design

Repository changes:

- Either restructure `docs/src/contributing/design-decisions.md`, or add `docs/src/contributing/decisions/`.
- Add an index page listing decisions and statuses.
- Link new RFC files to decisions when accepted.
- Update release checklist: "Does this change alter a design decision?"

Avoid over-splitting at first. A single register page is acceptable until it becomes too long.

## 7. Testing and Acceptance

Acceptance criteria:

- mdBook builds.
- Decision register is linked from contributing docs.
- At least the major boundaries are recorded: no theming layer, no full i18n, no forms/tables/decorative widgets, no `snora-test`, ABDD scope, crate layering.
- Each decision has a reconsideration trigger.

## 8. Documentation Updates

Update:

- `docs/src/contributing/design-decisions.md`
- `docs/src/SUMMARY.md`
- release process docs
- RFC acceptance process if present

The docs should explain that design decisions are project memory, not marketing copy.

## 9. Compatibility and Migration

Compatible.

No runtime impact.

## 10. Open Questions

- Should decisions be one file or many files?
- Should RFC acceptance automatically create/update a decision record?
- Should rejected feature requests be recorded only if they recur?
