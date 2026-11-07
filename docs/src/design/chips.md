# Chips

Chips are compact toggle controls for filtering or categorizing content.
The application owns selection state.

## Variants

### `chip::filter`

A simple toggle chip. Shows a solid accent background with `accent_text` foreground when selected (WCAG AA ≥4.5:1 across all presets).

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

- Both variants are snora controls (RFC-116; iced buttons before 0.55): from 0.55, Tab reaches the chip, and a removable chip's remove button, once the application routes it with `snora::keyboard::traverse`. Enter and Space press them, and a chip keeps its focus when pressing it toggles it.
- Selected state is communicated via solid accent background + contrast-tested `accent_text` foreground (WCAG AA ≥4.5:1 across all presets, verified by automated tests).
  If color-alone would be a barrier in your application context, add a
  checkmark glyph to the label.
- A focus ring from `Tokens::focus` shows focus, drawn above the neighbouring remove button so it is never partly hidden.
