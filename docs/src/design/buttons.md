# Buttons

Snora Design provides five semantic button variants via
`snora::design::button`. Each is snora's own `control` (RFC-116) with Snora
Design token styling applied, so each is reachable from the keyboard.

## Variants

| Function | Semantic use |
|---|---|
| `button::primary` | Strongest call to action; use once per surface |
| `button::secondary` | Secondary action alongside a primary button |
| `button::ghost` | Tertiary / low-emphasis **action**: it does something where the reader is |
| `button::link` | Takes the reader **somewhere**: another view, a document, a chooser (0.55, RFC-117) |
| `button::danger` | Irreversible actions — delete, revoke, reset |

## Basic usage

```rust,ignore
use snora::design::{Tokens, button};

// In your app state, store tokens:
struct App { tokens: Tokens }

// In view():
let t = &self.tokens;

button::primary(t, "Save", Message::Save)
button::secondary(t, "Cancel", Message::Cancel)
button::ghost(t, "Skip", Message::Skip)
button::link(t, "Choose another folder", Message::ChooseFolder)
button::danger(t, "Delete", Message::Delete)
```

## Link or action

> A **link** takes the reader somewhere: another view, a document, a chooser.
> An **action** does something where the reader already is: Cancel, Skip,
> Save. Use `link` for the first and `ghost` for the second.

"Choose another folder" opens a chooser, so it is a link. "Skip" acts where
the reader is, so it is a `ghost`.

**Why a link is underlined.** Older readers do not read coloured text as
clickable, and colour is the cue that fails first as eyesight changes (WCAG
1.4.1, Use of Color). `link` has `ghost`'s colours, and an underline in the
label's colour in every state (rest, hover, pressed), **under every line** if
the label wraps.

**The underline's rule.** Let a unit *u* be 1/16 em of the label's size,
rounded to the nearest whole pixel (halves up), and never under 1 px.

> An underline is *u* thick, and its top edge is *u* below its line's box;
> snapping to the pixel grid never moves it up toward the text. Every wrapped
> line has its own. A link's lines are 4 *u* taller than plain text's, with the
> text centred in each.

At the default label size (14 px), *u* is 1 px. The extra line spacing keeps
an underline clear of the next line's accents and tall letters. It was
measured at scale 1, 1.5 and 2, in three fonts, at 12 to 32 px (RFC-117).
**The link is as tall as a `ghost` with the same label**, because its padding
gives the extra height back, except:
- for a label of 40 px or more, where there is no padding left to give;
- at 24 px or more on the two high-contrast presets, where it is 1 to 2 px
  taller, to keep their wider focus ring off the underline.

The gap is measured from the line's box, which is all iced reports. A font
whose descenders reach past its own line box comes closer to the underline
than the default fonts do.

**There is no disabled link, and no `link_maybe`.** A dimmed, underlined
phrase still says "you can go here", and then does nothing. When the
destination is unavailable, show plain text, or nothing.

## Disabled state

Each variant except `link` has a `*_maybe` counterpart that accepts
`Option<Message>`. Passing `None` produces a disabled button. A disabled button
is not a focus target: Tab skips it.

```rust,ignore
button::primary_maybe(t, "Save", self.can_save.then_some(Message::Save))
button::danger_maybe(t, "Delete", self.can_delete.then_some(Message::Delete))
```

## Using the raw style bridge

If you need finer control — custom padding, width, child element — use the
style bridge directly:

```rust,ignore
use snora::design::style;
use iced::widget::button;

let t = &self.tokens;
let tok = t.clone();           // clone into the closure

button(my_custom_content)
    .on_press(Message::Save)
    .style(move |_theme, status| style::button::primary(&tok, status))
```

**That is iced's `button`, which is pointer and touch only** in iced 0.14. For
content of your own, use **`snora::design::control`** (0.58, RFC-122). It takes the
same builder, so only the name changes, and it is keyboard-reachable, with a ring from
`Tokens::focus` and the floor from `Tokens::targets.min`:

```rust,ignore
snora::design::control(&self.tokens, my_custom_content)
    .on_press(Message::Save)
    .style(move |_theme, status| style::button::primary(&tok, status))
```

Without `.style(…)` it looks like `ghost`, so content of your own does not look like a
call to action unless you say so. **`.id(…)`** (0.61) lets iced's `operation::focus(id)` focus it,
for a dialog's starting focus say ([Overlays](../guides/overlays.md#where-focus-starts-in-a-dialog-061-rfc-127)).
The variants above take no id: when you need one, build the button this way with the same style
function, and draw its label at the label size, `text(label).size(t.typography.label.size)`. It then
draws exactly as the variant does. The padding is already the same. On the engine path, `snora::control(content)` is the
same, with iced's default look and the theme's ring.

## Accessibility notes

- **Keyboard (0.55, RFC-116).** Every variant is a focus target. Enter and Space
  press the focused one and publish the same message a click does. **Tab moves
  between them once your application routes the key**, in one line of `update`:
  `snora::keyboard::traverse(key, modifiers)`. See
  [Accessibility](../guides/accessibility.md). *(Corrected 2026-11-04: before
  0.55 they were iced buttons, which take no keyboard input; this page once said
  they inherited Enter and Space.)*
- **Focus ring.** It is drawn from `Tokens::focus` (width, offset, colour), and
  clears 3:1 against what it sits on in every preset. Where the token's colour
  would not, it is black or white instead. It is drawn above neighbouring
  controls, so a neighbour cannot hide part of it.
- **Target size (RFC-115).** A button is at least
  `max(24 px, 1.5 × its label, Tokens::targets.min)` in both directions.
  `targets.min` is 24 in every preset; raise it to grow every control.
- The `danger` variant uses `palette.danger_text` on `palette.danger`; this
  pair is mandatory and tested by `cargo test -p snora-design`.
- Disabled buttons pass the WCAG 1.4.3 exception for low-contrast states.

See [Semantic accessibility](../contributing/semantic-accessibility.md) for
the full policy.
