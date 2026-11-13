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

**A press outside (a click or a tap) and Escape publish them. A wheel does not** (0.60):
a scroll that strays off a dialog's card does nothing, and the page beneath a dialog or an
open menu shows no hover and does not scroll. Until 0.60, a wheel over the dim closed the
dialog.

If you set an overlay but leave its sink `None`, the overlay still
renders. The framework simply omits the click-outside-to-close
backdrop, and you must provide an explicit close button inside the
overlay content. snora never silently drops a populated overlay.

## Where focus starts in a dialog (0.61, RFC-127)

When a dialog opens, the page's focus is set aside and nothing in the dialog is focused, so the
first Tab reaches its first control. To start somewhere else, for example on "Cancel" in a
destructive confirmation, give that control an id and focus it from the update that opens
the dialog:

```rust,ignore
const CANCEL: &str = "confirm-cancel";

// view
snora::design::control(&t, text("Cancel").size(t.typography.label.size))
    .id(CANCEL)
    .on_press(Message::Cancel)
    .style(move |_, status| style::button::secondary(&tok, status))

// update
Message::AskToDelete => {
    self.confirm = true;
    return iced::widget::operation::focus(CANCEL);
}
```

- **The task runs on the new view,** so it can name a control in the dialog that the same
  update opens. A task that runs before the dialog is in the view finds nothing.
- **The control shows its ring,** and Enter or Space presses it, as if Tab had reached it.
- **An id must be unique in the window,** as for `text_input`: `focus(id)` focuses every
  control that has it.
- **An id that no reachable control has,** including one under the dialog, focuses nothing and
  **leaves nothing focused.** `is_focused(id)` then gives no answer at all, so do not wait on it.
- **The `design::button` variants take no id.** Build the control with
  `snora::design::control` and the same style function, as above. **It draws exactly as the variant
  does when its label is drawn at the label size,** `text(label).size(t.typography.label.size)`, as
  above. The padding is already the same. At iced's default text size the control is a little
  larger. *(Corrected 2026-11-13, twice: this first said it looked identical with no condition, and
  the 0.62 audit's correction then blamed the padding, which was wrong. The label's size is the
  difference, tested in 0.62.)*

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

### The token card, around a styled container of your own

**On `snora::design::render` the card pads its content, so its fill and its border show around a
container of yours.** Across the card's edge, the pixels are the same with or without a filled,
rounded container as content. On the default path snora draws no card at all.

*(Corrected 2026-11-13, from the 0.62 audit: this section said the card's border never rendered
under a styled container of yours, and that the v0.34.0 `palette.border` repair did not reach
you. Measured on 0.61, both were false.)*

*History.* Reported by a consumer who measured it: scanning across their card's
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

> **Known defect, 0.4 to 0.61, fixed in 0.62:** a sheet on the **right** (`End` under LTR,
> `Start` under RTL) is given the spacer's share of the width. `OneThird` draws two thirds, and
> `TwoThirds` draws one third. `Half` and `Pixels` are right, and so are `Top`, `Bottom` and a
> sheet on the left. Found by the 0.62 audit.

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
4. modal dim          44 % dim click sink (if a modal is present)
5. dialog             centered; token-styled card via design::render
6. sheet              edge-anchored panel
7. toasts             always on top so they survive over modals
```

Toasts are deliberately on top of modals — a long-running export
finishing while a dialog is open should not be invisible.

**On the engine path a dialog has no card of its own: put its content on a container with the
theme's background.** A sheet has one: a surface, a 1 px border and inside-facing corners.
*(Corrected 2026-11-13, from the 0.62 audit: this said a sheet had none.)* A control or text placed directly on the dim
may not be legible, and its focus ring may fall under 3:1 (RFC-122, measured in 18 of
the 22 stock themes). The design path draws a card for you.

## Accessibility responsibilities

Snora provides visual modality, pointer blocking and, from 0.55, keyboard
blocking (RFC-119): with a dialog or sheet open, Tab and Enter reach only the
overlay and the toasts. With `on_close_modals` set, Escape closes it, as a click
on the dim does. When it closes, focus returns to the control that had it, unless
the page has changed or your application has focused something itself (Laws 7 and
8). It provides no screen-reader semantics. Before shipping a dialog or sheet,
check:

```text
[ ] A visible close/cancel button exists inside the overlay content.
[ ] `on_close_modals` is set when outside-click dismissal is intended.
    Escape then closes the overlay too, with nothing to wire (Law 7).
[ ] With no `on_close_modals`: the visible close button above is the only
    way out, for pointer and keyboard alike. If Escape should close it,
    set AppLayout::escape(Escape { modal, .. }) (0.57).
[ ] Destructive actions have explicit labels; they are not triggered
    by backdrop click alone.
[ ] If initial keyboard focus inside the dialog matters, give the control
    an id (snora::control(…).id(…), or design::control, 0.61) and return
    iced's operation::focus(id) from the update that opens the dialog.
    The task runs on the new view, so it reaches the dialog. Snora
    bounds Tab to the modal and the toasts, but does not choose where
    focus starts: without this, the first Tab reaches the dialog's first
    control.
```

ABDD is a layout discipline, not a complete accessibility or localization
stack. Snora's contribution is deterministic overlay layering and logical
edge placement. Full accessibility is a shared responsibility with iced
and your application.
