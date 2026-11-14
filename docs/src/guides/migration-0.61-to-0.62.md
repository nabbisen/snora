# Migration 0.61 → 0.62

> **Nothing breaks at compile time.** 0.62 is the release of an audit: every keyboard and overlay
> claim in the book was checked against a test. **Two layout defects and two contrast defects are
> fixed**, `Ctrl+Enter` is yours again while a snora control has focus, and several of the book's
> sentences are corrected. Read *"Re-check if you acted on…"* below if you followed the book
> closely.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| A `Sheet` on the **right** (`SheetEdge::End` left-to-right, `Start` right-to-left) | **It now has the width you asked for.** Since 0.4 it had the width of the space beside it | If you chose the opposite size to get the look you wanted, change it back (§1) |
| F6 after a click | F6 counts from the zone you clicked in, as Tab does | Nothing (§2) |
| A notice with an action | The action's label is the accent, in every tone | Retake screenshot baselines (§3) |
| Buttons, chips or links, hovered or pressed, on `light_aaa`, `dark_aaa`, and a few on `light`, `dark` and `high_contrast_dark` | Some state colours change so the label keeps its contrast | Retake screenshot baselines (§4) |
| An application shortcut on `Ctrl+Enter`, `Alt+Enter`, `Ctrl+Space`… | It now fires while a snora control has focus; the control no longer presses | If you relied on `Ctrl+Enter` pressing a focused control, bind plain Enter (§5) |
| Your own iced `button` styled as a snora chip | New: `snora::design::style::chip::{selected, unselected}` | Optional (§6) |
| Anything else | Nothing | Nothing |

## 1. A sheet on the right has the width you asked for

| You asked for | You got on the right (0.61 and earlier) | From 0.62 |
|---|---|---|
| `SheetSize::OneThird` (the default) | **two thirds** of the width | one third |
| `SheetSize::TwoThirds` | **one third** | two thirds |
| `SheetSize::Ratio(r)` | `1 − r` | `r` |
| `SheetSize::Half`, `SheetSize::Pixels(…)` | correct | unchanged |

A sheet on the left, at the top or at the bottom was always right. If you never noticed, there is
nothing to do: your right-hand sheet now looks narrower (or wider), and its screenshot baselines
change. **If you picked `TwoThirds` to get a third, change it back to `OneThird`**, or the sheet
becomes the two thirds you wrote. A test now measures every edge, direction and size.

## 2. F6 after a click

With nothing focused, F6 used to go to the first zone even right after a click in another zone,
though Tab carries on from the click (0.56, RFC-121). **F6 now counts from the zone of the control
last clicked:** click in the sidebar, press F6, and focus goes to the body. A press on anything
that is not a control, or a move of focus by F6, ends that starting point, as it does for Tab.

## 3. A notice's action is a ghost in the accent (RFC-128)

Its label used to take the tone's colour, which is not a text colour. On `Neutral` that read
3.12:1 on `light` and 3.50:1 on `dark`, under the 4.5:1 text needs. On the AAA presets
`Success`, `Warning` and `Danger` fell under 7:1. The tone still colours the bar and the border.
No API changes. **Retake screenshot baselines of notices with an action.**

## 4. Hovered and pressed labels keep their contrast (RFC-129)

A hovered or pressed label now stays at its palette's level, 4.5:1 or 7:1, on every neutral
surface, and every state differs visibly from rest. Where the shift a state has always made keeps
that, nothing moved. **These 17 changed:**

| Preset | Changed |
|---|---|
| `light` | `secondary` hovered (was 4.47:1 on `surface`) |
| `dark` | `secondary` and `ghost` pressed (were 3.73:1) |
| `light_aaa` | `primary`, `secondary`, `ghost` and `danger` hovered; `secondary` and `ghost` pressed; the unselected chip pressed |
| `dark_aaa` | `secondary`, `ghost` and `danger` pressed; `ghost` hovered; the selected chip pressed |
| `high_contrast_dark` | the unselected chip, hovered and pressed: it drew **exactly** like rest |

- `button::link` is drawn with `ghost`'s colours, so a hovered link on `light_aaa` changes too.
- `secondary`'s border is drawn in its label's colour, so it moves with the label and stays above
  3:1.
- **Retake screenshot baselines of hovered or pressed buttons, chips and links on these presets.**

**New: `Palette::text_contrast`** (`TextContrast::Aa` or `TextContrast::Aaa`) is the level snora
holds those states to. See [Design tokens](../design/tokens.md#the-palettes-text-level-062).
- Nothing to do unless you build a palette from a preset and change its colours. The field keeps
  the preset's level: `light_aaa` with your own `accent` still says `Aaa`, and the states still aim
  at 7:1.
- Lower it yourself if you mean to.
- `Palette` is `#[non_exhaustive]`, so no struct literal breaks.

## 5. `Ctrl+Enter` reaches you while a snora control has focus (RFC-130)

A focused control used to press on Enter or Space **whatever else was held, and to capture the
key**. With a snora button focused, an application's `Ctrl+Enter` ("send") pressed the button
instead and never reached `keyboard::listen()`.

**Now a chord with Ctrl, Alt or the logo key is not a press, and it is not captured.** Shift does
not count: `Shift+Enter` and `Shift+Space` still press. A held key still presses once.

```rust,ignore
// If you relied on Ctrl+Enter pressing the focused "Send" control, bind it yourself,
// in the update arm that receives your key events (as you route Tab):
if key == Key::Named(Named::Enter) && modifiers.command() {
    return self.update(Message::Send);
}
```

## 6. Chip styles for your own buttons

`snora::design::style::chip::selected` and `unselected` are the chips' style functions,
`fn(&Tokens, button::Status) -> button::Style`, beside `style::button::*`. They hold the same
contrast rule as the buttons. Like those, they take the tokens, so use them in a closure:
`.style(move |_, status| chip::selected(&tokens, status))`.

## Re-check if you acted on…

The audit found sentences in the book that were not true. They are corrected. **If you built
something on the old sentence, re-check it:**

| The book said | It is | Re-check |
|---|---|---|
| A styled container inside a `design::card` hides the card's border | **Withdrawn.** The card pads its content, so its border and fill show around yours | If you removed your own styling to avoid this, you can put it back |
| `design::control` "looks identical" to a `design::button` variant | **Only with its label at `t.typography.label.size`.** The padding was already the same | A `design::control` you made to give a button an id: set the label size, or it draws larger |
| On the engine path, a sheet has no card of its own | **It has one:** the theme's background, a 1 px border and inside-facing corners. A dialog has none | A card you added inside an engine-path sheet draws a second surface. Remove it |
| Popovers are kept in the window by collision detection | **Only the header dropdown is** (0.59). Other popovers are not a snora concept | An overlay of your own near the window's edge: keep it in yourself |
| Every ancestor crumb keeps focus in the breadcrumb bar | **Every one but the root.** Pressing the root leaves nothing focused | A test or flow that expects focus after pressing the root |
| A menu opens inline below its button | **It is an overlay below its trigger, and takes no room** (0.59) | Layout that left room for the open menu |
| "Every other key is the application's" | **snora answers:** Enter and Space on its controls, the arrows, Home and End in its menus, Escape, and Tab and F6 when you route them | Shortcuts on those keys, besides the chords above |
| Notices' colours "all pass AA" | **Each declared pair passes at its class,** and from 0.62 every text a notice draws, measured as drawn | Nothing, unless you quoted the old sentence |
| `FocusTokens::ring_offset` is the gap between the control and the ring | **It is the distance to the ring's outer edge.** The stroke is drawn inward, so a ring wider than its offset overlaps the control (1 px on the high-contrast presets) | A ring of your own drawn from the token: draw it the same way, or it sits a ring's width further out than snora's |
| The dim is 40% through `snora::render` and 44% through `design::render` | **44% on both paths**, pinned to a pixel | A contrast check of your own that assumed 40% |

Also not visible, and recorded here for completeness: the dim drawn without a close message lost
an `opaque` wrapper that no test could show working. The cover beneath any dim already keeps the
pointer and the wheel from the page (0.60, RFC-125). Nothing you can see or measure changed.

## Security

**RUSTSEC-2026-0486** (`rustix` 1.1.4, unsound extended-attribute functions) is **not reachable
through snora**: none of the crates in its graph that use `rustix` 1.x calls those functions.
snora's lockfile moves to 1.1.6. A library's lockfile does not reach you, so run
`cargo update -p rustix@1.1.4` in your own project to clear it from your audit.

## What did not change

- Tab, F6, Escape and the arrow keys, except §2 and §5.
- Every resting colour, every preset's palette apart from the new field, and the layout of
  anything but a right-hand sheet.

## If you are jumping more than one minor

Read [0.60 → 0.61](migration-0.60-to-0.61.md) first: it deprecated `next_zone`, `FocusZone` and
`ZonePresence`, which are removed in 0.64.0.
