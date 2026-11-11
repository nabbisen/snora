# Keyboard map

Every key snora answers, the rules behind them, and what your application wires. The
designs are RFC-116 (keyboard access and shortcuts), RFC-119 (overlays), RFC-120 (focus
after a removal) and RFC-121 (a click does not focus).

## What your application wires

iced binds no Tab key, and snora installs no subscription. Two things are yours:

```rust,ignore
// subscription: hand snora the keys
iced::keyboard::listen().map(|event| match event {
    iced::keyboard::Event::KeyPressed { key, modifiers, .. } => Message::KeyPressed(key, modifiers),
    _ => Message::Ignored,
})

// update: Tab and Shift+Tab move focus
Message::KeyPressed(key, modifiers) => {
    if let Some(task) = snora::keyboard::traverse(key, modifiers) {
        return task;
    }
}

// view: the shortcuts you want, from the same data your widgets get (all opt-in)
use snora::keyboard::{Digits, Escape, Shortcuts};

let shortcuts = Shortcuts::new()
    .side_bar(&side_bar)
    .tabs(&tab_bar, &on_tab_action)
    .crumbs(&crumbs, &on_crumb_action)
    .digits(Digits::Tabs); // the digit keys go to tabs, or to `Digits::SideBar`; one, never both

let layout = snora::AppLayout::new(body)
    .shortcuts(shortcuts)
    .escape(Escape { modal: Some(Message::CloseDialog), menu: None });
```

A route you leave out answers nothing. Without `.digits(…)`, no digit key does anything.

## The keys

| Keys (Windows, Linux) | Keys (macOS) | What it does | Needs |
|---|---|---|---|
| `Tab` / `Shift+Tab` | the same | Moves focus to the next / previous control. **Past the last, focus leaves the group**: it does not wrap | `traverse` |
| `Enter`, `Space` | the same | Presses the focused control, publishing exactly what a click publishes. **Held, it presses once** | nothing |
| `Escape` | the same | Closes the open overlay (a dialog, a sheet, then a menu) | a close message (below) |
| `↓` / `↑`, `Home` / `End` | the same | **Inside an open snora menu:** the next / previous item (wrapping), the first / last. With nothing focused, ↓ is the first item and ↑ the last ([Menus](menus.md#the-keyboard-inside-an-open-menu-060-rfc-123)) | nothing (0.60) |
| `Ctrl+Tab` / `Ctrl+Shift+Tab`, and `Ctrl+PgDn` / `Ctrl+PgUp` | `Control+Tab` / `Control+Shift+Tab`, and `Shift+Cmd+]` / `Shift+Cmd+[` | Next / previous **tab**. **Wraps** from the last to the first, as browsers do | `.tabs(…)` |
| `Ctrl+1`…`Ctrl+9` | `Cmd+1`…`Cmd+9` | Tab *n*, or sidebar item *n*: whichever you chose with `.digits(…)`. `9` is the last; a digit beyond the count does nothing | `.digits(…)` and that destination |
| `Alt+↑` | `Cmd+↑` | The breadcrumb's parent: the crumb just before the current page | `.crumbs(…)` |

**Tab does not wrap, and Ctrl+Tab does.** They are different things. Tab moves *focus*
through the controls, and leaves the group at its end so a keyboard user is never trapped.
Ctrl+Tab changes the *selected tab*, and wraps, as every browser does. **↓ and ↑ inside a menu wrap too:** they move within one
list, and wrapping means a reader can never get stuck at an end.

**The digits are the number row's physical keys,** so they work on every keyboard layout.
On a French AZERTY keyboard, `Ctrl+&`, the key marked 1, goes to destination 1. Named keys
(Tab, the arrows, Page Up and Page Down) are matched as named.

**macOS:** the macOS keys follow Apple's conventions, but **they have not yet been checked
on a Mac.** If one does not behave as this table says, please tell us (RFC-116's long-term
plan, L-8).

## The rules

1. **A shortcut does exactly what a click on that control does.** It publishes the same
   message, takes focus away from wherever it was, and makes that control the place Tab
   carries on from. A disabled control, or one not drawn, does nothing. After `Ctrl+2`,
   Tab therefore carries on from sidebar item 2 (or tab 2), exactly as after a click.
2. **A shortcut works unless the focused control uses that key, and a focused text field
   uses every arrow key, with any modifier.** A code editor keeps `Alt+↑`. A text field
   keeps `Cmd+↑` (macOS: start of text) and `Alt+↑`.
3. **Every shortcut needs a modifier** (Ctrl, Alt or Cmd). No single key is ever a
   shortcut, so a stray keystroke cannot trigger one.
4. **The keyboard does what the pointer can.** With a dialog, a sheet, or a menu with its
   outside-click backdrop open, Tab, Enter and every shortcut reach only the overlay and
   the toasts. Nothing beneath is reachable, as a click cannot reach it either.
5. **A click presses a control, and does not focus it.** The next Tab carries on from the
   control you clicked.

## Escape

| Your overlay has | Escape |
|---|---|
| an escape message (`.escape(Escape { modal, menu })`) | publishes it |
| no escape message, but a click-outside close message (`on_close_modals`, `on_close_menus`) | publishes the close message |
| neither | does nothing: the overlay's own close button is the way out |

An escape message lets an overlay close on Escape but **not** on a stray click outside,
which is right for a destructive confirmation. A focused text field in a dialog takes the
first Escape itself; the second closes the dialog.

`snora::keyboard::dismiss_on_escape` was deprecated in 0.57.0, and is **removed in 0.60.0**.
Use `AppLayout::escape`. For an overlay of your own outside `AppLayout`, match the key
yourself:

```rust,ignore
if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) && self.my_overlay_open {
    return self.update(Message::CloseMyOverlay);
}
```

## Where focus goes

| After | Focus |
|---|---|
| a modal closes | back to the control that had it, unless the page changed or you moved it |
| the focused control goes away (a chip removed, a notice dismissed, a toast closed) | its nearest neighbour in the same group: the next, or else the previous; otherwise nothing |
| a toast elsewhere expires | stays where it was |
| a shortcut | taken away; Tab carries on from the target (rule 1) |
| a click | unchanged by the click; Tab carries on from the clicked control |

Details: [Accessibility](accessibility.md), and the
[overlay interaction semantics](../reference/overlay-interaction-semantics.md).

## Your own content

`snora::control(content)` (engine path) and `snora::design::control(&tokens, content)`
(design path) make any content of yours a control, under every rule above. They take iced
`button`'s builder, so a `button` moves over by changing one name (0.58, RFC-122). An iced
`button` left as it is takes no keyboard input.

## Keys snora's own controls use

So that you can predict rule 2: a focused snora control, `snora::control` included, uses
**Enter and Space**, and nothing else. None of the shortcuts above is used by a snora control. A `text_input` or
`text_editor` of yours uses what iced gives it, plus every arrow key, with any modifier.
