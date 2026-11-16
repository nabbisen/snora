# Migration 0.63 → 0.64

> **A breaking release, on purpose: one migration instead of two.** The F6 types deprecated in
> 0.61 are removed, and every snora widget with a reading order takes the direction, so it can
> mirror under right-to-left. Every left-to-right frame is unchanged.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `snora::focus::next_zone`, `FocusZone`, `ZonePresence` | **Removed** (deprecated since 0.61) | Two lines of `focus_zone` (§1) |
| `chip::removable`, `removable_with_tooltip`, `progress::row`, `progress::card`, `app_footer` (both), `render_menu`, `Notice::new` | **A required `direction` as the last argument** | Pass the direction you already pass to `AppLayout::direction` (§2) |
| `Shortcuts::routes(direction)` | Takes no argument | Delete the argument |
| A right-to-left layout | Those widgets, a context menu, and snora's own text now mirror | Retake RTL screenshot baselines (§3) |
| Anything else | Nothing | Nothing |

## 1. The F6 types are removed (RFC-126)

```rust,ignore
// Before, deprecated since 0.61:
// if let Some(zone) = snora::focus::next_zone(self.focus_zone, cycle, presence, modal, menu) { … }

// Now: snora moves focus, from wherever focus is. No state of yours.
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    return snora::keyboard::focus_zone(cycle);
}
```

Delete the `focus_zone` field and the presence helper you kept for it. `Cycle`, `cycle_zones` and
`focus_zone` are unchanged ([0.60 → 0.61](migration-0.60-to-0.61.md#1-f6-moves-focus-rfc-126)).

## 2. The direction, everywhere it matters (RFC-133)

| Was | Now |
|---|---|
| `design::chip::removable(t, label, selected, on_toggle, on_remove)` | `… , on_remove, direction)` |
| `design::chip::removable_with_tooltip(t, label, selected, on_toggle, on_remove, tip)` | `… , tip, direction)` |
| `design::progress::row(t, label, value, tone)` | `… , tone, direction)` |
| `design::progress::card(t, label, value, tone)` | `… , tone, direction)` |
| `widget::app_footer(content)` | `app_footer(content, direction)` |
| `design::widget::app_footer(t, content)` | `app_footer(t, content, direction)` |
| `widget::render_menu(menu, &on_action, is_active)` | `… , is_active, direction)` |
| `design::notice::Notice::new(t, tone, body)` | `Notice::new(t, tone, body, direction)` |

- **Required, with no default.** A left-to-right default would be silently wrong in every RTL
  application, while a compile error shows you each place once.
- **A call with no direction of its own passes `LayoutDirection::Ltr`.** It then draws exactly
  what it drew before: every left-to-right frame of the eight is unchanged, 756 of them, compared to
  the pixel.
- **Every snora widget with a reading order now takes one**, and a test keeps it so
  ([Direction](direction.md#built-in-widgets-that-take-direction)).

## 3. What mirrors now, under right-to-left

| Widget | Under RTL |
|---|---|
| Removable chip | the label, then the × |
| Notice | the accent bar at the start edge, the controls at the end, the text aligned to the start |
| Progress bar | **the fill grows from the right**; the label starts there |
| Footer | its content at the start edge |
| A bare `render_menu`, **and so the engine's context menu** | the icon before the label; the items at the menu's start edge |
| **Snora's own text**, in the notice, the progress label, the footer and **the toast's title and message** | aligned to the start edge of its box |

**The progress bar is drawn by snora now,** not by iced's `progress_bar`, which has no direction.
Under left-to-right it is pixel-identical to iced's, on both renderers. **Retake the RTL screenshot
baselines** of each widget above.

## What did not change

- Every left-to-right frame, to the pixel.
- Tab, F6 and the keyboard, under both directions (0.63's reading order stays).
- Every colour, and every contrast guarantee.

## If you are jumping more than one minor

Read [0.62 → 0.63](migration-0.62-to-0.63.md) first: `row_dir` returns a `LogicalRow` there.
