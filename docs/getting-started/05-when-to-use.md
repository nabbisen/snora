# 5 — When to use snora

snora is opinionated. It does a small set of things well and is not
trying to be a general-purpose UI framework. Use this page to decide
whether snora is the right tool *before* you adopt it — that is cheaper
than discovering misfit later.

## A good fit

snora is built for **desktop applications that combine an interactive
UI with non-trivial background work** and want accessibility correct
from day one:

- **Local-first tools.** Apps that run heavy computation on the user's
  machine — local AI inference, search indexers, file processors,
  dataset converters. snora keeps the UI thread responsive while you
  drive `iced::Task::perform` for the heavy lifting.

- **Apps with ABDD as a hard requirement.** RTL support, logical edge
  layout, theme-aware colors are baked into the type signatures. You
  cannot accidentally hardcode "left" or "right" using snora's prefab
  widgets, and the `Edge` / `LayoutDirection` vocabulary makes
  custom widgets equally compliant.

- **Standard desktop chrome.** Apps whose UI fits the
  header / sidebar / body / footer skeleton with optional dialog,
  bottom sheet, context menu, and toast overlays.

- **Small to medium teams** that value reading the framework's source
  occasionally — snora is a few thousand lines, on purpose.

## A workable fit (with caveats)

You can ship with snora, but the framework gives you less leverage:

- **Form-heavy applications.** snora has nothing form-specific (no
  field widgets, no validation primitives). You wire up iced's
  `text_input` / `pick_list` / `checkbox` directly. snora won't
  *fight* you, it just won't help.

- **Multilingual apps with complex typography.** snora handles direction
  but not text shaping or i18n. Pair with
  [`fluent`](https://crates.io/crates/fluent) or similar for messages.

- **Highly bespoke chrome.** If your design language calls for a
  shoulder bar, a multi-row header, a non-rectangular sidebar, a
  vertical tab strip etc., the prefab widgets stop applying. The
  skeleton (`AppLayout`) still works because slots take any
  `Element`, but you write more of the UI yourself.

## A poor fit

- **Games.** snora is built for retained-mode UI; games want
  per-frame re-rendering with their own scene graph. Use iced
  directly, or a game-oriented framework.

- **Real-time visualization.** 3D scenes, live spectrograms,
  60 fps charts. iced's canvas widget can do this without snora's
  overhead, and snora's overlay machinery adds nothing for these
  workloads.

- **Web targets as the primary deliverable.** snora consumes iced
  0.14, which has limited web support. If the web is your main
  target, consider Leptos / Dioxus / Yew.

- **Very small applications.** A single-window calculator with one
  input box and one output gets nothing from snora. Use raw iced.

## Quick decision flow

> Does the app have a header / body / footer kind of layout?
> If no → use raw iced.
>
> Does the app run heavy work alongside an interactive UI?
> If no → snora's structure helps but a simpler iced setup works fine too.
>
> Will more than one person work on this app over months?
> If yes → snora's vocabulary (`AppLayout`, `Toast`, `Dialog`, `SheetSize`)
> gives you shared shorthand. Worth it.
>
> Does the app need to flip LTR ↔ RTL or support multiple themes?
> If yes → snora pays for itself the first time you flip.

When in doubt, write the smallest version of your screen with both
raw iced and snora and compare. snora's value shows up around the
second or third screen, not the first.
