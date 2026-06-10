# RFC-016-A — Alternate Engine Boundary Assessment

**Status.** Implemented (v0.16.0)
**Tracks.** Strategic architecture assessment.
**Touches.** `docs/src/contributing/alternate-engine-boundary.md` (new),
`docs/src/reference/architecture.md` (conservative wording check),
`docs/src/SUMMARY.md`.

## 1. [Decisions] Open questions answered

### Q: Is alternate-engine potential worth mentioning publicly?

Yes, but with conservative wording. The `snora-core` iced-free boundary
is real and valuable, and a test-double engine (or an alternate renderer
someone builds independently) should know the required capabilities. But
Snora must not imply a commitment it has not made.

Adopted public wording:
> `snora-core` is iced-free to keep vocabulary stable, testable, and
> insulated from iced upgrades. It may be useful for alternate engines in
> the future, but Snora does not currently promise alternate renderer
> support.

### Q: Would a test-double engine help internal testing?

`iced_test` with its headless CPU renderer already serves this purpose
as of v0.11 (RFC-011-D). A separate test-double engine is not needed.

### Q: Are there downstream users wanting vocabulary without iced?

No evidence. Not a roadmap item. Mentioned conservatively in the doc.

## 2. Deliverable

`docs/src/contributing/alternate-engine-boundary.md` contains:

1. Why `snora-core` is iced-free (vocabulary stability, not portability promise).
2. What an alternate engine would require (capability table from the planning draft).
3. What is iced-specific and not portable (`snora-widgets`, `iced::Element` return types).
4. The conservative public wording above.
5. Status: no implementation planned; `iced_test` covers test-double needs.

## 3. Acceptance criteria

- `alternate-engine-boundary.md` exists, linked from SUMMARY.
- No new public abstraction added.
- Architecture docs use conservative wording.
- README does not overclaim portability.
