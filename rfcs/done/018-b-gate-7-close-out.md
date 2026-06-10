# RFC-018-B — Gate 7 Close-Out

**Status.** Implemented (v0.18.0)
**Tracks.** 1.0 governance.
**Touches.** `docs/src/contributing/api-freeze-review.md`,
`ROADMAP.md`.

## 1. State of Gate 7

As of v0.18, all checklist sections in `api-freeze-review.md` are ✅
except the human sign-off. The maintainer must explicitly decide:

> "The public API is ready for 1.0 commitment pending gates 1, 3, and 9."

## 2. Action: record the decision

Update `api-freeze-review.md` Gate 7 row:

```
| 7. Public API freeze review completed | ✅ v0.18 — all sections green; maintainer declared ready |
```

And mark Gate 7 as satisfied in ROADMAP.

## 3. What "freeze" means at pre-1.0

Pre-1.0 SemVer still allows minor-version breaking changes. "Freeze
review completed" means: the API is considered stable enough that a
subsequent breaking change would need a stronger justification than
previously, and the 1.0 declaration can follow as soon as gates 1, 3,
and 9 are satisfied.

## 4. Acceptance criteria

- Gate 7 marked ✅ in `api-freeze-review.md` with version and note.
- ROADMAP updated: Gates 2, 4, 5, 6, 7, 8, 10 satisfied (seven of ten).
- Remaining three gates documented clearly: iced upgrade, adoption, data.
