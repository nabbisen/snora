# Handoff — 0.66 housekeeping: the toast close target is asserted exactly

**Ruled.** The owner, 2026-11-26, in the 0.66 scope.
**Release target.** 0.66.0.
**Where.** On `main`, by the one-`main` route, **after the review approves it.** It may travel in RFC-138's unit, or in a
commit of its own beside it.

## H-1: `close_button_target_is_exactly_the_floor` allows ±0.5 px

**Where:** `crates/snora/tests/toast_close_button.rs`.

**The finding** (logolig, 2026-11-24, triaged as F-5):
- logolig measured the toast's close control at 23.75 × 23.75 px.
- The architect's probe found the laid-out target **exactly 24.0 × 24.0** in all 12 cases: Info, Debug and Error; a short
  message, an empty one and a wrapping one; windows 800 and 320 px wide. logolig's figure is the edge's antialiasing at a
  fractional `y` (46.8), counted in whole pixels at 4×.
- **But the test asserts `|w − 24| ≤ 0.5` and `|h − 24| ≤ 0.5`** (`TOLERANCE`). A real 23.75 would pass it, and nothing
  else would catch it.

**The change:**
- Assert the target **exactly**: the laid-out bounds of the close control's focus target (the removal rig's `targets()`,
  as `mirrored_layout.rs` reads it), equal to the floor.
- Keep the corner-click reading (`pointer_target`). It is what says the pointer reaches the corners, and it keeps its
  tolerance. Assert both, with each one's message saying which it is.
- Cover the cases the probe did: three intents, the three message lengths, and the two widths.
- Use **exact** equality for the laid-out size. If a case yields a fraction (a layout that is genuinely not 24), report it.
  Do not widen the tolerance.

**Breakage:** set the floor to 23.75 (or the close control's padding so that it lays out under 24). The exact assertion
fails; show that the old ±0.5 one did not.

## Acceptance criteria

1. The breakage row fails, using named `cp` backups and `sha256sum -c`; never `git checkout` or `git restore`.
2. Every `ci.yaml` command, run locally.
3. No CHANGELOG entry (a test only), unless the exact assertion finds a real fraction. That would be a defect: report it.
