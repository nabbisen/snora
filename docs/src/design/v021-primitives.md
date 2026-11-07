# v0.21 Primitives — Notice, Chip, Progress

These three primitives shipped in v0.21. They are available when the
`design` feature is enabled (`features = ["widgets", "design"]`).

See the dedicated pages for usage details:

- [Notices](notices.md)
- [Chips](chips.md)
- [Progress](progress.md)

---

## Notice

A toned banner that communicates status. Action and dismiss controls are
snora's `control` (RFC-116), reachable from the keyboard from 0.55. App owns
visibility state.

```rust,ignore
use snora::design::{Tokens, Tone, notice::Notice};

Notice::new(&tokens, Tone::Warning, "Disk space low.")
    .title("Storage warning")
    .action("Free space", Message::FreeSpace)
    .dismiss(Message::DismissNotice)
    .render()
```

### Accessibility (RFC-027)

1. **Native primitive:** snora's `control` for action and dismiss (iced's `button` before 0.55).
2. **Keyboard reachable:** yes, from 0.55: Tab reaches each (once the application routes it with `snora::keyboard::traverse`), and Enter and Space press it. Dismissing moves focus nowhere: the next Tab starts from the top (RFC-116's long-term plan). *(Corrected 2026-11-07: up to 0.54 these were iced buttons, which take no keyboard input; this page had said "keyboard-reachable".)*
3. **Focus visible:** yes, a ring from `Tokens::focus`, ≥ 3:1.
4. **Semantic limitation:** no ARIA role exposed; visual tone only.
5. **Example:** design workbench notice section.

---

## Filter chip

A toggle chip built on snora's `control` (RFC-116; iced's `button` before 0.55).

```rust,ignore
use snora::design::chip;

chip::filter(&tokens, "Draft", self.show_drafts, Message::ToggleDrafts)
chip::removable(&tokens, "Tag: Rust", true, Message::Toggle, Message::Remove)
```

### Accessibility (RFC-027)

1. **Native primitive:** snora's `control`; a removable chip has two, the label and the remove button.
2. **Keyboard reachable:** yes, from 0.55, as for the notice. A chip keeps its focus when pressing it toggles it. *(Corrected 2026-11-07: up to 0.54 it was an iced button, which takes no keyboard input.)*
3. **Focus visible:** yes, a ring from `Tokens::focus`, ≥ 3:1, drawn above the neighbouring remove button.
4. **Semantic limitation:** selected state communicated via color — document
   if color-alone would be a barrier in the application context.
5. **Example:** design workbench chip section.

---

## Progress

Backed by `iced::widget::progress_bar`. Display-only; emits no events.
Pass `None` for indeterminate state — iced 0.14 has no native indeterminate
animation; renders as 0% with a "…" suffix.

```rust,ignore
use snora::design::{Tone, progress};

progress::row(&tokens, "Indexing files", Some(0.6), Tone::Accent)
progress::card(&tokens, "Syncing", None, Tone::Info)
```

Note: the `Tone` parameter was added beyond the RFC-032 sketch (which
omitted it) to allow toned progress bars consistent with notices and chips.
It is `Tone::Accent` for neutral progress.

### Accessibility (RFC-027)

1. **Native primitive:** `iced::widget::progress_bar`.
2. **Keyboard reachable:** N/A (display only).
3. **Focus visible:** N/A.
4. **Semantic limitation:** no ARIA progressbar role in iced 0.14.
5. **Example:** design workbench progress section.

### Visual fit checklist

- Bar fill at 0%, 60%, and 100%.
- Indeterminate ("…") label visible.
- Tone colors at high contrast.
