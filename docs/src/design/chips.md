# Chips

Chips are compact toggle controls for filtering or categorizing content.
The application owns selection state.

## Variants

### `chip::filter`

A simple toggle chip. Shows a tinted accent background when selected.

```rust,ignore
use snora::design::chip;

chip::filter(&tokens, "Draft",   self.show_drafts, Message::ToggleDrafts)
chip::filter(&tokens, "Active",  self.show_active, Message::ToggleActive)
```

Pass `None` (or `Option::<Message>::None`) to render a disabled chip.

### `chip::removable`

A chip with a separate remove (×) button. Both the label and the × are
independent `iced::widget::button` controls.

```rust,ignore
chip::removable(
    &tokens,
    "Tag: Rust",
    self.rust_selected,
    Message::ToggleRust,
    Message::RemoveRust,
)
```

Pass `None` for either `on_toggle` or `on_remove` to disable that control
independently.

## Managing selection state

Chips never store selection state internally. Drive them from your app:

```rust,ignore
for tag in &self.active_tags {
    chip::filter(&tokens, tag.as_str(), true, Message::RemoveTag(tag.clone()))
}
```

## Accessibility

- Both variants use `iced::widget::button`.
- Selected state is communicated via color (tinted background + accent text).
  If color-alone would be a barrier in your application context, add a
  checkmark glyph to the label.
- Custom focus ring not rendered in iced 0.14 — documented limitation.
