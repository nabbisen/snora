# RFC-017-A — 1.0 Gate Advancement

**Status.** Implemented (v0.17.0)
**Tracks.** 1.0 readiness / API audit.
**Touches.** `crates/snora-core/src/icon.rs` (`Icon` gains `PartialEq`),
`docs/src/contributing/api-freeze-review.md` (full current-state update,
Gate 2 marked ✅, type-names audit complete),
`ROADMAP.md` (Gate 2 and Gate 5 updated).

## 1. Gate 2 — vocabulary stability — satisfied

v0.13 through v0.16 = four consecutive minor releases with zero vocabulary
changes to `snora-core` public types. Gate 2 is satisfied.

## 2. Type-names audit — complete

All 22 public vocabulary types audited for derives. One gap found:
`Icon` lacked `PartialEq`. Fixed with a `cfg_attr`-conditional derive
(default + svg-icons) and a manual impl (lucide-icons, comparing `Lucide`
variant by discriminant as `usize` since `lucide_icons::Icon` doesn't
derive `PartialEq`).

All other types: `Dialog`/`Sheet`/`AppLayout` contain `Node` (cannot
derive without bound — correct); `SheetSize` intentionally omits `Eq`
because `f32` fields; all remaining types have complete derives.

## 3. Gate 5 update — RTL render-semantics added (via RFC-017-B)

Gate 5 note updated: "10 tests including 2 RTL" (from RFC-017-B).

## 4. Acceptance criteria

- `Icon` has `PartialEq` under all feature combinations.
- `api-freeze-review.md` reflects complete current state.
- Gate 2 marked ✅ in ROADMAP and api-freeze-review.
- No public API removed or renamed.
