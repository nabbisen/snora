# Migration 0.65 → 0.66

> **Nothing breaks at compile time.** Two additions: `keeps_focus(id)`, for a control that shows
> or hides others after it (RFC-137), and a radio group laid out in a row (RFC-138). Two fixes: the
> marks snora draws itself no longer break when `lucide-icons` is on and its font is not loaded
> (RFC-139), and focus no longer lands on a control nobody pressed (RFC-137's review). **One
> appearance change,** for applications that turn `lucide-icons` on *and* load its font.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `lucide-icons`, and you load `LUCIDE_FONT_BYTES` | The toast's ×, a notice's dismiss, a chip's remove, the checkbox's tick and the select's chevron change from Lucide's shapes to "×", "✓" and "▾", and those controls narrow by a few pixels | Re-check screenshot tests (§1) |
| `lucide-icons` (or a crate in your graph turns it on), and you do **not** load the font | Those five marks were broken, and now draw correctly | Nothing for snora's marks. Load the font if you show Lucide icons yourself (§1) |
| A disclosure, tree node or "show details" row built on `snora::control` | After Enter, nothing stayed focused | Add `keeps_focus(id)` (§2) |
| iced's radios laid out in a row, kept to keep the row | snora's group can be a row now | Optional: `.horizontal(true)` (§3) |
| A press that also adds or removes controls *before* the pressed one | Focus could land on a control nobody pressed | Nothing: fixed (§4) |
| Anything else | Nothing | Nothing |

## 1. `lucide-icons` changes nothing snora draws itself

With the feature on and the font not loaded, five marks snora draws itself were broken: a
missing-glyph box for the toast's close mark and the notice's and chip's, **a "1" in a checked
checkbox, and a "3" for the select's arrow.** An application did not have to ask for Lucide to
get them: a Cargo feature can be turned on by any crate in the dependency graph.

From 0.66 the five marks are the text glyphs "×", "✓" and "▾" in every build. The feature adds
the Lucide set for **your** icons, and nothing else.

- **If you load the font,** the five marks change shape, and the controls around them narrow: a
  notice's dismiss control from 34.0 to 28.0 px wide, and a select by up to 8 px. **Re-check any
  screenshot or layout test** that covers a toast, a notice, a removable chip, a checkbox or a
  select.
- **If you show Lucide icons yourself, load the font once, at start-up.** No page said so before
  0.66:

```rust,ignore
iced::application(App::default, App::update, App::view)
    .font(snora::lucide::LUCIDE_FONT_BYTES)
    .run()
```

See [Icons § `Icon::Lucide`](icons.md#iconlucide--the-curated-icon-set).

## 2. `keeps_focus(id)`: a disclosure keeps its focus

A control that stays where it is when pressed, and shows or hides controls after it, used to
leave **nothing** focused after Enter, so the next Tab started at the top of the window. Declare
it, and focus stays on it:

```rust,ignore
snora::control(text(node.name.as_str()))
    .on_press(Message::Toggle(node.path.clone()))
    .keeps_focus(format!("node:{}", node.path))
```

- **Name the item, not its place.** The id is found wherever the control now is in Tab order, so
  a node's path works and a row number does not.
- **It is `.id(id)` plus the declaration,** so `focus(id)` reaches it too.
- **Misuse is safe:** if the control went away after all, or two controls share the id, nothing
  is focused, never a neighbour.
- A click, an undeclared press and your own focus tasks are unchanged.

See [Keyboard § Your own content](keyboard.md#your-own-content).

## 3. A radio group in a row

```rust,ignore
snora::radio_group(["List", "Grid"], Some(&self.view_as), Message::ViewAs, direction)
    .horizontal(true)
```

- **The column stays the default.** The argument is a `bool`, so a layout that changes with the
  window's width can pass its own condition.
- **The row always wraps,** between options and never inside one, keeps the options' order
  line by line, and mirrors each line under right-to-left.
- **The keys are the column's:** in a wrapped row ↓ is the next option, not the one below it.

## 4. Focus no longer lands on a control nobody pressed

iced keeps a control's state by its place in the tree, not by its place in Tab order. So when a
press also added or removed controls *before* the pressed one, the focus the pressed control
carried could end up on another control, and one more Enter pressed it. **From 0.66 that focus
goes to the neighbour RFC-120 names, or to nothing where snora cannot tell.**

- A control replaced by one of the same kind in the same place (Play becoming Pause) passes its
  focus to the replacement.
- **Re-check** a keyboard test that expected focus to land somewhere after such a press. It may
  have been asserting the stranger.

## Re-check if you acted on what the book said

| The book said | It is | Re-check |
|---|---|---|
| A toast shows its intent by its background colour *and* the text around it | **By colour alone.** Nothing else in the toast changes with the intent | Say what happened in the toast's title and message ("Could not save"), not only with a red toast |
| `lucide-icons`: Cargo includes only the icons you reference | The feature brings the whole set, as one enum and one font file | Any binary-size budget that assumed otherwise |
| (nothing, before 0.66) whether the Lucide font must be loaded | **It must,** by the application | §1 |
| (nothing, before 0.66) the colours of an `Icon::Svg` | **The file's own colours.** No theme or token recolours it, and no snora contrast guarantee covers it | The contrast of your SVG icons, in every theme you ship |

## What did not change

Everything else you draw, in both directions, to the pixel.

## If you are jumping more than one minor

Read [0.63 → 0.64](migration-0.63-to-0.64.md) first: it is the breaking release.
