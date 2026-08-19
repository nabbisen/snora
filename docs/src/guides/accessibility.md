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
  active, all four built-in presets (`light`, `dark`,
  `high_contrast_light`, `high_contrast_dark`) are WCAG AA–verified —
  `light`/`dark` and AAA for the two high-contrast presets, where
  applicable. **Every threshold is a floor, never a ceiling** — if a
  decision in your own code depends on a snora colour staying *below*
  some contrast value, that is not a guarantee snora makes; see
  [API governance § Permitted without reopening the gate](../contributing/api-governance.md#permitted-without-reopening-the-gate)
  for the full statement.
- **Non-colour status encoding.** Toast intents and notice tones are
  distinguishable by more than colour alone in snora's prefab widgets.
- **Keyboard reachability inherited from iced.** Native iced controls
  (`button`, `checkbox`, `pick_list`, …) keep iced's own keyboard event
  routing; snora does not intercept it.
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

  **snora does not take Tab or Shift+Tab.** Tab already means "next
  control" to iced and to every application with a form or a text
  field; claiming it for zone cycling would break in-pane navigation
  everywhere. Instead, snora recommends the desktop convention —
  **F6 / Shift+F6** — and supplies a matching key helper,
  `snora::keyboard::cycle_zones`, so you don't have to re-derive which
  modifier means "backward." As with `dismiss_on_escape`, snora installs
  no subscription: your application wires `iced::keyboard::listen()`
  and calls the pure function itself. Cycling is automatically
  suspended while a dialog or sheet is open (focus belongs inside the
  modal), and unaffected by an open menu.

## What snora does not provide

- **No accessibility tree, no AccessKit integration.** iced 0.14 does
  not expose one, and a layout framework cannot supply this on its own.
  snora's stated position — and why it will not build an interim
  abstraction of its own — is in
  [Position on assistive technology](../contributing/semantic-accessibility.md#position-on-assistive-technology-rfc-045).
- **No custom focus ring on snora's own standard buttons or cards** — the
  widgets that let iced own their focus state. This is a hard constraint
  of the pinned iced 0.14 version (`button::Status` has no `Focused`
  variant), not a snora design choice. It does **not** mean focus styling
  is unavailable to your own code: an application that already owns focus
  as its own state can style it in its own `container` closure today,
  using `FocusTokens` for the colour/width vocabulary — see
  [Semantic accessibility § Consequence for the style bridge](../contributing/semantic-accessibility.md#consequence-for-the-style-bridge).
  Full detail on what iced *does* provide (keyboard activation still
  works; only the visual ring on iced-owned widgets is missing) is in the
  [iced 0.14 focus-state limitation](../contributing/semantic-accessibility.md#iced-014-focus-state-limitation).
- **No semantic identifiers or roles.** `iced::widget::Id`s attached to
  snora-rendered surfaces (RFC-047, where implemented) are labels for
  external observation, not accessibility roles or names — an `Id` is
  not a role.
- **No modal focus trapping.** Zone navigation (above) is correctly
  suspended while a dialog or sheet is open, but nothing bounds Tab
  *inside* the modal's own content once focus is there — a keyboard
  user can still Tab out to controls the modal is meant to be gating.
  This is a known, staged gap, not an oversight: it needs iced's
  `advanced` feature, which snora does not enable, and is a separate,
  measured decision. See [design decisions](../contributing/design-decisions.md#why-focus-trapping-is-deferred-v014).

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
