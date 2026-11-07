# Migration 0.54 → 0.55

> **Every control snora draws can now be reached from the keyboard**, once your
> application routes Tab in one line, and overlays block the keyboard as they
> block the pointer. Targets and icons grow with the text, and
> `design::button::link` is new. Nothing is removed and nothing is deprecated.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| Any snora control: tabs, crumbs, menus, notices, chips, `design::button`, the toast's ×, the sidebar | Each is a **focus target**: Enter and Space press it, and a ring shows focus | Route Tab (§1). Without it, nothing changes |
| The unstyled path at the default text size | Chrome icons are **16 px, not 14**; tabs and menu items with an icon are 1 to 1.5 px wider | Retake screenshot baselines (§2) |
| A larger `default_text_size` (unstyled) | Icons and targets follow the text: at 24 px, icons are 24 px and the toast's × target is 36 × 36 | Retake baselines (§2) |
| The design path's toasts | The × is **15.75 px, not 18**: it follows `Typography::body`. Its target stays 24 | Retake toast baselines (§2) |
| An overlay with `on_close_modals` / `on_close_menus` | **Escape closes it**, and the keyboard no longer reaches beneath it | Nothing; see §4 if your `update` handles Escape |
| `ghost` as a link ("Learn more", "Choose another folder") | `design::button::link` exists for that, underlined | Move links to `link` (§3) |
| `Tokens` | A new field, `targets` (`TargetTokens`, 24 in every preset) | Nothing; raise it to grow every design-path control (§2) |
| Your dependency list | A new crate, `snora-control`, arrives through `snora` | Nothing. **Do not depend on it directly** |

**For code:** nothing breaks to compile, and nothing warns.

## 1. The keyboard (RFC-116)

iced binds no Tab key of its own, so your application routes it:

```rust,ignore
// subscription
iced::keyboard::listen().map(|event| match event {
    iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
        Message::KeyPressed(key, modifiers)
    }
    _ => Message::Ignored,
})

// update
Message::KeyPressed(key, modifiers) => {
    if let Some(task) = snora::keyboard::traverse(key, modifiers) {
        return task;
    }
}
```

Then:
- **Tab and Shift+Tab** move between controls;
- **Enter and Space** press the focused one, and publish **the same message a click
  publishes**;
- **a ring** shows focus, at ≥ 3:1 against what it sits on. It is drawn from
  `Tokens::focus` on the design path, and in the theme's text colour on the unstyled
  path;
- **a control with no handler** (`*_maybe(.., None)`, a chip given `None`) is
  disabled, and Tab skips it;
- **Tab past the last control leaves the group.** It does not wrap. The next Tab
  starts again from the top of the window.

The `tab` example is wired this way. **Your own `iced::widget::button`s are not
changed:** in iced 0.14 they take no keyboard input. Where snora has a control for
the job, use it.

**Known gaps, planned (RFC-116's long-term plan):**
- when the focused control removes itself (a dismissed notice, a closed toast, a
  removed chip), focus goes nowhere, and the next Tab starts from the top;
- focusing a toast's × does not hold the toast, as a resting pointer does;
- an icon-only (`Tooltip`) rail shows no name to a keyboard user.

## 2. Sizes follow the text (RFC-115)

> **A control is at least `max(24 px, 1.5 × its text, Tokens::targets.min)` in both
> directions. A pictogram is 1 em of the text it sits with, and the toast's × is
> 1.125 em.**

| Where | 0.54 | 0.55 |
|---|---|---|
| Unstyled chrome icons, default text | 14 px | **16 px** |
| Unstyled tab or menu item with an icon, default text | | 1 to 1.5 px wider |
| Unstyled, 24 px text: icons | 14 px | **24 px** |
| Unstyled, 24 px text: toast × target | 24 × 24 | **36 × 36** |
| Design path: toast × | 18 px | **15.75 px** (1.125 × `body`) |
| Design path: chrome icons | `Typography::label` (14) | unchanged |
| Every target at the default text | 24 | unchanged |

**`Tokens::targets.min`** is 24 in all six presets. Raise it to 44, say, and every
control on the design path grows to match. A value under 24 changes nothing: WCAG
2.5.8's 24 px is a floor that no token can lower.

## 3. `design::button::link` (RFC-117)

```rust,ignore
button::link(t, "Choose another folder", Message::ChooseFolder)
```

> A **link** takes the reader somewhere: another view, a document, a chooser. An
> **action** does something where they are: Cancel, Skip, Save. Use `link` for the
> first, `ghost` for the second.

- **The underline is the label's colour in every state, and under every line.** Its
  rule is in [Buttons](../design/buttons.md#link-or-action).
- **There is no `link_maybe`.** When the destination is unavailable, show plain text.
- **`ghost` is unchanged.**

## 4. Overlays: the keyboard does what the pointer can (RFC-119)

| With this open | Tab and Enter reach | Escape |
|---|---|---|
| a dialog or sheet | the overlay and the toasts, never the page | closes it, if `on_close_modals` is set |
| a menu with its backdrop (`on_close_menus`) | the menu and the toasts | closes it |
| a menu without a backdrop | everything, as the pointer does | yours, as before |

- **When a modal closes, focus returns to the control that had it**, unless the page
  now has a different number of controls, or your application has focused something
  itself. Then focus is wherever you put it, or nowhere.
- **Opening a modal never resets the page:** scroll offsets and a text input's cursor
  survive.
- **The engine's Escape is captured,** so your `keyboard::listen()` does not receive
  it. If you call `dismiss_on_escape` as well, you get one message, not two. Keep
  `dismiss_on_escape` only for an overlay with no click-outside close message.
- **Your own `text_input` beneath an open modal no longer takes keys,** and has no
  cursor while the dim is up. The pointer could not reach it either.
- **If your `update` reacts to Escape for something else while an overlay with a close
  message is open,** it no longer sees that Escape.

## Fixed without a visible change

- A tab pressed from the keyboard kept its focus when it became the active tab.
- A focus ring is drawn above its neighbours, so a chip's remove button cannot hide
  part of its label's ring.

## For accessibility records

- **WCAG 2.1.1, Keyboard, is met by snora's own controls** once the application routes
  Tab (1.0 gate 11). An application's own iced buttons are not covered.
- **Corrected:** `design/notices.md` and `design/v021-primitives.md` said, through
  0.54, that the notice's buttons were "keyboard-reachable". In 0.54 they were not:
  they were iced buttons. From 0.55 they are.

## What did not change

- No public signature changed. Nothing was removed or deprecated.
- Every target's height at the default text size, and every design-path icon at the default
  tokens.
- Colours.

## If you are jumping more than one minor

Read [0.53 → 0.54](migration-0.53-to-0.54.md) first.
