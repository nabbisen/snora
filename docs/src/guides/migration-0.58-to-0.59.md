# Migration 0.58 → 0.59

> **`app_header`'s dropdown is fixed,** and now floats over the page. **One breaking
> change:** the three toast-hover items deprecated in 0.56 are removed.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `app_header` with menu items, wired as `menus.md` showed (`on_close_menus` and `header_menu(space())`) | **Fixed.** A click on an item now presses it (it closed the menu instead, at least since 0.54.0), and the keyboard reaches the items (it could not from 0.55). The dropdown floats over the page; opening it no longer pushes the page down | Nothing. You may drop `header_menu(space())`: it is harmless, and no longer needed (§1) |
| `app_header` with an empty `items` list and your own content in `header_menu`, as the `starter` does | Nothing | Nothing |
| `AppLayout::on_toast_hover`, `ToastHover`, `Holds::hover` | **Removed** (deprecated since 0.56.0) | `on_toast_attention`, `ToastAttention`, `Holds::attend` (§2) |
| Screenshot baselines with an open header menu | The dropdown is a card of its own below the trigger, and the header keeps its height | Retake them |
| Anything else | Nothing | Nothing |

## 1. The header dropdown (RFC-124)

**What was wrong.** `app_header` drew its dropdown inside the header. To close the menu on
a click outside, `menus.md` told you to put an empty `Space` in `header_menu`, which
installed a full-window backdrop over the header too. As a result:
- **a click on an item hit the backdrop and closed the menu,** at least since 0.54.0, the
  oldest release measured;
- **from 0.55 the keyboard could not reach the items,** because the page under a backdrop
  is out of the keyboard's reach (RFC-119);
- **opening a menu made the header taller** and pushed the page down.

The book's explanation for the design, that iced 0.14 cannot position anything above the
page, was false. It is corrected.

**What 0.59 does.**
- The dropdown is an overlay anchored below its trigger, at the trigger's start edge (the
  end edge under RTL), and kept inside the window.
- **Opening it moves nothing.** A dropdown taller than the window scrolls, and keeps the
  focused item in view.
- **While it is open, the whole page beneath it is covered** for the pointer and the
  keyboard: no click, no key, and no hover.
- **A click outside, on the trigger too, or Escape** publishes your `on_close_menus`, or
  your `AppLayout::escape` menu message for Escape if you set one.
- **Its items press,** by pointer and by keyboard.

**Your code:**

```rust,ignore
// Before (0.58 and earlier)
let mut layout = AppLayout::new(body).header(header).on_close_menus(Message::CloseMenus);
if state.active_menu.is_some() {
    layout = layout.header_menu(iced::widget::space().into());
}

// After: snora sees the open dropdown by itself
let layout = AppLayout::new(body).header(header).on_close_menus(Message::CloseMenus);
```

Keeping the old lines does no harm. `header_menu` stays for menu content of your own.

## 2. The removals (deprecated since 0.56.0)

The rule: **a deprecated item is removed in the third minor after the one that deprecated
it.**

```rust,ignore
// Before
Message::ToastHover(id, crossing) => self.holds.hover(id, crossing, &mut self.toasts, now),
layout.on_toast_hover(Message::ToastHover)

// After
Message::ToastAttention(id, attention) => self.holds.attend(id, attention, &mut self.toasts, now),
layout.on_toast_attention(Message::ToastAttention)
```

`ToastHover::Entered` is `ToastAttention::PointerEntered`, and `Left` is `PointerLeft`. A
toast now also waits while the keyboard is on its ×.

**Still deprecated, and removed in 0.60.0:** `snora::keyboard::dismiss_on_escape`. Use
`AppLayout::escape`.

## What did not change

- Context menus, `render_menu` on its own, and every other control.
- Colours and presets.

## If you are jumping more than one minor

Read [0.57 → 0.58](migration-0.57-to-0.58.md) first. On 0.55 or earlier, the
[0.55 → 0.56](migration-0.55-to-0.56.md) guide introduced `on_toast_attention`.
