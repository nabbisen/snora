# Token-derived chrome geometry

RFC-038 made chrome *colours* follow the emitted theme, because the
prefab widgets already read `theme.extended_palette()`. Geometry does
not follow, and cannot — an `iced::Theme` carries no spacing or radius.
`snora::design::widget::*` (RFC-040) adds a styled variant of each
prefab chrome widget that takes `&Tokens` first and maps its padding,
inter-element gaps, and corner radii to the `Spacing`/`Radius` scales,
leaving `snora::widget::*` exactly as it renders today.

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:chrome_header}}
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
| header | title↔menu / filler↔controls gap | 12 | `Spacing::md` | yes | ordinary component gap between sibling controls. The title is also followed by a fixed 20 px spacer, so the title's end to the first menu is 12 + 20 + 12 = 44 |
| header | corner radius | 0 | `Radius::sm` | **no** | RFC-040's stated target, see "Chrome radius" below |
| footer | horizontal padding | 16 | `Spacing::lg` | yes | same role as header |
| footer | vertical padding | 6 | `Spacing::sm` | **no** | 6 has no clean token; `sm` chosen for rhythm with header |
| footer | corner radius | 0 | `Radius::sm` | **no** | same as header |
| sidebar | icon-button gap | 16 | `Spacing::md` | **no** | reclassified — sibling controls in one list read as "ordinary component gap", not "section spacing"; deliberately not reproducing 16 |
| sidebar | rail padding | 16 vertical (8 horizontal) | `Spacing::lg` | yes, vertical | outer chrome padding, same role as header/footer |
| sidebar | button radius | 6 | `Radius::md` | yes | `Radius::md` is documented "buttons, chips, notices" |
| tab bar | tab-to-tab gap | 2 | `Spacing::xs` | **no** | no clean equivalent; smallest available |
| tab bar | bar horizontal padding | 12 | `Spacing::md` | yes | ordinary component padding |
| tab bar | icon↔label gap | 6 | `Spacing::sm` | **no** | shared "inline gap" rhythm, see below |
| tab bar | per-tab horizontal padding | 12 | `Spacing::md` | yes | button padding |
| tab bar | per-tab vertical padding | 8 | `Spacing::sm` | yes | button padding, smaller than horizontal |
| tab bar | corner radius | 0 | none | — | RFC-102 retired the tab bar's radius: a straight bar under a straight underline |
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

**Chrome radius.** The header and footer hardcode `radius: 0.0` on the
unstyled path (square corners), which RFC-040 names directly as "a large part
of why stock snora chrome reads as flat and dated". Both map to `Radius::sm`
(4): the smallest available radius, a modest rounding appropriate for a
full-width chrome strip. **The tab bar has no radius** since RFC-102, which
made its edge a straight rule. *(Corrected 2026-11-15, from the 0.63 audit: this listed the tab bar as a third.)*

**Unmapped literals** — every literal in the chrome builders that is not a `Spacing` or
`Radius` token, the same on the unstyled and the styled path (complete as of 0.63, from a reading
of `header.rs`, `footer.rs`, `sidebar.rs`, `tab.rs`, `crumb.rs`, `menu.rs` and `design/widget.rs`):

| Where | Literal | What it is |
|---|---|---|
| header | a fixed 20 px spacer | between the title and the first menu (title to menu is `gap + 20 + gap`) |
| header, footer | `radius: 0.0` (unstyled) | square corners (see *Chrome radius*) |
| sidebar | rail width 64, button 48 | the rail and each rail button |
| sidebar | horizontal padding 8 | derived: `(64 − 48) / 2` (RFC-099) |
| sidebar | labelled rail at most 240 wide | its widest form |
| sidebar | label gap `gap / 2` | between a button and its name; follows the mapped `gap` |
| sidebar | unstyled tooltip: radius 6, padding `[4, 8]`, gap 4, border 1 | the styled path maps these through `card_raised` |
| sidebar, breadcrumb | button padding 0, border width 0 | "no padding", "no border" |
| tab bar | underline 2 px; bottom edge 1 px | the active tab's indicator; the bar's edge (its container paints no border, which a test asserts) |
| tab bar | vertical padding 0 | tabs supply their own, through their button padding |
| tab bar | inactive label `mix(text, page, 0.3)` (unstyled) | on the design path the share is the first that holds the palette's level (RFC-131) |
| breadcrumb | ancestor rule 1 px | under an ancestor crumb |
| breadcrumb | separator `mix(text, page, 0.5)` | held to 3:1 as a glyph (RFC-135) |
| menu | iced's default button padding | a trigger's and an item's padding |

*(Corrected 2026-11-16, from the 0.63 audit follow-up: this list named four of these.)*
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

The `Radius::sm` chosen for header and footer chrome is
deliberately the *smallest* radius role — distinct from the dialog
card's `Radius::lg` (RFC-039, `docs/src/design/engine-surfaces.md`). A
full-width bar reads oddly with a pronounced rounding; a floating card
does not carry that constraint.

## What this RFC does not cover

Typography, which RFC-104 did later (0.52): chrome labels take the `label`
role, the header and toast titles `title`, and the toast message `body`. New widgets — this RFC restyles what exists; the
permanent non-goals (no form, data-display, or decorative widgets)
stand unchanged. Engine surfaces (dialog card, modal dim) are RFC-039.
