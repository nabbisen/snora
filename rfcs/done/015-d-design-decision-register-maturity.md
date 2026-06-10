# RFC-015-D — Design Decision Register Maturity

**Status.** Implemented (v0.15.0)
**Tracks.** Project governance / documentation quality.
**Touches.** `docs/src/contributing/design-decisions.md`
(status labels + reconsideration triggers added to each existing entry).

## 1. [Decision] Single file, not split directory

`design-decisions.md` is 312 lines with 17 well-structured decisions.
Splitting into `decisions/` adds navigation complexity without benefit
at this scale. Keep one file; add a status table at the top.

## 2. Status vocabulary (adopted)

- **Firm boundary** — a deliberate non-goal. Requires an RFC with a concrete
  scenario to reopen.
- **Accepted** — the current approach, open to revision if evidence appears.
- **Deferred** — planned but not yet done; trigger condition stated.

Every entry in `design-decisions.md` gets a one-line status + reconsideration
trigger appended. The trigger must be concrete, not "if circumstances change."

## 3. Status table format

Add at the top of the file:

```markdown
## Decision index

| Decision | Status | Reconsideration trigger |
|---|---|---|
| No PageContract trait | Firm boundary | A trait that an engine actually consumes |
| One close sink per channel | Firm boundary | A concrete app that needs per-overlay close |
| … | … | … |
```

## 4. Acceptance criteria

- Status table exists at top of `design-decisions.md`.
- Every existing `## Why …` section has a one-paragraph status + trigger.
- `SUMMARY.md` cross-reference unchanged (design-decisions already linked).
- mdBook builds.
