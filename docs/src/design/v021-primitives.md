# v0.21 Primitives — Notice, Chip, Progress

This page documents the planned v0.21 shallow primitives: **notice**,
**filter chip**, and **progress**. None of these ship in v0.20; they are
listed here so the design is visible and reviewable before implementation.

**Precondition:** These must not be started until:

1. The v0.20 style bridge and button/card helpers have been lightly dogfooded.
2. The RFC-027 semantic construction policy is applied to each primitive.
3. `api-governance.md` promotion criteria are understood by the implementer.

---

## Notice

A themed row or card that communicates status to the user. Has an optional
title, a body message, an optional action button, and an optional dismiss
button. App owns visibility state.

### Proposed API

```rust,ignore
Notice::new(Tone::Info, "Background sync is running.")
    .title("Syncing")
    .action("View details", Message::OpenSyncDetails)
    .dismiss(Message::DismissNotice)
    .render(&tokens)
```

### Internal model

```rust,ignore
pub struct NoticeSpec<Message> {
    pub tone: Tone,           // Info / Success / Warning / Danger
    pub title: Option<String>,
    pub body: String,
    pub action: Option<(String, Message)>,
    pub dismiss: Option<Message>,
}
```

### Events

- Action button emits the app-provided message.
- Dismiss button emits the app-provided message.
- The app is solely responsible for showing and hiding the notice.

### Accessibility requirements (RFC-027)

- Action and dismiss controls must be `iced::widget::button`, not
  `mouse_area` over a container.
- Tone color pairs (`success_text on success`, etc.) are already verified
  by the contrast tests for all four presets.
- Text remains readable when `Notice` is displayed at high contrast.

---

## Filter chip

A compact toggle control for filtering or categorizing. App owns the filter
state. Interactive chips must use `iced::widget::button` semantics.

### Proposed API

```rust,ignore
chip::filter(&tokens, "Draft", self.show_drafts, Message::ToggleDrafts)
chip::removable(&tokens, "Draft", self.show_drafts, Message::Toggle, Message::Remove)
```

### Internal model

```rust,ignore
pub struct ChipSpec<Message> {
    pub label: String,
    pub selected: bool,
    pub on_toggle: Option<Message>,
    pub on_remove: Option<Message>,
}
```

### Accessibility requirements (RFC-027)

- Use `iced::widget::button` for the chip body.
- Do not use a container with `mouse_area` only.
- Selected state communicated via visual styling (background, border);
  document color-alone limitation.

---

## Progress

A progress indicator that accepts a determinate ratio (`0.0..=1.0`) or
an indeterminate signal. App owns the task, the progress value, and any
cancellation semantics.

### Proposed API

```rust,ignore
progress::row(&tokens, "Indexing files", Some(0.6))
progress::indeterminate(&tokens, "Loading…")
progress::card(&tokens, "Indexing files", Some(0.6))
```

### Value model

```rust,ignore
pub enum ProgressValue {
    Determinate(f32),   // 0.0..=1.0
    Indeterminate,
}
```

### Accessibility requirements (RFC-027)

- Use `iced::widget::progress_bar` where available and appropriate.
- Indeterminate state must still be visually meaningful (animated or
  labeled, not a frozen bar at 0%).
- Progress percentage must be readable as text alongside the bar.

---

## Visual-fit checklist for v0.21

When adding these to the design workbench, inspect:

- Notice: tone color at all four presets; action/dismiss button alignment;
  text wrapping in narrow containers.
- Chip: selected vs unselected visual difference at high contrast;
  tap target size (`>= 24px` height).
- Progress: bar fill at 0%, 50%, and 100%; indeterminate animation
  visibility; text label legibility.

See [Accessibility checklist](../contributing/accessibility-checklist.md)
for the full gate.
