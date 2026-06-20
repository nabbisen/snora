# RFC 032 — Notice, Chip, and Progress Primitives

**Status.** Proposed
**Tracks.** Snora Design System migration; v0.21+.
**Touches.** `snora-widgets::design` future primitives.

## Summary

This RFC defines v0.21 candidates: notice, filter chip, and progress primitives.

These are not v0.20 foundation requirements.

## Goals

- Add behavior-light notice primitive.
- Add behavior-light filter chip primitive.
- Add progress row/card primitive.
- Preserve app-owned behavior.
- Apply semantic construction policy.

## Non-goals

- No form validation.
- No search/filter engine.
- No background task manager.
- No notification center.

## Notice primitive

Public candidate:

```rust
Notice::new(Tone::Info, body)
    .title("Setup needed")
    .action("Open settings", Message::OpenSettings)
    .dismiss(Message::DismissNotice)
```

Internal model:

```rust
pub struct NoticeSpec<Message> {
    pub tone: Tone,
    pub title: Option<String>,
    pub body: String,
    pub action: Option<NoticeAction<Message>>,
    pub dismiss: Option<Message>,
}
```

Events:

- action emits app message;
- dismiss emits app message;
- app owns visibility.

## Filter chip

Candidate:

```rust
chip::filter(tokens, label, selected, Message::Toggle)
chip::removable(tokens, label, selected, Message::Toggle, Message::Remove)
```

Internal model:

```rust
pub struct ChipSpec<Message> {
    pub label: String,
    pub selected: bool,
    pub on_toggle: Option<Message>,
    pub on_remove: Option<Message>,
}
```

App owns filter state.

## Progress

Candidate:

```rust
progress::row(tokens, "Indexing files", Some(0.6))
progress::card(tokens, "Indexing files", Some(0.6))
```

Model:

```rust
pub enum ProgressValue {
    Determinate(f32),
    Indeterminate,
}
```

App owns task state, progress calculation, and cancellation semantics.

## Accessibility

- Notice actions/dismiss controls are real controls.
- Interactive chips are button-like where possible.
- Progress uses iced progress primitive where possible.
- Limitations documented.

## Data lifecycle

```text
app state
  -> primitive helper
  -> iced elements
  -> user interaction
  -> app message
  -> app updates state
```

## Acceptance criteria

- primitives remain shallow;
- app owns behavior;
- semantic review complete;
- high-contrast examples exist.
