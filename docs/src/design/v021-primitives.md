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
{{#include ../../../examples/book_snippets/src/design_pages.rs:v021_notice}}
```

### Accessibility (RFC-027)

1. **Native primitive:** snora's `control` for action and dismiss (iced's `button` before 0.55).
2. **Keyboard reachable:** yes, from 0.55: Tab reaches each (once the application routes it with `snora::keyboard::traverse`), and Enter and Space press it. Dismissing moves focus to the nearest control after the notice in its container, or else before it (0.56, RFC-120). *(Corrected 2026-11-07: up to 0.54 these were iced buttons, which take no keyboard input; this page had said "keyboard-reachable".)*
3. **Focus visible:** yes, a ring from `Tokens::focus`, ≥ 3:1.
4. **Semantic limitation:** no ARIA role exposed; visual tone only.
5. **Example:** design workbench notice section.

---

## Filter chip

A toggle chip built on snora's `control` (RFC-116; iced's `button` before 0.55).

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:v021_chips}}
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

Drawn by snora itself since 0.64 (RFC-133), not by `iced::widget::progress_bar`, which has no direction. Display-only; emits no events.
Pass `None` for indeterminate state — iced 0.14 has no native indeterminate
animation; renders as 0% with a "…" suffix.

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:v021_progress}}
```

Note: the `Tone` parameter was added beyond the RFC-032 sketch (which
omitted it) to allow toned progress bars consistent with notices and chips.
It is `Tone::Accent` for neutral progress.

### Accessibility (RFC-027)

1. **Native primitive:** none; snora draws the track and the fill (0.64, RFC-133).
2. **Keyboard reachable:** N/A (display only).
3. **Focus visible:** N/A.
4. **Semantic limitation:** no ARIA progressbar role in iced 0.14.
5. **Example:** design workbench progress section.

### Visual fit checklist

- Bar fill at 0%, 60%, and 100%.
- Indeterminate ("…") label visible.
- Tone colors at high contrast.
