# Migration 0.64 → 0.65

> **Additions only: nothing breaks, and nothing you draw changes.** snora now has a checkbox, a
> toggle, a radio group, a select and a slider that the keyboard can reach (RFC-134). iced 0.14's
> own cannot be reached by the keyboard.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| iced's `checkbox`, `toggler`, `radio`, `pick_list` or `slider` | Nothing, but the keyboard cannot reach them | Move to snora's (§1) |
| A checkbox, toggle or radio you built on `snora::control` (the 0.63 advice) | Nothing | Optional: snora's own gives the radio group one Tab stop and arrow keys |
| Anything else | Nothing | Nothing |

## 1. Moving from iced's control: a name and a direction

```rust,ignore
// iced, which the keyboard cannot reach:
iced::widget::checkbox(self.remember).label("Remember me").on_toggle(Message::Remember)
iced::widget::slider(0.0..=100.0, self.volume, Message::Volume)
iced::widget::pick_list(SIZES, self.size, Message::Size)

// snora, from 0.65 (`snora::design::…` takes `&tokens` first):
snora::checkbox(self.remember, direction).label("Remember me").on_toggle(Message::Remember)
snora::slider(0.0..=100.0, self.volume, Message::Volume, direction)
snora::select(SIZES, self.size.as_ref(), Message::Size, direction)
    .opened(self.size_open)
    .on_open_change(Message::SizeOpen)
```

- **`direction` is the last argument,** as for every snora widget with a reading order.
- **A radio group is one control:** `radio_group(options, selected, on_select, direction)`. iced's
  `radio` has no group.
- **The select's open state is yours**, as a header menu's is. Choosing publishes `on_select`
  only, so close it there. Its arrows, its cover and a click outside closing it work inside
  `snora::render`, through your one `on_close_menus`.
- **Slider values are `f32`.**
- **Disabled when:**
  - a checkbox, a toggle or a select has no handler;
  - a radio group or a slider takes `.disabled(true)`.

The keys, and why each control is what it is, are in
[Keyboard § Form controls](keyboard.md#form-controls).

## What did not change

Everything you already draw, in both directions, to the pixel.

## If you are jumping more than one minor

Read [0.63 → 0.64](migration-0.63-to-0.64.md) first: it is the breaking release.
