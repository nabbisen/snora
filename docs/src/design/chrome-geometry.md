# Token-derived chrome geometry

RFC-038 made chrome *colours* follow the emitted theme, because the
prefab widgets already read `theme.extended_palette()`. Geometry does
not follow, and cannot — an `iced::Theme` carries no spacing or radius.
`snora::design::widget::*` (RFC-040) adds a styled variant of each
prefab chrome widget that takes `&Tokens` first and maps its padding,
inter-element gaps, and corner radii to the `Spacing`/`Radius` scales,
leaving `snora::widget::*` exactly as it renders today.

```rust,ignore
use snora::{AppLayout, LayoutDirection, widget::app_header};
use snora::design::{Tokens, widget};

let tokens = Tokens::light();
let styled_header = widget::app_header(
    &tokens, "My App", vec![], &on_menu_action, None, None, LayoutDirection::Ltr,
);
```

Snora never calls these on the application's behalf. Applications that
keep calling `snora::widget::*` see no change — that set's geometry is
unchanged, proven by a regression test asserting each unstyled builder
still receives the exact literal it did before this RFC.

## One implementation, two geometry sources

Every widget's body is written exactly once — in its home module inside
`snora-widgets`, as a `pub(crate) fn build_*` taking a small geometry
struct. The unstyled `snora::widget::*` function passes that struct's
`::unstyled()` constructor (today's literals, unchanged); the styled
`snora::design::widget::*` function passes a token-derived one instead.
Drift between the two paths is structurally impossible, not merely
discouraged: there is nowhere for a second copy of a widget body to
live.

## The mapping

`Spacing::comfortable()` is `{xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl:
32}`; `Radius::default_roles()` is `{sm: 4, md: 6, lg: 10, pill: 999}`.
Every value below is mapped to the token whose own documented semantic
fits it — several land on today's exact number anyway, because the
original hardcoded literals already loosely followed this scale; that
coincidence is called out per-row, not hidden behind it.

| Widget | Value | Today | Token | Exact match? | Why |
|---|---|---|---|---|---|
| header | vertical padding | 8 | `Spacing::sm` | yes | compact internal gap fits a shallow bar inset |
| header | horizontal padding | 16 | `Spacing::lg` | yes | section spacing fits outer chrome padding |
| header | title↔menu / filler↔controls gap | 12 | `Spacing::md` | yes | ordinary component gap between sibling controls |
| header | corner radius | 0 | `Radius::sm` | **no** | RFC-040's stated target, see "Chrome radius" below |
| footer | horizontal padding | 16 | `Spacing::lg` | yes | same role as header |
| footer | vertical padding | 6 | `Spacing::sm` | **no** | 6 has no clean token; `sm` chosen for rhythm with header |
| footer | corner radius | 0 | `Radius::sm` | **no** | same as header |
| sidebar | icon-button gap | 16 | `Spacing::md` | **no** | reclassified — sibling controls in one list read as "ordinary component gap", not "section spacing"; deliberately not reproducing 16 |
| sidebar | rail padding | 16 | `Spacing::lg` | yes | outer chrome padding, same role as header/footer |
| sidebar | button radius | 6 | `Radius::md` | yes | `Radius::md` is documented "buttons, chips, notices" |
| tab bar | tab-to-tab gap | 2 | `Spacing::xs` | **no** | no clean equivalent; smallest available |
| tab bar | bar horizontal padding | 12 | `Spacing::md` | yes | ordinary component padding |
| tab bar | icon↔label gap | 6 | `Spacing::sm` | **no** | shared "inline gap" rhythm, see below |
| tab bar | per-tab horizontal padding | 12 | `Spacing::md` | yes | button padding |
| tab bar | per-tab vertical padding | 8 | `Spacing::sm` | yes | button padding, smaller than horizontal |
| tab bar | corner radius | 0 | `Radius::sm` | **no** | RFC-040's stated target |
| breadcrumb | crumb-to-separator gap | 6 | `Spacing::sm` | **no** | shared "inline gap" rhythm, see below |
| breadcrumb | trail horizontal padding | 12 | `Spacing::md` | yes | ordinary padding |
| breadcrumb | trail vertical padding | 4 | `Spacing::xs` | yes | shallow strip padding |
| breadcrumb | per-crumb horizontal padding | 4 | `Spacing::xs` | yes | compact button padding |
| breadcrumb | per-crumb vertical padding | 2 | `Spacing::xs` | **no** | no token smaller than `xs`; used as the floor |
| breadcrumb | button radius | 3 | `Radius::sm` | **no** | smallest radius available |
| menu | icon↔label gap | 6 | `Spacing::sm` | **no** | shared "inline gap" rhythm, see below |

**Shared inline-gap rhythm.** The tab bar's icon-to-label gap, the
breadcrumb's item-to-separator gap, and the menu's icon-to-label gap
were all independently hardcoded at `6` — the same value, for the same
kind of thing, in three unrelated files. All three map to `Spacing::sm`
(8): the closest semantic fit ("compact internal gap"), applied
uniformly so these three sites share one rhythm instead of an
accidentally-identical-but-unrelated `6`.

**Chrome radius.** The header, footer, and tab bar all hardcode
`radius: 0.0` — square corners — which RFC-040 names directly as "a
large part of why stock snora chrome reads as flat and dated". All
three map to `Radius::sm` (4): the smallest available radius, a modest
rounding appropriate for a full-width chrome strip.

**Unmapped literals** — no `Spacing`/`Radius` equivalent, left as
literal constants, identical in both the unstyled and styled paths:

- Sidebar's and breadcrumb's button border *widths* (`0.0` — "no
  border" is not expressible on either scale).
- The tab bar container's border *width* (`1.0` — border widths aren't
  part of either scale, matching `chrome_container_style`'s own border
  width).
- The tab bar's vertical padding (`0.0` — structural: tabs supply their
  own vertical padding via their button padding; this is an absence of
  a value, not a design literal).

## Density (RFC-040 Q-1)

Every styled function above reads geometry from `tokens.spacing`/
`tokens.radius` directly — **not** by branching on `tokens.density`.
This is deliberate: `snora_design::Density::Compact` is documented as
"reserved; not resolved" — `Spacing` has only a `comfortable()`
constructor, so there is no compact scale to select. Adding one would
be inventing token values outside RFC-040's scope (no new token roles
or scales). Reading `tokens.spacing`/`tokens.radius` directly means
geometry is already density-*correct* in the sense that matters:
whatever a future compact scale resolves to (or a hand-mutated custom
`Tokens` supplies today) flows through with no widget-level branch to
keep in sync. Verified with a hand-mutated `Tokens` pair in
`crates/snora-widgets/src/design/widget/tests.rs`, since no built-in
preset currently offers two distinct `Spacing` values to compare.

## Chrome radius shared with the dialog card

The `Radius::sm` chosen for header/footer/tab-bar chrome is
deliberately the *smallest* radius role — distinct from the dialog
card's `Radius::lg` (RFC-039, `docs/src/design/engine-surfaces.md`). A
full-width bar reads oddly with a pronounced rounding; a floating card
does not carry that constraint.

## What this RFC does not cover

Typography (`Typography` exists and chrome could use it, but font-size
changes reflow layout and deserve their own evidence — deferred with
its own trigger). New widgets — this RFC restyles what exists; the
permanent non-goals (no form, data-display, or decorative widgets)
stand unchanged. Engine surfaces (dialog card, modal dim) are RFC-039.
