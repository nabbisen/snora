# Migration 0.55 → 0.56

> **A click presses a control and no longer focuses it**, as in 0.54. 0.55.0 drew a
> focus ring after every click, and let Space or Enter press the clicked control
> again. **When the focused control goes away, focus moves to its nearest neighbour.**
> A toast waits while the keyboard is on its ×. Three items are deprecated; nothing
> is removed.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| Any snora control, with the mouse or touch | **No ring after a click**, and no key presses a control the reader only clicked. Tab carries on from the clicked control | Retake screenshot baselines taken on 0.55.0 after a click (§1) |
| Enter or Space held on a focused control | Presses **once** (0.55.0 repeated it) | Nothing |
| Removing a chip, dismissing a notice, closing a toast, pressing an ancestor crumb, from the keyboard | Focus moves to the nearest neighbour in the same group (§2) | Nothing, unless you remove more than the pressed control (§2) |
| `on_toast_hover`, `ToastHover`, `Holds::hover` | **Deprecated**: warns, still works | Move to `on_toast_attention`, `ToastAttention`, `Holds::attend` (§3). Builds that deny warnings must move |
| A list of your own that changes under the reader | Unchanged: a removal elsewhere can still move focus (L-7) | Set focus explicitly (§2) |

**For code:** nothing breaks to compile; the three deprecations warn.

## 1. A click presses, and does not focus (RFC-121)

> **Clicking presses a control. It does not focus it. Focus moves only by the keyboard
> (Tab, through `snora::keyboard::traverse`) or by the application.**

- **0.55.0 got this wrong, and its notes said it had not.** A click focused the
  control, so a ring appeared after every press, and Space or Enter pressed it again,
  in every application, including those that never route Tab. The 0.54 → 0.55 guide
  carries the correction.
- **Tab after a click carries on from the clicked control:** Tab moves to the control
  after it, and Shift+Tab to the one before it. After a press anywhere else, Tab starts
  from the top. A browser does the same.
- **A held key presses once.** A pointer press never repeats, and now neither does a
  key.

**What to re-check:** screenshot baselines taken on 0.55.0 after a click or a tap show
a ring that 0.56.0 does not draw.

## 2. Focus after the focused control goes away (RFC-120)

> **When the focused control goes away, focus moves to its nearest neighbour in the
> same group: the next one, or else the previous one. Otherwise, nothing is focused.**

| You press, from the keyboard | Focus goes to |
|---|---|
| a chip's × | the next chip's **label**, so a second Enter toggles rather than removes; after the last chip, the previous chip's × |
| a notice's dismiss | the control after the notice in its container, else the one before it |
| a toast's × | the next toast's ×, else the previous one's; with no toast left, nothing |
| an ancestor breadcrumb | the nearest crumb before it, so focus stays in the bar; from the root crumb, nothing |
| a row's only control, in your own list | the next row's control, else the previous one's |

In 0.55.0, in half of these cases focus was **inherited by whichever control moved into
the removed one's place**, and one more Enter pressed it. After a chip's ×, that
removed the next chip.

**What the two sentences mean for your code:**
- **"The same group"** is the innermost container that held the control and is still
  there. It never reaches beyond its layer: the page, a dialog, a sheet, or the toasts.
  **If your container is left with no control in it, snora cannot tell that it is still
  there, and treats it as gone: focus moves to the nearest control in the container
  around it.**
- **"Otherwise"** covers two cases: the group has no control left, or snora cannot tell
  which controls went. The second happens if you remove more than the pressed control
  (a row with both Edit and Delete), or if the focused target is not a snora control.
  Nothing is focused then, which is safe: nothing can inherit a focus nobody chose.
- **Your own focus wins.** If `update` moves focus, through a task or an event, snora
  leaves it there.
- **Toasts are followed by id.** A toast that expires or is closed elsewhere leaves
  focus on the toast it was on.
- **A known limit (L-7):** in your own list, removing a *different* item from the
  focused one can still move focus, because iced 0.14 keeps widget state by position.
  **Set focus yourself** in a list that changes under the reader.

## 3. The toast hold, by pointer or keyboard

A toast now waits while the pointer rests on it **or** the keyboard is on its ×. Its
countdown restarts in full only when both have gone.

```rust,ignore
// Before (0.54, 0.55): deprecated, still works
Message::ToastHover(id, crossing) => self.holds.hover(id, crossing, &mut self.toasts, now),
layout.on_toast_hover(Message::ToastHover)

// After (0.56)
Message::ToastAttention(id, attention) => self.holds.attend(id, attention, &mut self.toasts, now),
layout.on_toast_attention(Message::ToastAttention)
```

- `ToastHover::Entered` is `ToastAttention::PointerEntered`, and `Left` is
  `PointerLeft`. `ToastAttention` adds `FocusGained` and `FocusLost`.
- `ToastAttention` is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
- **While both callbacks are set,** the pointer's crossings go to `on_toast_attention`
  only, so you can move without unsetting the old one first.

## What did not change

- No public signature was removed. Colours, sizes, presets and layout are unchanged.
- Tab, Escape and the overlays behave as in 0.55: the keyboard does what the pointer
  can.

## For accessibility records

- **WCAG 2.1.1 (Keyboard):** unchanged since 0.55, met by snora's own controls once
  the application routes Tab.
- **WCAG 2.4.3 (Focus Order):** focus after a removal is now defined (§2), where 0.55.0
  left it to the tree's positions.
- **Correction carried from 0.55:** 0.55.0's "nothing changes for an application that
  does not route Tab" was false (§1). If your record cited it, cite 0.56.0 instead.

## If you are jumping more than one minor

Read [0.54 → 0.55](migration-0.54-to-0.55.md) first.
