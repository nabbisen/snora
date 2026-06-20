# Recipe: Background task card

**Promotion status:** Recipe — v0.23

---

## 1. Purpose

Show the status of a running background task — its name, current progress,
and optional cancellation or pause control — inside a `card::surface`.

## 2. When to use

- Indexing, syncing, importing, exporting, or processing a batch of files.
- Any operation the user triggered that takes more than a few seconds and
  has quantifiable progress.
- Situations where the user might want to cancel or pause the task.

## 3. When not to use

- When the operation is blocking and the UI is entirely disabled — use a
  `dialog` with a progress indicator instead so the modal dim signals that
  nothing else is interactive.
- When progress cannot be quantified at all and the only feedback needed is
  "in progress" — consider an inline spinner icon or a simple text status
  label rather than this recipe.

## 4. Data the app owns

- Task name and optional subtitle (app-specific).
- Current progress value (`Option<f32>`: `Some(0.0..=1.0)` or `None` for
  indeterminate).
- Whether the cancel button is shown and the message it emits.
- Whether the task is paused (paused state is app-controlled).
- Tone for the progress bar.

## 5. Snora primitives used

- `snora::design::card::surface` — outer container.
- `snora::design::progress::row` — the progress bar and label.
- `snora::design::button::ghost` (optional cancel/pause).
- `snora::design::chip::filter` (optional pause toggle).

## 6. Accessibility notes

- Cancel and pause controls are `iced::widget::button` — keyboard reachable.
- Progress bar uses `iced::widget::progress_bar` — the percentage is also
  rendered as a text label so the value is not color-only.
- For indeterminate tasks, the "…" suffix is the only signal that the task
  is still running. If a more explicit indication is needed, add a text
  status field (e.g. "Indexing 1 042 of ? files") from the app.

## 7. Code example

```rust,ignore
use snora::design::{Tokens, Tone, card, button, progress};
use iced::widget::{column, row, text};
use iced::Element;

fn background_task_card<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    task_name: &'a str,
    value: Option<f32>,
    tone: Tone,
    on_cancel: Option<Message>,
) -> Element<'a, Message> {
    let t = tokens;

    let mut controls = row![].spacing(t.spacing.sm);
    if let Some(msg) = on_cancel {
        controls = controls.push(button::ghost(t, "Cancel", Some(msg)));
    }

    let inner = column![
        text(task_name)
            .size(snora::design::style::text::label_size(t))
            .color(snora::design::style::color::to_iced_color(t.palette.text_primary)),
        progress::row(t, task_name, value, tone),
        controls,
    ]
    .spacing(t.spacing.sm);

    card::surface(t, inner)
}
```

## 8. Customization points

- Add a second `progress::row` for sub-task progress below the main bar.
- Use `chip::filter` to render a pause toggle alongside the cancel button.
- Switch to `card::raised` if the task card lives in a background panel
  rather than in the main body area.
- Set `tone = Tone::Warning` if the task is nearing a quota limit.

## 9. Promotion status

**Recipe.** The pattern has three optional parameters (cancel, pause, tone)
that differ across apps. A stable helper would need to expose at least six
parameters, making it larger than the ~100 ELOC guideline. Keeping it as a
recipe lets each app tailor the controls.
