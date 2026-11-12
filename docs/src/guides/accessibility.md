# Accessibility

Short version: snora provides strong **layout-direction and visual**
accessibility. It provides **no assistive-technology support** — no
accessibility tree, no AccessKit integration, no semantic identifiers a
screen reader could read. If you are auditing snora for a downstream
project, this page is the place to start; the detail lives in two
contributor-facing documents linked below, not duplicated here.

## What snora provides

- **Logical layout direction (ABDD).** Layout is described in
  `Edge::Start` / `Edge::End`, not "left" / "right", so an application
  written for LTR readers also works for RTL readers without a
  per-screen rewrite. See [Direction and ABDD](direction.md).
- **Contrast-tested design tokens.** When the `design` feature is
  active, every built-in preset is WCAG AA–verified. Since 0.54 there are
  three families (`Tokens::presets()` lists them all):
  - `light` and `dark` hold text at AA, 4.5:1;
  - `light_aaa` and `dark_aaa` hold **every declared text pair at AAA,
    7:1**, with their parent's character (RFC-118);
  - `high_contrast_light` and `high_contrast_dark` are for maximum
    contrast, with text at 8.38:1 or more and borders at 21:1. **Every threshold is a floor, never a ceiling** — if a
  decision in your own code depends on a snora colour staying *below*
  some contrast value, that is not a guarantee snora makes; see
  [API governance § Permitted without reopening the gate](../contributing/api-governance.md#permitted-without-reopening-the-gate)
  for the full statement.
- **Every control snora draws is reachable from the keyboard (0.55,
  RFC-116).** Tabs, breadcrumb ancestors, menu triggers and items, the
  notice's buttons, chips and their remove buttons, every
  `design::button` variant (including `link`), the toast's close mark and
  the sidebar's items are focus targets. Enter and Space press the focused
  one and publish the same message a click does, and a focus ring of at
  least 3:1 shows where focus is. A control with no handler is disabled and
  skipped. **iced binds no Tab key, so your application routes it, in one
  line of `update`:**

  ```rust,ignore
  Message::KeyPressed(key, modifiers) => {
      if let Some(task) = snora::keyboard::traverse(key, modifiers) {
          return task;
      }
  }
  ```

  Subscribe with `iced::keyboard::listen()`, as the `tab` example does. Tab
  past the last control leaves the group; it does not wrap. **A click presses
  a control and does not focus it** (0.56, RFC-121): no ring appears from a
  click, and no key answers for a control the reader only clicked. The next
  Tab carries on from the clicked control, as in a browser. **A held key
  presses once.** Before 0.55,
  snora's controls were iced `button`s, which take no keyboard input at all.
  *(Corrected 2026-11-07: this once said "keyboard reachability inherited
  from iced", which iced 0.14 does not provide.)*
- **Toasts stay as long as it takes to read (RFC-113).** An error stays
  until it is dismissed. Every other toast stays 2 s plus 1 s per 6
  characters, rounded up, and at least 6 s. A toast under a resting
  pointer does not expire, and nor does one whose × has keyboard focus
  (0.56), once the application wires `on_toast_attention` and
  `snora::toast::Holds`. Use `.persistent()` for anything that must not be
  missed.
- **The sidebar names its items on the rail (RFC-114).** It no longer
  relies on a hover tooltip that touch users and anyone with an unsteady
  hand cannot use. The icon-only form remains as
  `SideBarLabels::Tooltip`, and says what it costs.
- **A typography scale with usable line-height**, for text you write on top
  of snora. See [Readability](readability.md) for how to pick a role.
- **F6 moves between the frame's zones (0.61, RFC-126):** the header, the sidebar, the body
  and the footer. Route the key, as you route Tab:

  ```rust,ignore
  if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
      return snora::keyboard::focus_zone(cycle);
  }
  ```

  - **F6 / Shift+F6** move focus to the first control of the next / previous zone, **counted
    from the zone that holds focus when you press the key.** After a Tab or a click into the
    sidebar, F6 goes to the body.
  - **A zone with nothing to focus is skipped.** The order wraps, and it does not mirror under
    RTL: the sidebar follows the header in either direction.
  - **With focus in no zone** (nothing focused, a toast, a context menu), F6 goes to the first
    zone and Shift+F6 to the last.
  - **F6 lands only on a zone's own controls, never inside an open menu.** An `app_header`
    dropdown counts in its trigger's zone, so F6 from one of its items moves on.
  - **Under a dialog, a sheet, or a menu with its backdrop, F6 does nothing:** the zones are
    beneath it.
  - **F6 never presses anything.** Held, it keeps moving, one zone per repeat, as Tab does.

  **Why you route it, when snora answers Escape by itself:** your application may already use
  F6, a debugger's "step over" for instance, and an upgrade must not take a key away. snora
  answers the keys inside its own controls and overlays (Enter, Space, the arrows in a menu,
  Escape). The keys that move between controls (Tab, F6) are yours to route to it.

  *Until 0.61, snora offered only `next_zone`, a decision your application had to act on. With
  snora's controls it could not: it moved no focus, counted from a variable, and ignored a
  covering menu. `next_zone`, `FocusZone` and `ZonePresence` are deprecated in 0.61, and removed
  in 0.64.0.*

## If your application serves older readers (RFC-115)

Each setting below says what snora then guarantees. Nothing here is on by default,
because each one costs space. orbok, whose readers are mainly older, uses all of
them.

| Do this | snora then guarantees |
|---|---|
| **Set a larger text size:** `default_text_size` on the unstyled path, or scale `Typography` on the design path | Every control's target is at least **1.5 × its text** (and never under 24 px). Icons are **1 em** of the text beside them, and the toast's × is 1.125 em. Sidebar names, tab labels and toast text follow it. Caller text wraps and is never clipped (RFC-105) |
| **Raise `tokens.targets.min`** (44, say) | Every design-path control is at least that, in both directions: `design::button`, `design::chip` and its ×, and the `design::widget` tab bar and sidebar. Below 24 it does nothing |
| **Choose `Tokens::light_aaa()` / `dark_aaa()`**, or the high-contrast presets | **Every declared text pair at 7:1 or more**, the accent included (the AAA presets keep Light's and Dark's character; high contrast goes further, with text at 8.38:1 or more and borders at 21:1). Asserted by `cargo test -p snora-design` |
| **Keep the labelled sidebar**, the default since 0.54 | Each item's name is on the rail, beneath its icon. Nothing depends on hovering, which an unsteady hand cannot do (RFC-114) |
| **Use `.persistent()`** for a toast that must be read | It stays until dismissed. Every other toast already stays 2 s plus 1 s per 6 characters, at least 6 s, and errors are persistent by default (RFC-113) |
| **Use `design::button::link`** for anything that takes the reader somewhere | It is underlined under every line, so it does not depend on colour (RFC-117) |
| **Route Tab** with `snora::keyboard::traverse` | Every control snora draws is reachable and pressable from the keyboard, with a focus ring of at least 3:1 (RFC-116) |

## What snora does not provide

- **snora's prefab surfaces distinguish semantic variants by colour
  alone.** A consumer relying on them for WCAG 1.4.1 (Use of Colour)
  must supply a non-colour channel themselves — typically per-variant
  text, which most applications already do: a notice or a toast
  written with its own distinct title and body carries the distinction
  in words no matter what colour it renders in, which is a legitimate
  division of labour, not a workaround. snora contributes colour; the
  consumer supplies the words.

  This is asserted by test, not just stated — the same shape
  [Law 8](../reference/overlay-interaction-semantics.md#law-8--the-keyboard-reaches-only-what-the-pointer-can-press)
  uses for its own boundary. Three surfaces are covered, each exhaustive
  over its variant enum so a new variant fails to compile without an
  entry:

  | Surface | Varies by | Register |
  |---|---|---|
  | `snora::toast` (`toast_style`, `close_button_style`) | `ToastIntent` (5) | `crates/snora/src/toast/channel_register.rs` |
  | `snora_widgets::design::notice` | `Tone` (6) | `crates/snora-widgets/src/design/channel_register.rs` |
  | `snora_widgets::design::progress` | `Tone` (6), via `snora-style` | `crates/snora-widgets/src/design/channel_register.rs` |

  The register does not, and cannot, prove 1.4.1 conformance — that
  depends on text the application supplies, not on anything snora
  renders. Its job is narrower: to fail the day a prefab's colour-only
  claim and its code diverge, in either direction.

  Adding a non-colour cue (an icon, a textual prefix) to these prefabs
  was **ruled against on 2026-09-12, not left open** — see RFC-093's
  Q-1 ruling for the reasoning. In short: the division of labour above
  is the answer, it is asserted by test rather than merely stated, and
  every adopting team that has checked is already safe without a cue.
  Adding one later stays additive if a requirement ever arrives, so
  nothing here is foreclosed.

  **If you expected these prefabs to carry a non-colour channel, tell
  us.** That is the one report that would reopen the decision, and it
  would mean this page failed to reach you in time — which is worth
  more to us than the feature request.
- **States snora draws itself each carry an asserted non-colour cue
  (RFC-103, RFC-106).** Variants are the caller's to put into words (above).
  States are different: the caller's words do not change when a tab becomes
  active or a chip is selected, so snora supplies the cue, and a state
  register (`crates/snora-widgets/src/state_register.rs`) makes a new state
  without one fail to compile:

  | State | Cue | Asserted |
  |---|---|---|
  | Active tab | shape — a 2 px underline | rendered bounds |
  | Sidebar active item | fill | ≥ 3.0:1 against the rail |
  | Chip selected | fill luminance | ≥ 3.0:1 against unselected (measured 6.19–11.75:1) |
  | Chip disabled, selected | fill | ≥ 2.6:1 against enabled |
  | Chip disabled, unselected | dimmed label | ≥ 3.0:1 against enabled |
  | Design button disabled | fill or label, by family | ≥ 2.9:1 against enabled |
  | Menu open | shape — the dropdown | rendered |
  | Breadcrumb ancestor | shape — a 1 px underline (RFC-109) | rendered bounds |
  | Breadcrumb leaf | position — last, plain text | documented |

  **One qualification, stated so a record does not overclaim.** A selected
  chip's cue is a luminance difference, not a word or a shape. If your
  conformance record needs a word or shape for it, **put a mark in the
  chip's label** (a ✓, for instance), as orbok does. snora does not add one,
  because a default-on mark would double theirs (RFC-103 Q-1). Disabled
  labels are dimmed below 4.5:1 by design; WCAG 1.4.3 exempts inactive
  controls.
- **Keyboard (WCAG 2.1.1): met by snora's own controls from 0.55, once
  your application routes Tab with `snora::keyboard::traverse`** (above).
  Without that one line, Tab does nothing, as before. *(In 0.55.0 alone, a click
  also focused a control, so a ring appeared after a click and Space or Enter
  pressed it again. 0.56.0 removed that; see RFC-121 and the 0.54 → 0.55
  guide's correction.)*
- **Shortcuts, opt-in (0.57, RFC-116 R-2).** `AppLayout::shortcuts` routes
  next and previous tab, tab or sidebar item *n*, and the breadcrumb's parent. A
  shortcut does exactly what a click does, and works unless the focused control uses
  the key. Every binding needs a modifier, and the macOS table has not yet been
  checked on a Mac. The [Keyboard map](keyboard.md) lists every key.
- **Overlays: the keyboard does what the pointer can (0.55, RFC-119).** With a
  dialog, a sheet, or a menu with its backdrop open, Tab and Enter reach only
  what the pointer can press: the overlay and the toasts, never the page
  beneath. An overlay that closes on a click outside also closes on **Escape**,
  with nothing to wire. **When a modal closes, focus returns to the control
  that had it**, unless the page now has a different number of controls, or
  your application has focused something itself; then focus is wherever you
  put it, or nowhere, and the next Tab starts from the top. Opening a modal
  never resets the page: scroll offsets and a text input's cursor survive.
  The rules are [Laws 7 and 8](../reference/overlay-interaction-semantics.md#law-7--an-overlay-that-closes-on-a-click-outside-also-closes-on-escape).
- **When the focused control goes away (0.56, RFC-120):**

  > **Focus moves to its nearest neighbour in the same group: the next one, or
  > else the previous one. Otherwise, nothing is focused.**

  Removing a chip moves focus to the next chip's label, not its ×, so pressing
  Enter again cannot chain into removing more. Dismissing a notice moves focus
  to the control after it. Closing a toast moves focus to the next toast's ×.
  Pressing an ancestor breadcrumb keeps focus in the breadcrumb bar.

  **For application authors, what the two sentences mean exactly:**
  - **"The same group"** is the innermost container that held the control and
    is still there: a chip row, the toast stack, the breadcrumb bar, your list.
    It never reaches beyond its layer (the page, a dialog, a sheet, the
    toasts), so focus does not jump to the footer, or from the toasts to the
    page. **If your container is left with no control in it, snora cannot tell
    that it is still there, and treats it as gone: focus moves to the nearest
    control in the container around it.**
  - **"Otherwise"** is when the group has no control left, or snora cannot
    tell which controls went. That happens if you removed more than the
    pressed control, such as a row with both Edit and Delete, or if the
    focused target is not a snora control. Leaving nothing focused is the safe
    answer: an inherited focus is what would let one more Enter press a
    control nobody chose.
  - **Your own focus wins.** If your `update` moves focus itself, snora
    leaves it there.
  - **A known limit (RFC-116 L-7):** in your own list, removing a *different*
    item from the one focused can still move focus to another control, because
    iced 0.14 keeps widget state by position, and snora has no identity for
    your widgets. snora's toasts are followed by id and are not affected. In a
    list that changes under the reader, set focus yourself.
- **No accessibility tree, no AccessKit integration.** iced 0.14 does
  not expose one, and a layout framework cannot supply this on its own.
  snora's stated position — and why it will not build an interim
  abstraction of its own — is in
  [Position on assistive technology](../contributing/semantic-accessibility.md#position-on-assistive-technology-rfc-045).
- **Your own pressable content: use `snora::control` (0.58, RFC-122).** An
  iced `button` you build yourself takes no keyboard input and registers no
  focus target, in iced 0.14. **Change its name to `snora::control`**, or to
  `snora::design::control(&tokens, …)` on the design path, and keep the rest
  (`on_press`, `padding`, `style`). It then gets everything above: Tab, Enter
  and Space once, no focus from a click, a focus ring, the target floor
  (`targets.min` on the design path), silence under an overlay, and focus to a
  neighbour if it goes away. *The ring is at least 3:1 against the page and every neutral surface snora's tokens define; on the engine path, against the theme's background tiers except SolarizedDark's `strongest` (2.91:1). On a coloured fill of your own, check it.* For plain labels,
  `design::button::*` is still the shorter way. Details:
  [iced 0.14 focus-state limitation](../contributing/semantic-accessibility.md#iced-014-focus-state-limitation).
  *(Corrected 2026-11-04: this once said "keyboard activation still works;
  only the visual ring on iced-owned widgets is missing".)*
- **No semantic identifiers or roles.** `iced::widget::Id`s attached to
  snora-rendered surfaces (RFC-047, where implemented) are labels for
  external observation, not accessibility roles or names — an `Id` is
  not a role.
- **No strict modal focus trap.** Tab inside an open modal moves through the
  modal and the toasts only (Law 8), but past the last control there is one
  press where nothing is focused before Tab starts again from the modal's
  first control. That is iced's `focus_next`. Where focus *starts* when a
  modal opens is your application's to set.

## What "ABDD" means, precisely

**Accessible By Default and by Design** means layout-direction
correctness and visual accessibility. It does not mean assistive
technology is supported. Read that as a boundary, not an apology: the
visual and layout work is real, tested, and unusual for a framework
this size. The correction is that the name does not, on its own, cover
more than that.

## If you are writing acceptance evidence

If your audit needs a citable statement rather than an inference from
silence: **snora will integrate an accessibility tree when iced exposes
one, and will not build a parallel accessibility abstraction of its own
in the interim.** That is snora's own stated position, recorded with a
reconsideration trigger in
[design decisions](../contributing/design-decisions.md#why-snora-has-no-interim-accessibility-tree-v027).
