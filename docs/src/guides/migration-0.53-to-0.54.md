# Migration 0.53 → 0.54

> **Toasts stay as long as they take to read, and errors stay until they are dismissed.
> The sidebar names its items on the rail.** An application can now choose iced's
> features itself. Two deprecations warn, but nothing is removed. Read §6 if your
> project keeps an accessibility record.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| `Toast::new` | Errors are persistent; other toasts last as long as their text takes to read (≥ 6 s, not 4 s) | Nothing, or wire the pointer hold (§1) |
| `snora::toast::sweep_expired` or `ToastLifetime::DEFAULT` | **Deprecated**: warns, still works | Move to `Holds::sweep` / `reading_time` (§1). Builds that deny warnings must move |
| `app_side_bar` | The rail shows each item's **name**, beneath its icon, and is wider for long names | Retake screenshot baselines; `app_side_bar_with(…, SideBarLabels::Tooltip)` keeps the old rail (§2) |
| `default-features = false` on `snora`, `snora-widgets` or `snora-style` | Also turns off iced's defaults | Nothing, if your own `iced` line keeps its defaults (§3) |
| The active sidebar item, on Solarized Light/Dark, Tokyo Night Light or Catppuccin Latte | Its text and icon colour change to clear 4.5:1 | Retake baselines on those themes (§2) |
| `design` presets | Two new AAA presets, `light_aaa` and `dark_aaa`, plus `Tokens::presets()`; `light` and `dark` unchanged | Nothing, or adopt them (§4) |

**For code:** nothing breaks to compile, except the one narrow iced case in §3, and
builds that deny warnings, which see the two deprecations.

## 1. Toasts (RFC-113)

**How long a toast stays,** a rule you can apply by hand:

> *A toast stays 2 seconds, plus 1 second for every 6 characters of its title and
> message, rounded up to a whole second, and never less than 6 seconds.*

- "Saved" / "Your changes are stored." is 29 characters, so 7 s.
- It counts **characters, not words**, so Japanese and Chinese text gets the same
  margin.
- **`Toast::new` with `ToastIntent::Error` is now `Persistent`:** a failure stays
  until the reader dismisses it.
- `.with_lifetime(…)` and `.persistent()` still override.

**The pointer hold.** A toast does not expire while the pointer rests on it, and its
countdown restarts in full when the pointer leaves. Wire it in three lines:

```rust,ignore
// state
holds: snora::toast::Holds,

// view
layout.on_toast_hover(Message::ToastHover)

// update
Message::ToastHover(id, crossing) => self.holds.hover(id, crossing, &mut self.toasts, Instant::now()),
Message::ToastTick               => self.holds.sweep(&mut self.toasts, Instant::now()),
```

`crossing` is `snora::ToastHover::{Entered, Left}`. **A keyboard user has no hold:**
iced 0.14 cannot tell snora which widget has focus. Use `.persistent()` for anything
a reader must not miss.

**Deprecated, still working:**

| Was | Now |
|---|---|
| `snora::toast::sweep_expired(&mut toasts, now)` | `holds.sweep(&mut toasts, now)`, which also keeps held toasts |
| `ToastLifetime::DEFAULT` | `ToastLifetime::reading_time(title, message)`. `DEFAULT` keeps its 4 s value, but is no longer what `Toast::new` uses |

## 2. The sidebar names its items (RFC-114)

`app_side_bar` used to show icons only, with each name in a tooltip that appears only
under a resting pointer. **Now the name is on the rail, beneath the icon.**
- The rail is 64 px for short names, widening for longer ones, and capped at 240 px.
- Names wrap rather than clip.

**Choose the form** with `app_side_bar_with(side_bar, direction, labels)`, or
`design::widget::app_side_bar_with(tokens, …)`:

| `SideBarLabels::` | Item |
|---|---|
| `Beneath` (the default) | icon, with the name beneath it |
| `Beside` | icon, then the name, mirrored under RTL |
| `Tooltip` | the pre-0.54 icon rail. It names items only under a resting pointer, so touch users and anyone with an unsteady hand get none |

**The active item's colours** now always clear 4.5:1, because a name is text. On four
of iced's built-in themes the theme's own pair missed, so the text and icon use black
or white instead:

| Theme | Before | After |
|---|---|---|
| Solarized Light, Solarized Dark | 3.93 | 4.65 |
| Tokyo Night Light | 4.23 | 5.07 |
| Catppuccin Latte | 4.27 | 5.13 |

Stock Light and Dark, and every design preset, are unchanged.

## 3. Choosing iced's features yourself (RFC-112)

**Until 0.53, snora turned on all of iced's default features for every application,**
and Cargo features only add up. So `default-features = false` on your own `iced` line
did nothing. Now `snora`, `snora-widgets` and `snora-style` take iced without
defaults, and pass them through a default-on feature, `iced-default`. A plain
`cargo add snora` is unchanged.

**The rule:** iced features that change *how snora draws* (`web-colors`, `crisp`) are
snora's, and always on. Everything else is yours. The recipe, for example to drop
`linux-theme-detection` and the `zbus` advisories behind it, is in
[feature flags](../design/feature-flags.md#choosing-iceds-features-yourself-iced-default-rfc-112).

**The one case that stops compiling:**
- `default-features = false` on a snora crate;
- *and* an `iced` line of your own that also turns defaults off and names no
  windowing backend.

winit then refuses the platform. Add `iced-default`, or name iced's features.

## 4. AAA presets (RFC-118)

`Tokens::light_aaa()` and `Tokens::dark_aaa()` hold **every declared text pair at
7:1** (WCAG AAA), keeping Light's and Dark's background and surfaces. Only text
roles, and fills whose own text needed it, move, in lightness only, with hue within
6°. `Tokens::presets()` lists every preset, for a theme picker for instance:

| Preset | Text | For |
|---|---|---|
| `light`, `dark` | AA, 4.5:1 | general use |
| `light_aaa`, `dark_aaa` | AAA, 7:1 | Light's or Dark's character at AAA text, e.g. older readers |
| `high_contrast_light`, `high_contrast_dark` | ≥ 8.38:1, borders 21:1 | maximum contrast |

With `text_muted` raised to 7:1, colour carries less of the text hierarchy. Let size
and weight (`Typography`) carry it.

## 5. Fixed without a visible change

- **The accent's use as text is now guarded** (RFC-117 R-5). Ghost button labels draw
  in the accent, which was declared only as a non-text colour (3:1). It is now also
  declared as text (4.5:1). Every preset already cleared it: no colour changed.

## 6. For accessibility records: claims corrected

**iced 0.14's `button` takes no keyboard input and is not focusable,** so **no snora
control can be reached from the keyboard today.** That covers the sidebar, tabs,
crumbs, toast and notice dismiss buttons, chips, menu items and `design::button`.

The book and the rustdoc said otherwise in several places. All are corrected with
dated notes:
- "Enter / Space activates a focused button";
- "keyboard reachability inherited from iced";
- "keyboard reachable" in the recipes and the primitives page;
- "a tooltip serves keyboard and screen-reader users";
- "Tab already means next control to iced". iced binds no Tab.

**If your record cites any of these, it is wrong. Cite this section instead.**

**The plan** is RFC-116, tracked as 1.0 gate 11: a focusable control under every snora
control, with a focus ring, needing no iced feature. It is planned for 0.55. Until
then, give every action that matters a keyboard shortcut of your own.

## One question for adopters

Does your application serve **mainly older users**? orbok's does, and their evidence
shaped this release and the next. Size rules that follow the text, and a minimum
target size you can raise to 44 px, are coming in 0.55 (RFC-115). If yours does too,
tell us what you measure.

## What did not change

- No public API was removed or renamed. MSRV is still 1.88.
- `light`, `dark` and the high-contrast presets have the same colours.
- iced's own default features are unchanged for anyone who keeps snora's defaults.

## If you are jumping more than one minor

**0.53** made the modal dim visible on dark themes, underlined ancestor breadcrumbs,
and withdrew the "unchanged from v0.25" guarantee. The
[migration index](migrations.md) lists every step.
