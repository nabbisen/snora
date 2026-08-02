# RFC 038 — Token-derived `iced::Theme` emission

**Status.** Proposed
**Tracks.** v0.26 appearance milestone. Authorized by RFC-037; constrained
by RFC-036's additive-only covenant. Prerequisite for RFC-039 and RFC-040.
**Touches.** `crates/snora-widgets/src/design/theme.rs` (new),
`crates/snora-widgets/src/design.rs`, `crates/snora/src/design.rs`,
`docs/src/design/`, contrast test suite. No changes to `snora-core`,
`snora-design`, or any existing signature.

## Summary

Add one pure function — `snora::design::theme(&Tokens) -> iced::Theme` —
that derives a complete iced theme from a Snora Design token bundle, so
that stock iced widgets and the window background follow the same palette
as snora's design primitives.

The function does **not** use iced's default palette generator. It builds
`iced::theme::palette::Extended` directly from snora-design's 18 verified
roles, because iced's generator would silently substitute its own
heuristic colors for the contrast-tested ones the application selected.

## Motivation

Today an application that selects `Tokens::high_contrast_dark()` gets
high contrast on snora design primitives and nowhere else. Its
`text_input`, `pick_list`, `scrollable`, and window background follow
whatever `iced::Theme` it configured separately — so the preset reaches a
minority of the pixels on screen, and the application must maintain two
palettes by hand and keep them in agreement.

This is the double-configuration that DEC-02's original reasoning warned
about; it exists *today*, and emission removes it rather than creating it.

## Verified iced 0.14 facts

All verified against `iced_core-0.14.0` in the local registry. These
determine the design; an implementer should re-confirm them before
starting.

| Fact | Location | Consequence |
|---|---|---|
| `Theme::custom_with_fn(name, palette, generate: impl FnOnce(Palette) -> Extended)` | `theme.rs:100` | We may supply our own `Extended` generator |
| `Extended` fields are all `pub`: `background`, `primary`, `secondary`, `success`, `warning`, `danger`, `is_dark` | `theme/palette.rs:288` | We can construct a full `Extended` |
| `Pair` has **`pub color`** and **`pub text`** | `theme/palette.rs:426` | We can build pairs by struct literal, bypassing `Pair::new` |
| `Pair::new` calls `readable(color, text)` | `theme/palette.rs:440` | Using it would **override** our verified text colors |
| `readable()` lightens/darkens by 0.1, 0.2, … carrying `TODO: Compute factor from relative contrast value` | `theme/palette.rs:692` | iced's correction is a heuristic, not a contrast computation |
| `is_readable_on` threshold is `relative_contrast >= 6.0` | `color.rs:183` | Stricter than WCAG AA (4.5) but applied heuristically |
| **Base `Palette` and `Extended` both have a `warning` field** | `theme/palette.rs:18, 297` | iced 0.14 *does* have a warning pair |

### The finding that shapes the design

`Pair::new` does not pair colors naively — it *corrects* them. That is
reasonable behavior for iced, but for snora it is the problem: passing
snora-design's contrast-tested roles through `Pair::new` means iced may
replace them with heuristically-adjusted approximations. The application
would then be running colors that are neither its tokens nor snora's
verified values, and `snora-design`'s WCAG guarantees would not transfer.

Because `Pair`'s fields are public, we can avoid this entirely: construct
`Pair { color, text }` directly from the token roles we have already
tested. **Emission then preserves exactly the colors the preset
guarantees**, and the contrast obligation becomes a property we can prove
rather than one we inherit from someone else's heuristic.

## Goals

- G-1. `snora::design::theme(&Tokens) -> iced::Theme`, pure, no state.
- G-2. The emitted `Extended` carries snora-design's verified colors
  unmodified.
- G-3. Automated contrast tests over the *emitted* theme for all four
  presets, not only over the raw tokens.
- G-4. Document the 18→6 mapping, including what is necessarily lost.

## Non-goals

- **N-1. Snora does not hold theme state.** The function returns a value;
  the application stores it and passes it to iced's `.theme()` hook.
- **N-2. No change to any existing signature**, per RFC-036's covenant.
- **N-3. No automatic application.** Nothing in snora calls this function
  on the application's behalf.
- **N-4. Does not restyle snora's own chrome geometry.** Colors follow
  transitively because chrome reads `iced::Theme`; radius, spacing, and
  elevation are RFC-040's scope.
- **N-5. Does not resolve the `WARNING_COLOR` question** — see Open
  questions Q-2.

## Design

### Placement

The function is iced-typed, so it cannot live in `snora-design` (iced-free,
NF-1). It goes in `crates/snora-widgets/src/design/theme.rs`, re-exported
as `snora::design::theme` from `crates/snora/src/design.rs`. This is
purely additive and touches neither `snora-core` nor the frozen surface.

### Shape

```rust,ignore
/// Derive a complete `iced::Theme` from a Snora Design token bundle.
pub fn theme(tokens: &Tokens) -> iced::Theme;
```

Implementation outline:

1. Build the six-slot base `iced::theme::Palette` from the token roles —
   this is what `Theme::palette()` reports to consumers.
2. Pass a **custom generator** to `Theme::custom_with_fn` that constructs
   `Extended` from the 18 roles using `Pair { color, text }` struct
   literals, so no value is heuristically altered.
3. Set `is_dark` from the preset rather than inferring it from background
   luminance.

### The 18 → 6 mapping

`Theme::palette()` returns the six-slot base palette, so the mapping is
necessarily lossy *for that accessor only*. Widgets read
`extended_palette()`, which we construct in full — so the loss does not
affect rendering.

The mapping table (background, text, primary, success, warning, danger ←
their corresponding token roles) must be documented in
`docs/src/design/` with an explicit note that `Theme::palette()` is a
lossy view and `extended_palette()` is authoritative.

## Compatibility

Additive and feature-gated. With `design` inactive the function does not
exist and rendering is unchanged — **RFC-037's gating invariant holds
trivially**, because nothing calls this function unless the application
does.

Applications that adopt it will see a visual change. That is a **Changed**
entry in the v0.26 migration guide, not **Fixed**.

**RFC-036 covenant compliance:** no item in the frozen token or
style-bridge surface is removed, renamed, retyped, or redefined. This RFC
is purely additive. ✅

## Testing and verification

The contrast obligation is the blocking requirement, not an afterthought.

| Test | Assertion |
|---|---|
| Emitted-theme contrast, all four presets | Every `Pair` in the emitted `Extended` meets WCAG AA (≥ 4.5 for body text) using `snora_design::contrast::contrast_ratio` |
| Fidelity | Every emitted color equals the corresponding token role exactly — proves iced's heuristic did not run |
| `is_dark` correctness | Matches the preset's intent for all four |
| High-contrast strictness | HC presets meet AAA (≥ 7.0) where the tokens already do |
| Feature isolation | `cargo check -p snora --no-default-features` and the `widgets`-without-`design` matrix entry still pass |

Standard gates additionally: fmt, clippy, per-crate tests, engine-only
build, `mdbook build docs`, `mdbook test docs`.

## Alternatives considered

- **`Theme::custom` (default generator).** Rejected: it routes every pair
  through `Pair::new` → `readable()`, discarding the verified colors. This
  is the whole reason the RFC specifies a custom generator.
- **Document a hand-rolled recipe instead of shipping a function.** This
  was the pre-existing plan (the orbok question). Rejected: every adopter
  would re-derive the mapping, and the `Pair::new` pitfall above is
  precisely the kind of thing each of them would get wrong silently.
- **Emit only the base six-slot palette.** Rejected: iced would then
  generate `Extended` heuristically, reintroducing the substitution.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Emitted theme drifts from tokens in a future iced version | Medium | High | Fidelity tests assert exact equality; they fail loudly if iced changes `Extended`'s shape |
| `Extended` gains fields in a future iced version | Medium | Medium | Construction is exhaustive; a new field is a compile error, not a silent default — this is desirable |
| Applications assume `Theme::palette()` is authoritative | Medium | Low | Documented as a lossy view |
| iced's 6.0 readability threshold differs from our AA target | Certain | Low | We bypass iced's correction, so its threshold does not apply to emitted values |

## Open questions

- **Q-1.** Should `theme()` take `&Tokens` or `Tokens`? Recommended
  `&Tokens`, matching every existing style-bridge signature.
- **Q-2 (deferred to RFC-039).** iced 0.14 **does** have a warning pair, so
  `WARNING_COLOR` (`crates/snora/src/toast.rs:46`) is now removable in
  principle. But toasts render on the design-**off** path too, and
  switching them to `extended_palette().warning` would change appearance
  for existing applications — violating RFC-037's gating invariant. This
  needs an explicit owner decision, not an implementer's judgment.

## Acceptance criteria

1. `snora::design::theme(&Tokens) -> iced::Theme` exists behind the
   `design` feature and is documented.
2. The emitted `Extended` is constructed without `Pair::new`; fidelity
   tests prove every emitted color equals its token role exactly.
3. Contrast tests pass over the emitted theme for all four presets.
4. The 18→6 mapping is documented, including the lossy-accessor note.
5. `cargo check -p snora --no-default-features` and every feature-matrix
   entry pass unchanged.
6. RFC-036 covenant compliance is stated in the review request.
7. No file under `crates/snora-core/` or `crates/snora-design/` changes.

## Release implications

Ships in v0.26 as an **Added** entry. Advances design gate D-2's
prerequisites (minimal path clean) but closes no gate on its own. It is
the dependency for RFC-039 and RFC-040: once an application's base theme
derives from tokens, snora's existing chrome follows transitively, which
is what lets RFC-040 confine itself to geometry.
