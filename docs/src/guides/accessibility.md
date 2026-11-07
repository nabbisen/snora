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
  past the last control leaves the group; it does not wrap. Before 0.55,
  snora's controls were iced `button`s, which take no keyboard input at all.
  *(Corrected 2026-11-07: this once said "keyboard reachability inherited
  from iced", which iced 0.14 does not provide.)*
- **Toasts stay as long as it takes to read (RFC-113).** An error stays
  until it is dismissed. Every other toast stays 2 s plus 1 s per 6
  characters, rounded up, and at least 6 s. A toast under a resting
  pointer does not expire, once the application wires
  `on_toast_hover` and `snora::toast::Holds`. A keyboard user has no
  hold, so use `.persistent()` for anything that must not be missed.
- **The sidebar names its items on the rail (RFC-114).** It no longer
  relies on a hover tooltip that touch users and anyone with an unsteady
  hand cannot use. The icon-only form remains as
  `SideBarLabels::Tooltip`, and says what it costs.
- **A typography scale with usable line-height**, for text you write on top
  of snora. See [Readability](readability.md) for how to pick a role.
- **Frame-level keyboard zone navigation.** snora supplies the decision
  logic to move between skeleton regions — header, sidebar, body,
  footer — as pure vocabulary in `snora_core::focus`:

  ```rust,ignore
  use snora_core::focus::{next_zone, Cycle, ZonePresence};

  let next = next_zone(
      self.focus_zone,
      Cycle::Forward,
      ZonePresence::none().header(true).side_bar(true).footer(true),
      self.show_dialog || self.show_sheet,
      self.open_menu.is_some(),
  );
  if let Some(zone) = next {
      self.focus_zone = zone;
  }
  ```

  **snora does not take Tab or Shift+Tab** for zone cycling. Tab means
  "next control" to users, and to every application that wires focus
  traversal. iced 0.14 itself binds no Tab: an application calls
  `iced::widget::operation::focus_next()`. Claiming it for zones would
  break in-pane navigation. *(Corrected 2026-11-07: this said Tab
  already means "next control" to iced.)* Instead, snora recommends the desktop convention —
  **F6 / Shift+F6** — and supplies a matching key helper,
  `snora::keyboard::cycle_zones`, so you don't have to re-derive which
  modifier means "backward." As with `dismiss_on_escape`, snora installs
  no subscription: your application wires `iced::keyboard::listen()`
  and calls the pure function itself. Cycling is automatically
  suspended while a dialog or sheet is open (focus belongs inside the
  modal), and unaffected by an open menu.

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
  [Law 8](../reference/overlay-interaction-semantics.md#law-8--modal-focus-trapping-is-staged-not-shipped)
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
  Without that one line, Tab does nothing, as before. Two gaps remain, and
  are planned:
  - **with a dialog or sheet open, Tab also reaches the controls beneath
    it**, which the pointer cannot reach (RFC-119);
  - **when the focused control removes itself** (a dismissed notice, a
    closed toast, a removed chip), focus goes nowhere, and the next Tab
    starts from the top of the window (RFC-116's long-term plan).
- **No accessibility tree, no AccessKit integration.** iced 0.14 does
  not expose one, and a layout framework cannot supply this on its own.
  snora's stated position — and why it will not build an interim
  abstraction of its own — is in
  [Position on assistive technology](../contributing/semantic-accessibility.md#position-on-assistive-technology-rfc-045).
- **Your own `iced::widget::button`s are still pointer and touch only.**
  The keyboard route above is snora's `control`, under every control snora
  draws. An iced `button` you build yourself takes no keyboard input and
  registers no focus target, in iced 0.14. Where snora has a control for the
  job, use it: `design::button::{primary, secondary, ghost, danger, link}`
  on the design path. snora has no public control for content of your own
  yet; RFC-116's long-term plan records that. Until then, give such an action
  a keyboard shortcut. Details:
  [iced 0.14 focus-state limitation](../contributing/semantic-accessibility.md#iced-014-focus-state-limitation).
  *(Corrected 2026-11-04: this once said "keyboard activation still works;
  only the visual ring on iced-owned widgets is missing".)*
- **No semantic identifiers or roles.** `iced::widget::Id`s attached to
  snora-rendered surfaces (RFC-047, where implemented) are labels for
  external observation, not accessibility roles or names — an `Id` is
  not a role.
- **No modal focus trapping, yet: with a dialog or sheet open, Tab also
  reaches the controls beneath it.** The dim blocks the pointer (Law 8), but
  iced delivers key presses to every layer, so a control under the dim can be
  focused and pressed. This matters from 0.55, which made snora's controls
  focus targets. **RFC-119 proposes the fix**: the keyboard reaches exactly
  what the pointer can press. Until it ships, close menus and modals before
  acting on the page, and do not rely on a modal to gate keyboard input. Zone
  navigation (above) is already suspended while a modal is open.

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
