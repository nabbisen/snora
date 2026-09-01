# Migration 0.40 → 0.41

> **Breaking, for a specific and narrow reason: a fix to a real bug.**
> Overlays did not contain pointer events. Clicking or scrolling on top of
> a dialog, a toast, or a modal dim could reach whatever was rendered
> beneath it — including, for a dialog, dismissing itself on a click on
> its own content. 0.41.0 fixes this. If your application depended on the
> old fall-through behavior (deliberately or by accident), that dependency
> stops working. There is no configuration flag to restore the old
> behavior, because the old behavior was the bug.

## Who is affected

Anyone whose application, before 0.41.0, could observe one of:

- A click on a dialog's own content (its padding, its plain text, any
  part that is not itself an interactive widget) dismissing the dialog.
- A modal shown with no `on_close_modals` handler failing to block clicks
  or scroll-wheel input from reaching content beneath it.
- A click or scroll on a toast's own body (not its `×` button) reaching
  whatever was rendered beneath the toast.

If you never relied on any of these — and nothing in snora's own
documentation ever described them as intended behavior — nothing changes
for you except that a class of latent input bugs in your own application
is now closed rather than open.

**Security note.** A modal that does not block input is a UI-integrity
issue: a confirmation dialog could be bypassed by clicking through it to
the control it was meant to be guarding. If your application shows
confirmation dialogs over destructive actions, this fix closes a real gap
— it is worth re-testing those flows specifically.

## What changed

### Dialogs, toasts, and the no-sink modal dim now capture pointer input over their own bounds

Four surfaces should have contained pointer input and only one of them
(the sheet) did:

| Surface | Before 0.41.0 | As of 0.41.0 |
|---|---|---|
| Dialog content | Fell through to the dim backdrop beneath it — a click on the dialog's own text or padding fired `on_close_modals` | Captured; only the dialog's own interactive content responds |
| Modal dim with **no** close sink | Did not capture clicks or scroll at all | Captures both; produces no message, since there is none to produce |
| Modal dim with a close sink | Captured clicks; did **not** capture scroll | Captures both clicks and scroll, both dispatching the same close message |
| Toast body (outside the `×` button) | Fell through to whatever rendered beneath the toast | Captured; only the `×` button's own click still dispatches its message |
| Sheet | Already correct | Unchanged |

Found by an external architect's audit (four findings, F-01 through
F-04) and fixed the same way in each place: `iced::widget::opaque`
wraps the surface's own content, matching the sheet's existing
(and always-correct) implementation.

### Why the no-sink case does not merely omit the block

Before this fix, `docs/src/reference/overlay-interaction-semantics.md`'s
Law 5 said a missing close sink meant *"outside clicks are not
captured."* That was true, and it was also the bug: a missing dismiss
message is not the same question as whether pointer input should reach
through the modal, and conflating them meant an application that
declined to wire `on_close_modals` — to provide its own explicit close
button instead, exactly as Law 5 recommends — got no pointer blocking as
a side effect nobody asked for. Law 5 is corrected to say what is now
true: a missing sink omits the *message*, never the *containment*.

## What did not change

- No z-order change. The layer stack order (skeleton, menus, modal dim,
  dialog, sheet, toasts) is unchanged; only pointer capture within each
  layer's own bounds was added.
- No new public API and no new overlay vocabulary — `opaque` was already
  used in this codebase (by the sheet) for exactly this purpose.
- The sheet's own rendering is untouched; it was already correct.
- Outside-click dismissal (clicking the dim outside a dialog or sheet to
  close it) is unchanged and still works exactly as before.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.

---

# Also in 0.41.0 — widget-layer contrast (RFC-085)

**Originally planned for 0.42.0 and folded into this release.** The split
existed because RFC-085 needed a contrast suite built in a crate that had never
had one, and that must not delay RFC-084's Critical fix. **The suite was
finished before 0.41.0 was cut, so the reason for the split ended** — holding
this back would have cost a release cycle for a condition that no longer
applied.

> **Rendered appearance changes on both the stock and `design` theme
> paths.** Menu text, the sidebar's active highlight, chrome borders, tab
> labels, and breadcrumb text all change color. Every change is a
> contrast repair — text or a border that previously failed WCAG AA (or,
> for borders, the WCAG non-text minimum) now clears it. **If you hold
> reference images or visual-regression baselines that include any of
> these surfaces, they are invalidated**, the same precedent 0.34.0's
> border repair set.

## Who is affected

Anyone whose application uses, in either the default (stock `iced::Theme`)
or `design`-enabled path:

- The prefab header/context menu (`snora::widget::app_header`'s dropdown,
  or `render_menu` directly).
- The prefab sidebar (`app_side_bar`), specifically its active-item
  highlight.
- The prefab header, footer, or tab bar chrome border.
- The prefab tab bar's active-tab label color.
- The prefab breadcrumb's link color.

If your application supplies its own elements for all of these slots
(bypassing snora's prefab widgets entirely), nothing changes for you.

## What changed, and why

**The widget layer paired colours from different token families, and no
contrast suite in this project could see it.** Every contrast assertion
before this release lived in `snora-design` and tested tokens against
roles (`Palette::usages`) — a different, and already-correct, layer.
`snora-widgets` invents its own pairings at render time (a background-tier
colour used as a button's text, for instance), which a role-based suite
has no way to know exist. A new suite in `snora-widgets` itself now
measures every style this crate produces, against the background it is
actually painted over, on both theme paths.

| Widget | Was | Measured (worst case) | Now |
|---|---|---|---|
| Menu text (all states) | `primary.weak`/`.strong`/`.base` — a background-tier family used as text | 1.89:1 (stock light, at rest) | `background.base.text` — 4.5:1+ everywhere |
| Sidebar active highlight vs. rail | `primary.weak.color` | 1.89:1 (stock light); **1.51:1 under `high_contrast_dark`** | `primary.strong.color` — clears the 3.0:1 floor in every preset |
| Sidebar active item's icon/text | `background.base.text` — calibrated for the page background, not the highlight | 1.51–2.13:1 across presets | `primary.strong.text` — iced's own calibrated pairing for the new highlight |
| Chrome border (header/footer/tab bar) | `background.weak.color` | 1.02–1.48:1, every preset and both stock themes | `background.base.text` — clears the 3.0:1 floor everywhere |
| Active tab label | `primary.base.color` | 2.99:1 (stock dark) | `background.base.text` — the active/inactive underline (unchanged) now carries the state distinction instead of label colour |
| Breadcrumb text | `primary.base.color` | 2.03–3.42:1, stock themes | `background.base.text` / `background.weak.text`, matched to the actual background |

**The `high_contrast_dark` preset — the one that exists specifically for
low-vision users — was measured failing worse than every other preset
(1.51:1) before this release.** It is now the *best* of the four design
presets on every corrected pairing (worst case 9.96:1, versus 6.58:1 for
`light` and 8.59:1 for `dark`).

## What did not change

- No `snora-design` token value changed — `git diff -- crates/snora-design`
  for this release is empty. Every fix is a different choice of *which*
  existing, already-correct token the widget layer reads, not a new
  colour.
- No new `Palette` role — `RFC-036`'s frozen token surface is untouched.
- `Palette::usages`'s own contract is untouched; it declares role usage
  and continues to do that well. This release adds a *different*
  suite, in `snora-widgets`, for render-time pairings it was never meant
  to see.
- No public API removed or renamed.

## A visual trade-off, stated plainly

Two of these fixes drop a color-based state distinction that existed
before, because no color from the family being used could clear AA
against the actual background on **both** theme paths at once:

- **Menu items** no longer change color on hover/press — the text color
  is now the same in every state, since the background never changes
  and only one calibrated color is guaranteed safe against it.
- **The active tab's label** is now the same color as an inactive one;
  the underline (unchanged, still the theme's primary color) is the
  state indicator.

Both are candidates for a follow-up that reintroduces the distinction
via a **background** change on hover/press (the pattern the sidebar and
breadcrumb already use) rather than a foreground color — not done here,
to keep this release a contrast repair rather than a visual redesign.

## What is still a known, un-addressed concern

The sidebar's active-item highlight remains the **only** cue that an
item is active on the stock theme path (a WCAG 1.4.1 use-of-colour
consideration, separate from the contrast fix here) — it is far more
visible now than before, but a non-color cue (a border, an icon change)
would still be a more complete fix. Not added in this release, to keep
the change a re-pairing rather than a new visual element; tracked as a
follow-up, not silently dropped.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
