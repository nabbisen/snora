# Overlays

snora has three overlay surfaces. They differ in how modal they are
and how the user dismisses them.

| Surface | Modal? | Default dismiss | Layer |
|---|---|---|---|
| `Dialog` | yes | click backdrop or close button you provide | above modal dim |
| `Sheet` | yes | click backdrop or close button you provide | above modal dim |
| `context_menu` slot | no (light overlay) | click anywhere outside | below modal dim |
| `header_menu` slot | no (light overlay) | click anywhere outside | below modal dim |

## One close sink, two channels

```rust,ignore
let layout = AppLayout::new(body)
    .on_close_modals(Message::CloseModals)   // dialog / sheet
    .on_close_menus(Message::CloseMenus);    // context / header menus
```

These are the **only** two close sinks. Individual `Dialog` and `Sheet`
values do not carry their own close messages — there is exactly one
place per channel.

If you set an overlay but leave its sink `None`, the overlay still
renders. The framework simply omits the click-outside-to-close
backdrop, and you must provide an explicit close button inside the
overlay content. snora never silently drops a populated overlay.

## Dialog

Snora paints the dim backdrop and centers your content. On the
**default path** (`snora::render`), that is all it does — no card, no
border, no fill. `Dialog` does not own the card chrome: you decide
whether the dialog content is a plain `container`, a styled card with a
border, an entire form. snora is a positioner, not a styler, **on this
path**.

```rust,ignore
use snora::{AppLayout, Dialog};

let layout = AppLayout::new(body)
    .dialog(Dialog::new(my_card_element()))
    .on_close_modals(Message::CloseModals);
```

**As of v0.27.0**, a token-styled card — fill, border, radius, all
derived from your active preset — is available opt-in via
`snora::design::render` (RFC-039) in place of `snora::render`, with no
other change to how you build your `AppLayout`. This matters if your
dialog content can land over saturated or dark backgrounds, where
unstyled centered content is easy to lose. See [Token-derived engine
surfaces](../design/engine-surfaces.md) for the mapping and how to opt
in.

### The token card only helps if you let it draw

**If your own dialog content is a styled container, snora's card is
behind yours and its border never renders.** On the default path snora
draws no card at all; on `snora::design::render` it draws one, and your
own container is drawn inside it — so a content container with its own
fill and radius overpaints the card whose border the tokens supply.

This is worth knowing because **the `palette.border` repair in v0.34.0
does not reach you in that case, in any preset.** It landed for
consumers who let the token card draw its own chrome; if you paint your
own, you are supplying the card's separation yourself and the token's
contrast guarantee is not the one in effect.

Reported by a consumer who measured it: scanning across their card's
edge at v0.28.0 and v0.42.0 — spanning the repair — produced
byte-identical pixels, because the falloff they were looking at was
their own shadow and there was no border pixel at all.

**Two things follow if you paint your own card.** Separation over the
dim comes from your fill, which is usually fine — that is the same
mechanism the 0.38→0.39 correction identified as what actually defines
the card. But **where there is no dim**, a card that separates only by
shadow can vanish: shadows carry almost no information in the
high-contrast presets, which is why the token card is border-defined
against its own surface rather than shadow-defined.

## Sheet

A modal panel anchored to one of the four window edges, occupying a
configurable size along the perpendicular axis.

```rust,ignore
use snora::{AppLayout, Sheet, SheetEdge, SheetSize};

let sheet = Sheet::new(my_drawer_content())
    .at(SheetEdge::Bottom)
    .with_size(SheetSize::Half);

let layout = AppLayout::new(body)
    .sheet(sheet)
    .on_close_modals(Message::CloseModals);
```

### Edges

| Variant | Where it slides from |
|---|---|
| `SheetEdge::Bottom` *(default)* | bottom of the window |
| `SheetEdge::Top` | top of the window |
| `SheetEdge::Start` | logical start (LTR=left, RTL=right) |
| `SheetEdge::End` | logical end (LTR=right, RTL=left) |

`Start` and `End` mirror automatically under
[`LayoutDirection::Rtl`](direction.md), like every other axis-aligned
piece of snora vocabulary.

The engine rounds only the *inside-facing* corners — the corners that
sit against the application content, not against the window edge. So
a bottom-anchored sheet rounds its top corners; a start-anchored sheet
under LTR rounds its right corners; etc.

### Size

The size is interpreted along the axis perpendicular to the edge: it
is a *height* for top/bottom edges and a *width* for start/end edges.

| Variant | Resolved size |
|---|---|
| `SheetSize::OneThird` *(default)* | 33 % of the relevant axis |
| `SheetSize::Half` | 50 % |
| `SheetSize::TwoThirds` | 67 % |
| `SheetSize::Ratio(f32)` | clamped to `0.0..=1.0` |
| `SheetSize::Pixels(f32)` | fixed pixels, ignores window size |

Pixel sizes ignore window resize and are usually wrong; prefer ratio
variants unless you have a hard pixel budget.

## Context menu

A floating menu (right-click style). It uses `on_close_menus`, not
`on_close_modals`, so it can coexist with an open dialog without one
dismissing the other.

```rust,ignore
let layout = AppLayout::new(body)
    .context_menu(my_floating_menu(point))
    .on_close_menus(Message::CloseMenus);
```

iced 0.14 does not surface the click coordinate alongside a button
press, so `Point`-based positioning of a context menu currently
requires either a `mouse_area` subscription or the iced advanced
widget API. The
[`examples/context_menu`](https://github.com/nabbisen/snora/tree/main/examples/context_menu)
demo uses fixed positions for clarity; treat it as a starting point
rather than a complete recipe.

## Header menu

Drop-down menus attached to a header bar (File / Edit / View …). See
the dedicated [Menus](menus.md) guide.

## Z-order recap

From bottom of the stack to top:

```text
0. skeleton           header / body+sidebar / footer
1. menu backdrop      transparent click sink (if any menu is open)
2. header_menu        header-attached dropdown
3. context_menu       floating context menu
4. modal dim          40 % dim click sink (if a modal is present)
5. dialog             centered; token-styled card via design::render
6. sheet              edge-anchored panel
7. toasts             always on top so they survive over modals
```

Toasts are deliberately on top of modals — a long-running export
finishing while a dialog is open should not be invisible.

## Accessibility responsibilities

Snora provides visual modality and pointer blocking. It does not manage
keyboard focus or screen-reader semantics. Before shipping a dialog or
sheet, check:

```text
[ ] A visible close/cancel button exists inside the overlay content.
[ ] `on_close_modals` is set when outside-click dismissal is intended.
[ ] Escape is wired via snora::keyboard::dismiss_on_escape if desired.
    (See keyboard section of overlay-interaction-semantics.md.)
[ ] Destructive actions have explicit labels; they are not triggered
    by backdrop click alone.
[ ] If initial keyboard focus inside the dialog matters, wire it via
    iced's widget::Id and the operate mechanism — Snora does not
    trap focus automatically.
```

ABDD is a layout discipline, not a complete accessibility or localization
stack. Snora's contribution is deterministic overlay layering and logical
edge placement. Full accessibility is a shared responsibility with iced
and your application.
