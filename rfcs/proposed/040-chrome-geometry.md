# RFC 040 — Chrome geometry: token-derived spacing and radius

**Status.** Proposed
**Tracks.** Appearance milestone, second half (v0.27.0). Authorized by
RFC-037; constrained by RFC-036's additive-only covenant; depends on
RFC-038.
**Touches.** `crates/snora-widgets/src/` (prefab widgets and a new styled
variant module), `crates/snora/src/design.rs`, `docs/src/design/`.

## Summary

RFC-038 made chrome *colours* follow the emitted theme, because
`snora-widgets` already reads `theme.extended_palette()`. **Geometry does
not follow, and cannot** — an `iced::Theme` carries no spacing or radius.
The prefab widgets hardcode both:

```
header.rs:67,81    .spacing(12)
header.rs:90       .padding([8.0, 16.0])
sidebar.rs:38,57   .spacing(16) / .padding(16.0)
sidebar.rs:82      radius: 6.0
style.rs:41        radius: 0.0        ← square chrome corners
```

Those are unrelated magic numbers with no shared rhythm, and `radius: 0.0`
is a large part of why stock snora chrome reads as flat and dated.

This RFC adds **styled variants** that take `&Tokens`, leaving the existing
unstyled set exactly as it is.

## Motivation

Colour harmony without geometric harmony is half a design system. An
application can now select a preset and get consistent colours across
chrome, primitives and stock iced widgets — while the header pads by 8/16,
the sidebar by 16, and corners are square in one place and 6px in another.

The token set already carries the vocabulary to fix this: `Spacing`
(xs/sm/md/lg/xl/xxl) and `Radius` (sm/md/lg/pill). Nothing new is needed.

## Scope correction

Earlier scoping of this milestone said "radius, spacing rhythm, **and
elevation**". **There is no elevation or shadow token** — `Tokens` carries
`palette`, `spacing`, `typography`, `radius`, `focus`, `density`. Elevation
was an architect assumption about a vocabulary that does not exist, and is
removed from scope. Adding it would be a frozen-surface change under
RFC-036 requiring a D-3/D-4 reset; see RFC-039 for the same finding on the
engine side.

## Design

### Styled variants, not changed signatures

Add, under the `design` feature:

```rust,ignore
// snora::design::widget::*
pub fn app_header(tokens: &Tokens, /* existing params */) -> Element<'_, Message>;
pub fn app_side_bar(tokens: &Tokens, /* existing params */) -> Element<'_, Message>;
pub fn app_footer(tokens: &Tokens, /* … */) -> Element<'_, Message>;
pub fn app_tab_bar(tokens: &Tokens, /* … */) -> Element<'_, Message>;
pub fn app_breadcrumb(tokens: &Tokens, /* … */) -> Element<'_, Message>;
```

The existing `snora::widget::*` functions keep their signatures and their
current geometry. This satisfies RFC-037's gating invariant structurally:
an application that does not opt in cannot be affected, because it calls
different functions.

### One implementation, two geometry sources

The obvious risk is two parallel implementations that drift. **Do not
duplicate the widget bodies.** Extract each widget's construction into a
private builder parameterised by a small geometry struct:

```rust,ignore
struct ChromeGeometry { pad_x: f32, pad_y: f32, gap: f32, radius: f32 }
```

- the unstyled entry point passes today's literals, unchanged;
- the styled entry point passes token-derived values.

The literals then survive in exactly one place, explicitly labelled as the
pre-design defaults rather than scattered through the code. Drift becomes
structurally impossible rather than merely discouraged.

### Choosing the mapping

Spacing and radius must be **mapped deliberately and documented**, not
picked to match the current numbers. The point is a shared rhythm, not
reproducing today's inconsistency in token form. Where a current literal
has no clean token equivalent, that is a signal the literal was arbitrary —
say so in the mapping table rather than inventing a token value to match
it.

## Goals

- G-1. Chrome geometry derives from `Spacing` and `Radius` under `design`.
- G-2. The unstyled widget set is untouched — signatures and rendering.
- G-3. One implementation per widget, two geometry sources.
- G-4. The mapping is documented, including anywhere a current literal had
  no principled basis.

## Non-goals

- **N-1. No signature changes** to existing `snora::widget::*` functions.
- **N-2. No new token roles or scales** — no elevation, no shadow. See
  §Scope correction.
- **N-3. No new widgets.** N-3/N-4/N-5 of the permanent non-goals stand:
  no form, data-display or decorative widgets. This RFC restyles what
  exists.
- **N-4. No engine surfaces.** Dialog card and modal dim are RFC-039.
- **N-5. No typography changes.** `Typography` exists and chrome could use
  it, but font-size changes reflow layout and deserve their own evidence.
  Deferred, with the trigger being a concrete complaint about chrome text
  sizing.

## Compatibility

Additive and feature-gated. With `design` inactive, nothing changes.
Applications adopting the styled variants see a visual change — a
**Changed** entry in the v0.27 migration guide.

**RFC-036 covenant compliance:** consumes the frozen token surface, does
not modify it. No `Palette` role, no `Spacing`/`Radius` field, and no
style-bridge signature is touched.

## Testing

| Test | Assertion |
|---|---|
| Unstyled geometry unchanged | The unstyled builders receive exactly today's literals — a regression test on the geometry struct, not on rendered pixels |
| Styled geometry is token-derived | Each styled variant's geometry equals the mapped token value, for all four presets |
| Density respected | If `Density` participates in the mapping, compact and comfortable produce different values |
| Gating invariant | `cargo check -p snora --no-default-features` and `--features widgets` (no design) both pass |

Note what is **not** claimed: none of this tests that the result *looks*
good. Visual judgement belongs in the design workbench, which should gain
a styled-chrome view so the outcome can be seen rather than inferred.

## Alternatives considered

- **Change the existing signatures to take `&Tokens`.** Rejected —
  breaking, and it would force `design` on every widget user.
- **Feature-gate the geometry inside the existing functions.** Rejected —
  makes rendering depend on a Cargo feature, which is exactly the
  non-additive behaviour feature flags must not have.
- **Emit geometry through `iced::Theme`.** Not possible; the theme carries
  no geometry. This is the finding that produced this RFC.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Styled and unstyled implementations drift | **High** if duplicated | High | The shared-builder requirement makes it structural, not a discipline problem |
| Mapping reproduces today's inconsistency in token form | Medium | Medium | Mapping must be justified per widget; unmapped literals flagged as arbitrary |
| Doubling the widget surface confuses adopters | Medium | Low | Styled variants live under `snora::design::widget`, mirroring how primitives are already namespaced |

## Open questions

- **Q-1.** Should `Density` participate in the geometry mapping, or is it
  reserved for the primitives? Recommended: yes, include it — chrome
  padding is exactly what a density setting should affect. Flag if the
  implementation finds it awkward.
- **Q-2.** Does the design workbench gain a styled-chrome view in this RFC
  or a follow-up? Recommended: this one — without it the result cannot be
  judged.

## Acceptance criteria

1. `snora::design::widget::*` styled variants exist for header, sidebar,
   footer, tab bar and breadcrumb.
2. Each widget has **one** implementation, parameterised by geometry.
3. Unstyled variants are unchanged — signatures and geometry both.
4. The spacing/radius mapping is documented, including any literal found to
   be arbitrary.
5. Gating-invariant checks pass.
6. The design workbench can display styled chrome.
7. RFC-036 covenant compliance stated in the review request.

## Release implications

Ships in v0.27.0 with RFC-039, completing the appearance milestone's
geometry half. At that point RFC-037's boundary statement becomes fully
true and its incremental-coverage caveat should be revisited.
