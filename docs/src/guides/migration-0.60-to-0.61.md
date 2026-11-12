# Migration 0.60 → 0.61

> **F6 moves focus between the frame's zones,** and **a snora control can be given an id.**
> Nothing breaks. Three zone types are deprecated.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `snora::focus::next_zone`, `FocusZone`, `ZonePresence` (F6) | **Deprecated** (warns, still works, removed in 0.64.0). They could not move focus | Replace them with `focus_zone`, in two lines (§1) |
| F6 not wired at all | Nothing, until you route it | Optional: route F6 as you route Tab (§1) |
| A dialog whose first focus matters, built with snora's controls | You can now choose it | `.id()` and `focus(id)` (§2) |
| Anything else | Nothing | Nothing |

## 1. F6 moves focus (RFC-126)

```rust,ignore
// Before (deprecated): a decision your application had to act on, and could not
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    if let Some(zone) = snora::focus::next_zone(self.focus_zone, cycle, self.zone_presence(),
                                                self.modal_open(), self.menu_open()) {
        self.focus_zone = zone;
    }
}

// After: snora moves focus, from wherever focus is. No state of yours.
if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
    return snora::keyboard::focus_zone(cycle);
}
```

Delete the `focus_zone` field and `zone_presence()` you kept for it.

- **F6 / Shift+F6** go to the first control of the next / previous zone (header, sidebar, body,
  footer), **counted from the zone that holds focus**. After a Tab or a click, that is the zone
  the reader is in.
- **A zone with nothing to focus is skipped.** The order wraps, and does not mirror under RTL.
- **With focus in no zone** (nothing focused, a toast, a context menu): F6 goes to the first zone,
  Shift+F6 to the last.
- **F6 lands only on a zone's own controls, never inside an open menu.**
- **Under a dialog, a sheet, or a menu with its backdrop, F6 does nothing.**
- **Held, it keeps moving,** one zone per repeat. It never presses anything.

**Why the old way did nothing:** a snora control's id could not be set, and iced 0.14 cannot focus
"the first control inside" a region without its `advanced` feature. So the application could
decide the next zone but not move focus into it. The current zone was the application's own
variable, which went stale after a Tab, and an open menu with its backdrop was ignored.

## 2. Where focus starts in a dialog (RFC-127)

```rust,ignore
snora::design::control(&t, text("Cancel")).id("confirm-cancel").on_press(Message::Cancel)
    .style(move |_, status| style::button::secondary(&tok, status))

// in the update that opens the dialog
return iced::widget::operation::focus("confirm-cancel");
```

- The task runs on the new view, so it reaches the dialog the same update opens.
- An id must be unique in the window.
- An id no reachable control has focuses nothing and leaves nothing focused.

See [Overlays](overlays.md#where-focus-starts-in-a-dialog-061-rfc-127). The `design::button`
variants take no id: build that control with `snora::design::control` and the same style.

## What did not change

- Tab, Enter, Space, Escape and the arrows in a menu.
- `Cycle` and `cycle_zones`.
- Appearance, colours and presets.

## If you are jumping more than one minor

Read [0.59 → 0.60](migration-0.59-to-0.60.md) first: it removed `dismiss_on_escape`.
