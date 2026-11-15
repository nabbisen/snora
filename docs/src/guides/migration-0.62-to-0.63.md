# Migration 0.62 → 0.63

> 0.63 is the release of a second audit, of every visual guarantee in the book: contrast,
> surfaces, typography, direction and stability. **Right-to-left layouts are mirrored for real,**
> **every word and every non-text element snora draws is measured as drawn,** and several of the
> book's sentences are corrected. **One compile-time break:** `row_dir` returns a `LogicalRow`.
> Read *"Re-check if you acted on…"* below if you followed the book closely.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `snora::direction::row_dir` / `row_dir_three` | They return **`LogicalRow`**, not `iced::widget::Row`; under RTL the children keep their order and only the positions mirror | If you name `Row` or call `wrap`, `clip`, `extend` and so on, see §1 |
| A right-to-left layout | The header, breadcrumb, toast close mark and tab and menu icons now mirror; **Tab and `Ctrl+Tab` follow reading order**; a direction change keeps focus and scroll | Retake RTL screenshot baselines; check any flow that assumed the old Tab order (§2) |
| `light_aaa` with a tab bar | An inactive tab's label is a little darker | Retake its baselines (§3) |
| A progress bar, a `Debug` toast, a long menu, a raised card on `light` / `light_aaa` | Each gains or firms an outline, to reach 3:1 | Retake their baselines (§3) |
| The unstyled path with a `Debug` toast | It has an outline in the page's text colour | Retake its baselines (§3) |
| Your own tab buttons styled like snora's | New: `snora::design::style::tab::{active, inactive}` | Optional |
| iced's checkbox, toggler, radio, pick list, combo box or slider | **Nothing changed in snora**, but the book now says the keyboard cannot reach them | §4 |
| Anything else | Nothing | Nothing |

## 1. `row_dir` returns a `LogicalRow` (RFC-132)

```rust,ignore
// Unchanged: pass the result on as an element, or use the row builders it keeps.
let bar = row_dir(direction, title, actions).spacing(8).width(Length::Fill);

// No longer compiles: `wrap`, `clip`, `extend`, or naming the type `Row`.
// Build the row you need instead:
let bar = LogicalRow::new(direction).push(title).push(actions).spacing(8);
```

- `LogicalRow` keeps `push`, `spacing`, `align_y`, `width`, `height`, `padding` and
  `Into<Element>`. The other `Row` methods have no mirrored meaning, and one that half-worked under
  RTL would fail silently, so they are not there.
- **Under RTL your row's children keep the order you gave.** A direction change then keeps their
  state, and Tab visits them start to end, as it does in snora's own widgets (§2).
- **Give a mirrored row a width, or align its container to the start edge:** a row only as wide as
  its content is mirrored about its own width ([Direction](direction.md#direction-aware-rows)).

## 2. Right-to-left (RFC-132)

- **Mirrored now:**
  - the header, with the title at the right edge, its menus to the left and the end controls at
    the left edge (before, the end controls sat beside the title);
  - the breadcrumb, which starts at the right edge;
  - a toast's `×`, at its end edge;
  - the icon in a tab, a menu trigger and a menu item.
- **Tab and `Ctrl+Tab` follow reading order:** start to end, which is right to left on screen.
  - Before, under RTL they went left to right on screen, which was reading order reversed. The
    header's end control came before its menus, and the body came before the sidebar.
  - `Ctrl+Tab` now moves to the next declared tab, which is to the left.
  - `Ctrl+1`…`Ctrl+9`, `Alt+↑` and F6 are unchanged.
- **A live direction change keeps focus, scroll offsets and an open menu.** Before, it lost them.
- **`Shortcuts::routes(direction)`:** its direction is ignored now. It is removed in 0.64.0.
- **Still left to right under RTL, until 0.64:** `design::notice::Notice`, `design::chip::removable`,
  `design::progress` (and its fill), `app_footer`, and a bare `render_menu`. 0.64 gives each a
  required `direction` (RFC-133).

## 3. Contrast, as drawn (RFC-131, RFC-135)

Every **word** snora draws on the design path now holds its palette's level (4.5:1 or 7:1). Every
**non-text element** a reader needs (a glyph, a border, a track, a scrollbar) holds 3:1. Both are
measured on the drawn pixels, in every preset, state and direction. What moved:

| What | Before | Now |
|---|---|---|
| An inactive tab's label, `light_aaa` | 6.10:1 on `surface`, under 7 | 7.01:1 (a 26% mix, not 30%); other presets unchanged |
| A progress bar's track | 1.00:1, invisible on the high-contrast presets | a 1 px outline in the `border` role |
| A `Debug` toast | 1.00 to 1.20:1 against the page | a 1 px outline (`border` role; the page's text colour on the unstyled path) |
| A long menu's scrollbar thumb | 1.02 to 1.8:1 | 3:1 against its track and the page |
| `card::raised`'s border, `light` and `light_aaa` | 2.75:1 against its own shadow | 3.06:1 (firmed 7%); this includes the sidebar's and a chip's tooltip bodies |

**Retake the screenshot baselines** of each. Nothing else moved: tests assert every colour that
passed before, to the float.

## 4. Form controls

snora draws no checkbox, toggle, radio, pick list or slider, and **iced 0.14's own cannot be
reached by the keyboard.** Only `text_input` is a Tab stop. Nothing changed in 0.63; the book used
to recommend these controls without saying so.
- **A checkbox, toggle or radio:** build it on `snora::control`
  ([Keyboard § Form controls](keyboard.md#form-controls)).
- **A pick list or a slider** has no keyboard route yet.
- snora's own, keyboard-reachable versions are planned for 0.65 (RFC-134).

## Re-check if you acted on…

The audit found sentences that were not true. They are corrected. **If you built on the old
sentence, re-check:**

| The book said | It is | Re-check |
|---|---|---|
| snora applies only the `label` and `body` text roles, so `title` is safe to redefine | **snora applies `title` too:** the design header's title and a toast's title, since 0.52 | A redefined `title` has resized those since 0.52 |
| 0.62.0: "every text snora draws holds its preset's contrast level" | It held for buttons, chips, links and a notice's action; **from 0.63, every word** | A conformance record that cited 0.62 for all text |
| Prefer iced's checkbox, pick list and slider as "native" controls | **The keyboard cannot reach them** | Forms built on that advice (§4) |
| High-contrast borders are 21:1 | 21:1 against the page and `surface`; **19.80:1** on `high_contrast_dark`'s raised surface | A figure you quoted |
| The unstyled dim is black | **The pole opposite the theme:** white on a dark theme | A dim you assumed black |
| `background.strong` is a border colour only | iced also uses it as a **pressed fill under text** (`button::background`, `button::subtle`); text on it holds AA | Nothing, unless you relied on it never carrying text |
| `AppLayout::direction` reaches the prefab widgets | **It does not:** pass `direction` to each one | A prefab widget you passed no direction |
| The palette's roles are asserted "on every build" | By snora's own test suite, on **snora's** CI | Nothing, unless you counted it as your own check |
| The frozen surface (stability, API governance) | The lists now include `light_aaa`, `dark_aaa`, `presets()`, `text_contrast`, `TextContrast`, `TargetTokens`, and `snora_style::chip` and `tab` | Nothing to do; the covenant always covered them |

## What did not change

- Every left-to-right layout, to the pixel.
- Every colour that passed, to the float.
- The keyboard under left-to-right.

## Coming in 0.64.0, a breaking release

- `next_zone`, `FocusZone` and `ZonePresence` are removed (deprecated in 0.61).
- The five widgets above take a required `direction`, and snora's own text aligns to the start
  edge under RTL (RFC-133).
- `Shortcuts::routes` loses its ignored direction.

## If you are jumping more than one minor

Read [0.61 → 0.62](migration-0.61-to-0.62.md) first: a right-hand sheet's width, and `Ctrl+Enter`.
