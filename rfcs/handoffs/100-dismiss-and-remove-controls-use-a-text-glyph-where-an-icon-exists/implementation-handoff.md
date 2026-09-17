# Implementation handoff — RFC-100: dismiss and remove controls use a text glyph where an icon exists

**RFC.** `rfcs/accepted/100-dismiss-and-remove-controls-use-a-text-glyph-where-an-icon-exists.md`
— **read its *Rulings* section first**: two facts found after acceptance change
the API shape, and the testing trap is recorded there.
**Release target.** 0.51.0. **Priority.** Low.
**Rulings.** Q-1 both controls · Q-2 `dismiss_tooltip`, plus additive
`removable_with_tooltip` · Q-3 glyph at label size.
**Touch list.** `crates/snora-widgets/src/design/notice.rs`,
`crates/snora-widgets/src/design/chip.rs` (and its `tests.rs` if needed), new
tests under `crates/snora/tests/`, `CHANGELOG.md`. **Not** `crates/snora-design`,
**not** `crates/snora-style`, **not** `docs/` — guides and the migration guide are
the architect's.

---

## Already established, so you do not redo it

- **Measured before any change, all four presets:** `Notice`'s dismiss button is
  **27.00 × 28.20**; the "×" glyph box is 7.00 × 18.20; its box centre sits at the
  same y as the action label's (22.10). There is no WCAG 2.5.8 defect today.
- **orbok's ink-centring observation was not measurable** — the layout harness
  sees boxes, not ink. Do not try to settle it with box centres; RFC-099 showed
  those can agree while the glyph is off.
- **Verified API facts:** `snora_core::Icon::Lucide(lucide_icons::Icon)` and
  `snora_widgets::icon::icon_element_sized(&Icon, size)` exist behind
  `lucide-icons`; `lucide_icons::Icon::X` exists (`'\u{e1b2}'`); lucide renders
  through `Font::with_name("lucide")`; the font bytes are
  `lucide_icons::LUCIDE_FONT_BYTES`; `iced_test::Simulator::with_settings` loads
  `settings.fonts`; iced's `tooltip::Position` is `Top` (default), `Bottom`,
  `Left`, `Right`, `FollowCursor`.
- **`chip::removable` is a free function** with five callers in this repository.
- **No design primitive uses lucide yet** — you are introducing the first
  `#[cfg(feature = "lucide-icons")]` branch in `design/`.

## The trap — read before writing any lucide test

A plain `iced_test::simulator(element)` does **not** load the lucide font. Without
it, `Font::with_name("lucide")` falls back, and the `'\u{e1b2}'` codepoint renders
as whatever the fallback font does with it. **A natural-size-then-centre test
would still pass**: the in-button glyph and the reference glyph would be the
same fallback, measured consistently. That is a test measuring the wrong glyph
and agreeing with itself.

So, for every lucide-feature test:

1. Build the simulator with `Simulator::with_settings`, passing
   `fonts: vec![lucide_icons::LUCIDE_FONT_BYTES.into()]`.
2. **Add a control that fails if the font did not load.** For example, render the
   lucide `X` alone with the font and without it, and assert the two natural
   sizes differ. Or find a metric of the loaded glyph that the fallback cannot
   produce. Show the control failing with the font line removed. Pick whichever
   method you can make fail cleanly, and say why.

## Unit 1 — failing-first tests, before any production change

Cover **both controls × both feature states × all four presets**. Run each
against the current code and **record it failing where it should**.

**(a) The glyph under `lucide-icons`.** Assert the dismiss / remove control
renders lucide `X`, e.g. by finding the `'\u{e1b2}'` text in the rendered tree.
**Today: fails** — only "×" exists. Without the feature, assert "×" is still what
renders (the fallback must be **unchanged**).

**(b) Target ≥ 24 × 24.** Rendered bounds of each control's button, both feature
states. Passes today (27 × 28.2 for notice; chip's RFC-061 fix for chip). Its
failing-first evidence is a **scratch shrink**: zero the notice button's padding,
or reduce chip's computed content width, confirm the test fails, restore
byte-identical.

**(c) Centred.** Natural-size-then-centre, as in `side_bar_fit.rs`. Under
`lucide-icons` this is where the trap above bites — load the font.

**(d) Tooltip.** Hover the control (`point_at` its centre, then a cursor-move
event) and assert the tooltip text appears. Also assert it is **absent** when no
tooltip was given. Today: fails, because the API does not exist yet (a compile
failure counts as failing-first for an absent API; say so).

**Placement and gating** as in RFC-099: in `crates/snora/tests/`, gated
`#![cfg(all(feature = "widgets", feature = "design"))]`, with the lucide cases in
a `#[cfg(feature = "lucide-icons")]` module. CI's matrix already runs
`widgets-design` (text "×") and `widgets-design-lucide-icons` (lucide), so both
states are exercised without a new job. The workspace `.cargo/config.toml`
already forces tiny-skia, so the parallel-simulator crash does not apply; **do
not add a mutex**.

## Unit 2 — the glyph

- `Notice`'s dismiss control and `chip`'s remove control render
  `icon_element_sized(&Icon::Lucide(lucide_icons::Icon::X), label_size)` under
  `#[cfg(feature = "lucide-icons")]`, and the existing
  `text("×").size(label_size)` otherwise. **One helper shared by both**, so the two
  controls cannot drift apart. Put it wherever keeps the `cfg` in one place.
- **Size = the row's label size** (Q-3). No new size constant without a stated
  reason.
- **Keep `chip`'s RFC-061 width mechanism** (`remove_btn_target_size`, the
  computed content width and `center_x`). Put the glyph inside it rather than
  replacing it. Unit 1(b) proves it still clears 24 with the new glyph.
- For `Notice`, centre the glyph within the button. Iced's default button padding
  may do this for a naturally sized child, but Unit 1(c) decides, not the
  assumption.

## Unit 3 — the tooltips

- **`Notice::dismiss_tooltip(self, tooltip: impl Into<String>) -> Self`** — a
  builder method, matching `Notice`'s existing shape.
- **`chip::removable_with_tooltip(tokens, label, selected, on_toggle, on_remove,
  tooltip: impl Into<String>) -> Element`**, with **`removable` delegating to it**
  without a tooltip, so there is one implementation. **Do not change
  `removable`'s signature** — additive only, per the RFC's resolution.
- **Position: `Top` or `Bottom`, never `Left`/`Right`**, because neither primitive
  knows the layout direction. Say which one you chose and why.
- **Rustdoc on both, plainly:** the text is a **visual tooltip** and is **not
  exposed to assistive technology** under iced 0.14, which has no accessible-name
  API for buttons, and snora has no accessibility tree. This sentence is the point
  of Q-2 — orbok's WCAG record cites snora, and a tooltip API it could read as an
  accessible name would become a withdrawn claim.
- **Correct `notice.rs`'s module doc**, which names `.dismiss_label(msg, label)` as
  a future customisation point. It should now point at `dismiss_tooltip` and
  carry the same accessibility caveat. **Check `chip.rs`'s module doc too** — it
  names "a future customization point" for the "×" as well. Correct it the same
  way.

## Criterion 5 — the covenant, quoted not assumed

```bash
git diff <base> -- crates/snora-design crates/snora-style
```

must be empty. `style::button::ghost` and the chip style functions are **used**,
not modified.

## CHANGELOG

- **Added:** `Notice::dismiss_tooltip`, `chip::removable_with_tooltip` — each
  stating that it is a visual tooltip, not an accessible name.
- **Changed:** under **`design` + `lucide-icons` only**, the notice dismiss and
  removable-chip remove controls render lucide `X` instead of text "×". **Visual
  baselines that include either control are invalidated for those applications
  only.** Without `lucide-icons`: unchanged.
- Credit **orbok**.

## Gate suite

The usual, plus the feature matrix locally for at least `widgets-design`,
`widgets-design-lucide-icons` and `--all-features`, and
`cargo +1.88 check --workspace --all-features --locked`.

## Acceptance criteria

1. Under `lucide-icons` both controls render lucide `X`; without it, text "×",
   unchanged — both asserted, both feature states in CI.
2. **The font-loaded control exists and is shown failing** with the font line
   removed.
3. Target ≥ 24 × 24 for both controls, both states, all presets; **shown failing**
   under a scratch shrink.
4. Centring by natural-size-then-centre, with the font loaded.
5. `dismiss_tooltip` and additive `removable_with_tooltip` exist; `removable`'s
   signature is unchanged; the hover test shows the tooltip present and absent;
   the position is direction-neutral.
6. Rustdoc and both module docs state that the tooltip is not an accessible name.
7. `git diff` on `snora-design` / `snora-style` quoted and empty.
8. CHANGELOG as above, crediting orbok. Nothing under `docs/` touched.
