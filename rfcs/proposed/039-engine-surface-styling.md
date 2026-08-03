# RFC 039 — Engine surfaces: the dialog card and the modal dim

**Status.** Proposed
**Tracks.** Appearance milestone, second half (v0.27.0). Authorized by
RFC-037; constrained by RFC-036's additive-only covenant; depends on
RFC-038.
**Touches.** `crates/snora/src/design.rs`, a new design-gated render path,
`crates/snora/src/overlay/dialog.rs`, `crates/snora/src/render.rs`,
`docs/src/design/`.

## Summary

Two surfaces the engine renders itself are not styled at all:

```rust
// crates/snora/src/overlay/dialog.rs:14
center(dialog.content).into()          // no background, padding, radius, or border

// crates/snora/src/render.rs:192,210
Color::from_rgba(0.0, 0.0, 0.0, 0.4)   // fixed scrim, theme-independent
```

A dialog is therefore bare content floating on a fixed grey wash,
regardless of preset. This RFC gives both surfaces token-derived styling
behind the `design` feature, through a **sibling render entry point** —
because `AppLayout` cannot carry `Tokens` without inverting the crate
dependency direction.

## Motivation

RFC-038 made chrome *colours* follow the emitted theme. It deliberately did
not touch engine-rendered surfaces, and those are the ones a user notices
first: the dialog is the most visually prominent thing snora draws, and it
currently draws nothing.

This is the surface that most directly produced the original complaint
that an application built on snora is "not kind to see."

## The mechanism, and why it is not a field on `AppLayout`

The obvious design — `AppLayout::tokens(…)` — does not work:

- `AppLayout` lives in `snora-core`, which is **iced-free** and has **no
  dependencies at all**. `Tokens` lives in `snora-design`, an independent
  iced-free sibling. Making `snora-core` depend on `snora-design` would
  invert the documented architecture and pull tokens into every
  engine-only build, defeating the opt-in size discipline (DEC-11).
- Feature-gating the field would make the struct's shape depend on a Cargo
  feature, which breaks feature additivity.

Instead, add a sibling entry point in `snora`, which already depends on
both:

```rust,ignore
// snora::design::render — available when `design` is enabled
pub fn render<'a, Message>(
    layout: AppLayout<Element<'a, Message>, Message>,
    tokens: &Tokens,
) -> Element<'a, Message>;
```

`snora::render` is untouched. Applications opt in by calling the other one.

## The covenant bites here — deliberately

RFC-036's additive-only covenant freezes `snora-design`'s public surface,
including the `Palette`'s 18 roles. **Two things this RFC wants do not
exist in that surface:**

| Needed | Available? |
|---|---|
| A scrim / overlay / dim colour | **No.** The 18 roles are background, surface, surface_raised, three text roles, border, focus, and five semantic pairs. None is a scrim. |
| An elevation or shadow scale | **No.** `Tokens` carries `palette`, `spacing`, `typography`, `radius`, `focus`, `density` — there is no shadow or elevation token. |

This is the covenant working as intended: it makes the cost of extending
the frozen surface visible *before* the extension happens.

**Recommended: work within the covenant, do not extend it.**

- **The dim** derives from existing roles rather than gaining a `scrim`
  role. The implementer proposes the derivation and its rationale; it must
  be a deterministic function of tokens, not a magic constant, and must be
  checked against **all four presets** — a scrim that works in `light` and
  vanishes in `high_contrast_dark` is the same class of defect as RFC-038's
  invisible borders.
- **The card** uses `surface_raised` for fill, `border` for its edge,
  `radius.lg`, and `spacing.lg` for padding. A **border-defined card rather
  than a shadow-defined one** is a legitimate design choice, not a
  consolation: it renders correctly in high-contrast presets, where shadows
  are close to meaningless.

Extending the token surface instead would require **resetting D-3 and D-4
to open in the same change** (RFC-036's reopening obligation). That cost is
not worth a shadow. If a future RFC genuinely needs elevation, it can pay
it deliberately.

**Note for the record:** earlier scoping of this milestone listed
"elevation" alongside radius and spacing. That was the architect assuming a
token that does not exist. It is removed from scope here.

## Goals

- G-1. A dialog rendered through the design path has a visible card.
- G-2. The modal dim is token-derived and works in all four presets.
- G-3. `snora::render`'s output is byte-for-byte unchanged.
- G-4. No change to `snora-core`, and no change to the frozen design
  surface.

## Non-goals

- **N-1. No `Tokens` field on `AppLayout`.** See above.
- **N-2. No new token roles or scales.** Working within the covenant is the
  recommendation; overturning it is an owner decision, not an
  implementation one.
- **N-3. No chrome geometry.** Header/sidebar/footer spacing and radius are
  RFC-040.
- **N-4. `WARNING_COLOR` stays.** Toasts render on the design-inactive path
  too; changing them there would break RFC-037's gating invariant. Still
  deferred.
- **N-5. No sheet restyling.** The sheet already has an `opaque()` wrapper
  and edge-aware rounding; it is not visually broken in the way the dialog
  is. Revisit only with evidence.

## Compatibility

Additive and feature-gated. With `design` inactive, `snora::design::render`
does not exist and `snora::render` behaves exactly as in v0.26.

Applications that adopt it see a visual change — a **Changed** entry in the
v0.27 migration guide, not **Fixed**.

**RFC-036 covenant compliance:** nothing in the frozen token or
style-bridge surface is removed, renamed, retyped, or redefined, provided
the recommendation above is followed. Any deviation resets D-3/D-4.

## Testing

| Test | Assertion |
|---|---|
| Dim visibility | The derived scrim is distinguishable from the underlying surface in **all four presets** — a modest contrast floor against `background`, in the manner of RFC-038's border floor |
| Card distinguishable | The card fill is distinguishable from the page background in all four presets |
| Card text contrast | `text_primary` on the card fill meets WCAG AA |
| Gating invariant | `snora::render` output unchanged — render-semantics integration tests still pass untouched |
| Engine-only | `cargo check -p snora --no-default-features` still passes |

The render-semantics suite must continue to pass **without modification**.
If a test needs changing, the gating invariant has been broken.

## Alternatives considered

- **`Tokens` field on `AppLayout`.** Rejected — inverts the crate
  dependency direction and taxes engine-only builds.
- **Add `scrim` and `elevation` token roles.** Rejected as the default
  path: it costs a D-3/D-4 reset for cosmetic gain. Available to a future
  RFC that can justify it.
- **Style the dialog in `snora-widgets` instead.** Rejected — the dialog is
  composed by the engine's z-stack; the styling has to live where the
  composition does.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The scrim derivation works in some presets and not others | **High** | Medium | Explicit per-preset test; this is exactly how RFC-038's border defect appeared |
| Two render entry points confuse adopters | Medium | Low | The design path is documented as the `design`-feature variant; `snora::render` stays the default in all getting-started material |
| Scope creep into restyling every overlay | Medium | Medium | N-3 and N-5 bound it to the dialog and the dim |

## Open questions

- **Q-1 (owner).** Confirm the recommendation to work within the covenant
  rather than add `scrim` / `elevation` roles. Recommended: yes.
- **Q-2.** Should `snora::design::render` take `&Tokens` or a narrower
  style struct? Recommended `&Tokens`, matching every existing style-bridge
  signature.

## Acceptance criteria

1. `snora::design::render(layout, &tokens)` exists behind `design`.
2. Dialogs rendered through it have a card: fill, border, radius, padding —
   all token-derived.
3. The dim is token-derived and passes the per-preset visibility test.
4. `snora::render` is unchanged; render-semantics tests pass unmodified.
5. No file under `crates/snora-core/` or `crates/snora-design/` changes.
6. RFC-036 covenant compliance stated explicitly in the review request.

## Release implications

Ships in v0.27.0 with RFC-040. Additive; no existing API changes.
