# Migration 0.59 → 0.60

> **Arrow keys, Home and End work inside snora's menus,** and **a covered page gets no
> pointer:** no hover under a dialog, and a wheel over the dim no longer closes it. **One
> breaking change:** `dismiss_on_escape`, deprecated in 0.57, is removed.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `snora::keyboard::dismiss_on_escape` | **Removed** (deprecated since 0.57.0) | `AppLayout::escape`, or a three-line `match` (§1) |
| `app_header`'s menus, or a `render_menu` context menu, through `snora::render` | **↓ ↑ Home End** move between the items (§2) | Nothing |
| A dialog or a sheet with `on_close_modals` | **A wheel over the dim no longer closes it** (§3) | Nothing, unless you relied on that |
| A dialog, a sheet, or a menu with its backdrop, over a page | **The page shows no hover** and no hand cursor while covered (§3) | Retake screenshot baselines taken with the pointer over the page |
| A context menu or `header_menu` with its backdrop, over a scrolling page | **A wheel no longer scrolls the page** beneath (§3) | Nothing |
| Anything else | Nothing | Nothing |

## 1. `dismiss_on_escape` is removed

The rule: **a deprecated item is removed in the third minor after the one that deprecated
it.**

```rust,ignore
// Before (deprecated since 0.57.0)
if let Some(msg) = snora::keyboard::dismiss_on_escape(
    self.dialog.is_some(), self.menu.is_some(),
    Some(Message::CloseModals), Some(Message::CloseMenus), key,
) {
    return self.update(msg);
}

// After: nothing to wire for snora's overlays. Escape publishes the escape message
// if you set one, otherwise the click-outside close message.
let layout = AppLayout::new(body)
    .on_close_modals(Message::CloseModals)
    .on_close_menus(Message::CloseMenus)
    // Optional: Escape alone, for an overlay a stray click must not close.
    .escape(Escape { modal: Some(Message::CancelDelete), menu: None });
```

**For an overlay of your own, outside `AppLayout`,** match the key yourself:

```rust,ignore
if key == Key::Named(Named::Escape) && self.picker_open {
    return self.update(Message::ClosePicker);
}
```

No `#[deprecated]` item remains in snora.

## 2. The keyboard inside an open menu (RFC-123)

| Key | What it does |
|---|---|
| `↓` / `↑` | The next / previous item, wrapping at both ends. Held, it keeps moving |
| `Home` / `End` | The first / last item |
| `↓` / `↑` with nothing focused | The first / last item |

- **Opening a menu still moves no focus.** From a focused trigger, **Enter then ↓** reaches
  the first item.
- **The arrows are the menu's only while focus is on its trigger, on an item, or nowhere.** A
  focused text field, a page control or a toast's × keeps its arrows.
- **An arrow the menu answers is captured,** so your `keyboard::listen()` never receives it.
  If your `update` reacted to ↓ or ↑ while a snora menu was open with nothing focused, it no
  longer sees them.
- **Only the plain keys:** Shift+↓ and Ctrl+Home are left alone.
- **Nothing to wire.** A `render_menu` you draw outside `snora::render` gets no arrows.

See [Menus](menus.md#the-keyboard-inside-an-open-menu-060-rfc-123).

## 3. A covered layer gets no pointer (RFC-125)

> **Whatever the keyboard cannot reach, the pointer cannot reach either:** no hover, no
> pointer shape, no tooltip, no wheel. A press outside (a click or a tap) and Escape close
> what is open. Nothing else does.

| Under… | 0.59 | 0.60 |
|---|---|---|
| a dialog or a sheet with `on_close_modals`: a wheel over the dim | **closed it, once per wheel event** | nothing |
| a dialog, a sheet, a context menu, a `header_menu` with its backdrop: the pointer over a page control | hover and the hand cursor | neither |
| a context menu or a `header_menu` with its backdrop: a wheel | **scrolled the page** | nothing |
| a dialog: a slider dragged when the dialog opened | kept moving | stops; the release still ends the drag |
| a dialog's own card: a wheel | scrolls the card | unchanged |
| an open `app_header` dropdown | no pointer (from 0.59) | unchanged |

**Why the wheel stopped closing dialogs.** A two-finger swipe that strayed off a long
dialog closed it, with whatever the reader had typed. That was never a ruled behaviour:
0.41 asked only that the dim block the wheel. **If your application closed a dialog on a
wheel on purpose,** that close is gone. Give the dialog a visible close control.

## What did not change

- A click or a tap on the dim or a backdrop, and Escape (Law 7).
- Tab, Enter and Space, everywhere.
- Toasts: reachable by pointer and keyboard under every cover (Law 6).
- Colours and presets.

## If you are jumping more than one minor

Read [0.58 → 0.59](migration-0.58-to-0.59.md) first. On 0.56 or earlier, the
[0.56 → 0.57](migration-0.56-to-0.57.md) guide introduced `AppLayout::escape`.
