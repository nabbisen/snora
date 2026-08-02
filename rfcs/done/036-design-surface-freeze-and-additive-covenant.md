# RFC 036 — Design surface freeze review and additive-only covenant

**Status.** Implemented (v0.25.3)
**Tracks.** Snora Design stability governance; closes design gates D-3 and
D-4 (RFC-034). Prerequisite for the v0.26 appearance work
(RFC-037 … RFC-040).
**Touches.** `docs/src/contributing/api-freeze-review.md` (D-gate rows),
`docs/src/contributing/api-governance.md` (covenant text). No source
changes.

## Summary

RFC-034 defines design gates D-3 ("token model stable for ≥2 consecutive
minors") and D-4 ("style bridge stable for ≥2 consecutive minors"). This
RFC records the freeze review that closes both, on evidence spanning
**six** consecutive minors (v0.20 → v0.25), and establishes the
**additive-only covenant** that lets the v0.26 appearance work proceed
without retroactively invalidating the gates it just closed.

Closing a stability gate immediately before expanding the surface it
covers would hollow out the gate. The covenant is the mechanism that
prevents that: D-3/D-4 close on what the surface *has been*, and the
covenant binds what may be done to it *next*.

## Motivation

Two pressures meet here.

1. D-3/D-4 have been eligible to close since v0.22 and have sat open
   through four further minors purely for want of a recorded review. The
   v0.25.1 handoff lists "close design D-gates D-3/D-4" as an actionable
   next step blocked on nothing.
2. The v0.26 milestone (RFC-037 …) will extend the design layer to cover
   snora's own rendered surfaces. Extending a surface is safe; *changing*
   one that was just declared stable is not.

Recording the review and the covenant together resolves both, and makes
the constraint on RFC-038 … RFC-040 explicit before those RFCs are
written rather than discovered during their review.

## Goals

- G-1. Record the freeze review with reproducible evidence.
- G-2. Close D-3 and D-4.
- G-3. Define precisely which items constitute the frozen surface.
- G-4. Establish the additive-only covenant and its reopening conditions.

## Non-goals

- **N-1. No source changes.** This RFC records and constrains; it does not
  implement.
- **N-2. Does not close D-1, D-2, D-5, D-6, or D-7.** D-1/D-2 are coupled
  to core gate 1 (iced major upgrade); D-5 to core gate 3; D-6 awaits a
  promotion with evidence; D-7 is reviewed per minor.
- **N-3. Does not freeze the design surface against all future change.**
  A freeze gate records demonstrated stability. It is not a promise of
  immutability. The covenant defines what change remains permitted.
- **N-4. Does not authorize any appearance work.** That is RFC-037's job.

## Evidence

Method: `git diff 0.20.0 0.25.2 -- <path>`, reproducible from the tags in
this repository.

### Token model (D-3)

Across v0.20.0 → v0.25.2, `crates/snora-design/src/` changed in exactly
**two files**:

| File | Change | Assessment |
|---|---|---|
| `palette.rs` | `roles()` narrowed from `pub` to `#[cfg(test)] pub(crate)` | **Removal from the public API**, deliberate (DEC-12). The fixed-size `[Color; 18]` return type would have become a breaking change on any future role addition to `#[non_exhaustive] Palette`. Shipped at a minor per pre-1.0 policy. |
| `contrast.rs` | `composite_over` gained a debug-only precondition (`debug_assert!(bg.is_opaque())`) plus documentation | **Contract tightening.** Signature unchanged. Debug builds now panic on a translucent background that previously produced a silently wrong result. |

Unchanged, byte-for-byte, across all six minors: `tokens.rs`,
`palette.rs`'s 18 role fields, `color.rs`, `spacing.rs`, `typography.rs`,
`radius.rs`, `focus.rs`, `variants.rs`, and every file under `presets/`.

**Zero** role additions, renames, retypings, or preset value changes.

Honest qualification: this is not "no change at all." One public item was
removed and one contract was tightened. Both were deliberate hardening in
service of SemVer safety, and neither altered the token *model*. D-3 asks
whether the token model is stable. It is.

### Style bridge (D-4)

Across the same span, `crates/snora-widgets/src/design/style/` changed by
**addition only**: `style::progress::toned` arrived in v0.21 alongside the
progress primitive (RFC-032).

All fifteen public style-bridge functions retain their v0.20 signatures.
No function was removed, renamed, or retyped.

### Assessment

| Gate | Bar | Actual | Disposition |
|---|---|---|---|
| D-3 token model stable | ≥2 consecutive minors | 6 (v0.20–v0.25) | **Satisfied — close** |
| D-4 style bridge stable | ≥2 consecutive minors | 6 (v0.20–v0.25) | **Satisfied — close** |

## The frozen surface

"The design surface" means precisely these items. Anything not listed is
outside the covenant.

**Token surface** — all public items of `snora-design`: `Tokens` and its
constructors (`light`, `dark`, `high_contrast_light`,
`high_contrast_dark`); `Palette` and its 18 role fields; `Color`;
`Spacing`; `Typography` and `TextRole`; `Radius`; `FocusTokens`; `Tone`,
`Emphasis`, `Size`, `Density`; and the `contrast` module's three
functions.

**Style-bridge surface** — all public functions of
`snora_widgets::design::style`: `color::to_iced_color`; `button::{primary,
secondary, ghost, danger}`; `container::{card_surface, card_raised,
card_selected}`; `progress::toned`; `text::{body_size, body_small_size,
label_size, title_size, heading_size, display_size}`.

The design **primitives** (`button`, `card`, `notice`, `chip`,
`progress` helper modules) are deliberately **not** in the frozen surface.
They are governed by the promotion lifecycle in RFC-034 /
`api-governance.md`, which is a different and more permissive regime.

## The additive-only covenant

While D-3 and D-4 are closed, work on the design layer **may**:

- add new items to `snora-design` (new token types, new fields on
  `#[non_exhaustive]` types, new presets, new contrast helpers);
- add new functions to the style bridge;
- add new modules, entry points, and primitives elsewhere;
- change values *inside* a preset **only** where a contrast test proves
  the change fixes an accessibility defect, recorded as **Fixed**.

It **may not**, without reopening the gate:

- remove, rename, or retype any item in the frozen surface;
- change the signature of any frozen style-bridge function;
- add, remove, rename, or retype a `Palette` role;
- change the meaning of an existing token (e.g. redefining what
  `Spacing::md` denotes) even where the type is untouched.

**Reopening condition.** If work requires any forbidden change, the
implementing RFC must say so explicitly and **reset D-3 and D-4 to open**
in the same change. It must not proceed and rationalise afterwards. The
gate's value is entirely in its being expensive to reverse.

**Deliberate consequence.** RFC-038 … RFC-040 are thereby constrained to
mechanisms that do not alter the frozen surface. That constraint was a
material input to their proposed design and is the reason none of them
modifies `snora-core` or an existing widget signature.

## Compatibility, security, operational

**Compatibility.** None affected — no source change.
**Security.** None affected. No data flow, dependency, or integration
change. Existing controls remain valid.
**Operational.** No CI change. The covenant is enforced at RFC review, not
by tooling; a mechanical check is possible later but is not proposed here
(see RFC-035 Q-3).

## Testing and verification

No new tests. Evidence is reproducible by:

```bash
git diff 0.20.0 0.25.2 -- crates/snora-design/src/
git diff 0.20.0 0.25.2 -- crates/snora-widgets/src/design/style/
```

The reviewer should re-run both and confirm the change set matches the
Evidence section.

## Alternatives considered

- **Close D-3/D-4 with no covenant.** Rejected: the next milestone
  extends the surface, and a gate closed without a constraint on what
  follows is a gate that means nothing.
- **Hold D-3/D-4 open until after v0.26.** Rejected: it would misreport
  six minors of demonstrated stability as instability, and the gates would
  then close on a *shorter* stable span than exists today.
- **Freeze the primitives too.** Rejected: primitives are explicitly
  governed by RFC-034's promotion lifecycle, which allows experimental
  status. Freezing them would contradict a shipped governance decision.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| v0.26 work discovers it needs a forbidden change | Medium | Medium | Reopening is defined and cheap to execute; the covenant makes the cost visible at design time, not merge time |
| The covenant is treated as advisory | Low | High | It is stated as a gate condition with an explicit reset obligation, and cited in each dependent RFC |
| "Additive" is read loosely (e.g. semantic drift in an unchanged type) | Medium | Medium | The forbidden list names meaning-change explicitly, not just signature change |

## Acceptance criteria

1. `api-freeze-review.md` D-3 and D-4 rows read ✅ with the version span
   v0.20–v0.25 and a link to this RFC.
2. No other gate row's status value changes.
3. `api-governance.md` carries the covenant: frozen surface list,
   permitted changes, forbidden changes, reopening obligation.
4. The two `git diff` commands reproduce the Evidence section.
5. RFC-037 … RFC-040 each cite this covenant and state their compliance.

## Release implications

Documentation and governance only; no version implication of its own.
Closes 2 of 8 design D-gates, bringing the design track to D-3, D-4, D-8
satisfied. Core 1.0 gates are untouched — the design track does not block
core 1.0.
