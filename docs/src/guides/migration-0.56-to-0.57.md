# Migration 0.56 → 0.57

> **Opt-in keyboard shortcuts** for snora's tabs, sidebar and breadcrumbs, each doing
> exactly what a click does, and **an Escape message of its own** for each overlay.
> `snora-control` is renamed `snora-interaction`. **One breaking change:** two items
> deprecated since 0.54 are removed.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `snora::toast::sweep_expired` | **Removed** (deprecated since 0.54.0) | `self.holds.sweep(&mut self.toasts, now)` (§1) |
| `ToastLifetime::DEFAULT` | **Removed** (deprecated since 0.54.0) | `ToastLifetime::reading_time(title, message)`, or `ToastLifetime::seconds(n)` for a fixed time (§1) |
| `snora::keyboard::dismiss_on_escape` | **Deprecated**: warns, still works, removed in 0.60.0 | `AppLayout::escape` (§3) |
| `Ctrl+Tab`, `Alt+Tab` or `Ctrl+Shift+Tab` reaching `traverse` | No longer moves focus as Tab does: `traverse` claims plain Tab and Shift+Tab only | Nothing |
| `snora-control` in your `Cargo.toml` | **Renamed** `snora-interaction`; `snora-control` 0.57.0 is an empty notice | Depend on `snora` instead, as always intended (§4) |
| Nothing above | Nothing changes until you set `shortcuts` or `escape` | Nothing |

## 1. The two removals

The rule, from this release on: **a deprecated item is removed in the third minor after
the one that deprecated it**, and its removal version is named from the start (the
[versioning policy](../contributing/versioning-policy.md#deprecation-bridges)).

```rust,ignore
// Before (deprecated since 0.54.0, removed in 0.57.0)
snora::toast::sweep_expired(&mut self.toasts, Instant::now());
let lifetime = ToastLifetime::DEFAULT;

// After
self.holds.sweep(&mut self.toasts, Instant::now()); // `holds: snora::toast::Holds` on your state
let lifetime = ToastLifetime::reading_time(title, message); // what `Toast::new` uses
```

`Holds::sweep` behaves as `sweep_expired` did for a toast no one is attending to, and
keeps one the reader is attending to (0.54, 0.56).

**Still deprecated, and removed in 0.59.0:** `on_toast_hover`, `ToastHover` and
`Holds::hover` (0.56). Their notes now say so.

## 2. Shortcuts, opt-in (RFC-116 R-2)

```rust,ignore
use snora::keyboard::{Digits, Shortcuts};

let layout = snora::AppLayout::new(body).shortcuts(
    Shortcuts::new()
        .side_bar(&side_bar)
        .tabs(&tab_bar, &on_tab_action)
        .crumbs(&crumbs, &on_crumb_action)
        .digits(Digits::Tabs), // the digit keys go to tabs, or to `Digits::SideBar`
);
```

| Windows, Linux | macOS | Does |
|---|---|---|
| `Ctrl+Tab` / `Ctrl+Shift+Tab`, `Ctrl+PgDn` / `Ctrl+PgUp` | `Control+Tab` / `Control+Shift+Tab`, `Shift+Cmd+]` / `[` | next / previous tab, wrapping |
| `Ctrl+1`…`9` | `Cmd+1`…`9` | tab or sidebar item *n*, as you chose; `9` is the last |
| `Alt+↑` | `Cmd+↑` | the breadcrumb's parent |

**The rules:**
- **A shortcut does exactly what a click does:** the same message, focus taken away, and
  Tab carrying on from that control.
- **A focused control that uses the key keeps it,** and a text field keeps every arrow key.
- **Every shortcut needs a modifier.**
- **Shortcuts are silent under a dialog, a sheet, or a menu with its backdrop.**
- **The digits are the number row's physical keys,** so every keyboard layout works.

**The macOS keys have not yet been checked on a Mac.** The full table and rules are in the
[Keyboard map](keyboard.md).

## 3. Escape: a message of its own

```rust,ignore
// Before: deprecated since 0.57.0, removed in 0.60.0
if let Some(msg) = snora::keyboard::dismiss_on_escape(modal_open, menu_open,
        Some(Message::CloseModals), Some(Message::CloseMenus), key) { … }

// After: nothing in `update`
let layout = layout.escape(snora::keyboard::Escape {
    modal: Some(Message::CancelDelete), // Escape alone; a stray click does not close it
    menu: None,
});
```

> **Escape publishes the overlay's escape message if one is set, and otherwise its
> click-outside close message.**

If you already set `on_close_modals` and `on_close_menus`, Escape already worked, from 0.55.
Set `escape` only for an overlay that must close on Escape but not on a click outside. The
named fields replace `dismiss_on_escape`'s two `bool`s and two `Option`s in a row, which
were easy to pass in the wrong order.

## 4. `snora-control` is now `snora-interaction`

The crate that holds snora's focusable control, focus, Escape and covered layers is renamed.
Beside `snora-widgets`, "control" read as another word for widget.
- **Applications never needed it.** `snora` brings it, and nothing in your code names it.
- **`snora-control` 0.57.0 is its last release, and an empty notice.** 0.55.0 and 0.56.0
  stay published and are never yanked, so existing lockfiles keep working.
- **If your `Cargo.toml` names `snora-control`,** remove it and depend on `snora`.

## What did not change

- Appearance, sizes, colours and presets.
- Tab, Enter, Space, the overlays, and focus after a removal behave as in 0.56.

## If you are jumping more than one minor

Read [0.55 → 0.56](migration-0.55-to-0.56.md) first. On 0.53 or earlier, you may still
call `sweep_expired`: §1 is the replacement.
