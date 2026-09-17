# Implementation handoff — RFC-101: the toast close button is under the target floor, and still a text glyph

**RFC.** `rfcs/accepted/101-the-toast-close-button-is-under-the-target-floor-and-still-a-text-glyph.md`
— **read its *Rulings* section**; Q-3 was resolved differently from its
recommendation's literal wording, and the sequencing note matters.
**Release target.** 0.51.0, alongside RFC-100. **Priority.** Medium.
**Rulings.** Q-1 (a) the engine renders lucide `X` itself · Q-2 (a) centring
container at a 24 × 24 minimum · Q-3 test-local feature-conditional glyph
constant.
**Touch list.** `crates/snora/src/toast.rs`, `crates/snora/tests/render_semantics.rs`,
new or extended tests under `crates/snora/tests/`, `CHANGELOG.md`. **Not**
`crates/snora-design`, `crates/snora-style`, `crates/snora-widgets` (unless the
drift test needs nothing there, which it should not), and **not** `docs/` — R-3
(the checklist narrowing) and the migration guide are the architect's.

---

## Start after RFC-100 is committed

This work reuses two things RFC-100 introduced in
`crates/snora/tests/dismiss_remove_controls.rs`: the **corner-click
`pointer_target` probe**, which measures what a pointer can actually hit, and
the **`lucide::font_is_loaded` 1em control**. RFC-100 is uncommitted pending its
R-1. **Begin once it lands**, so you build on committed helpers. If sharing them
needs a `crates/snora/tests/common/` module, move them there in this RFC's
commit and say so; that is an approved touch-list expansion.

## Already established, so you do not redo it

- **Measured today** (tiny-skia, `render(AppLayout::new(..).toasts(..))`):
  close button **25.00 × 23.40**, "×" glyph box 9.00 × 23.40. Identical across
  intents.
- **Why it is short:** `button(text("×").size(18)).padding([0, 8])`, which has zero
  vertical padding, so the height is the line box: 18 × 1.3 = 23.4.
- **It is the only interactive control the engine renders itself.**
- `close_button_style` and `toast_style` live in the engine
  (`crates/snora/src/toast.rs`), **not** in `snora_style`. They are outside the
  frozen surface.
- **RFC-093's channel register calls `close_button_style`**
  (`crates/snora/src/toast/channel_register.rs`), canonicalised. Do not change
  that function's output.
- snora's `lucide-icons` feature enables `dep:lucide-icons` **without**
  `widgets`, so `lucide_icons::Icon::X` is reachable from the engine.
  `char::from(lucide_icons::Icon::X)` is how `snora_widgets::icon` obtains the
  codepoint (`'\u{e1b2}'`).
- The toast's text column (title 16, message 14, spacing 4) is about **43 px**
  tall, so a 24 px button should not grow the toast. **Confirm by measurement.**
- `render_semantics.rs` clicks the close button by text at **lines 408 and 504**.

## The traps, both already paid for

**Font loading is process-global, and a missing font passes silently** (RFC-100
§3). Build every lucide simulator with the font loaded, and keep the 1em control
in this test binary. Show the control failing with the font line removed.

**A fixed-size box centres its text box, not the glyph** (RFC-099). If the
container gives the glyph fixed limits, the text box fills it and the glyph draws
at its top-left. That applies to lucide's 18 × 18 glyph in a 24 px button in
particular. **Test centring by natural-size-then-centre, never by box centres.**

## Unit 1 — failing-first tests

Both feature states (text "×" / `lucide-icons`), at least two intents:

**(a) Target ≥ 24 × 24**, via `pointer_target`. **Today: fails** (height 23.40).
Record it.

**(b) Centred**, natural-size-then-centre, font loaded under lucide.

**(c) The glyph.** Under `lucide-icons`, the close button draws `'\u{e1b2}'`;
without it, `"×"`, **unchanged**. **Today, under lucide: fails.**

**(d) Engine and `snora-widgets` do not drift (Q-1).** Gated
`all(feature = "widgets", feature = "design", feature = "lucide-icons")`. Render a
toast and a dismissible `Notice` through **public API only**, and assert that
**both trees contain `'\u{e1b2}'`** and that **both glyphs are exactly 1em wide
at their own size**. That proves both use the lucide font rather than a fallback.
It fails if either site changes codepoint or loses the font. Under
`--all-features` (CI) it runs. Say which scratch edit you used to show it failing,
e.g. the engine's `.font(..)` removed.

## Unit 2 — the target and the glyph

- **Q-2 (a):** wrap the glyph in a container that **centres it on both axes** at a
  minimum of **24 × 24**, inside the button. Name the 24 as the floor it is, in a
  `const` with a comment citing WCAG 2.5.8 and snora's checklist. Keep the
  horizontal padding unless measurement says otherwise. **Report the final
  measured size in both feature states.**
- **Q-1 (a):** under `#[cfg(feature = "lucide-icons")]`, render
  `text(char::from(lucide_icons::Icon::X).to_string()).size(18).font(Font::with_name("lucide"))`.
  Otherwise use today's `text("×").size(18)`. Add a comment at this site pointing
  at `snora_widgets::icon::icon_element_sized`, and one there pointing back here:
  **two sites that must agree**, guarded by Unit 1(d).
- **Colour:** leave it to `close_button_style`, through the button's text colour,
  and **add no explicit glyph colour**. Unlike RFC-100's notice there is none to
  preserve. **Verify the inheritance rather than assume it**, as evidence, not as
  a permanent test. Under `lucide-icons`, render the close button as built, then
  again with the glyph explicitly coloured to `close_button_style(..)`'s
  `text_color` for the same intent and status. **The idle frame hashes must be
  equal**, which shows the lucide glyph takes the button's text colour exactly as
  "×" did. Confirm RFC-093's channel register still passes.

## Unit 3 — the two `render_semantics` tests (Q-3)

Replace `ui.click("×")` at both sites with a **test-local**
feature-conditional constant:

```rust
#[cfg(feature = "lucide-icons")]
const TOAST_CLOSE_GLYPH: &str = "\u{e1b2}";
#[cfg(not(feature = "lucide-icons"))]
const TOAST_CLOSE_GLYPH: &str = "×";
```

with a comment that these tests assert **reachability**, and that the glyph is
asserted by Unit 1(c). **Do not add a public item** to share it (see the RFC's
Q-3 ruling). If `render_semantics.rs`'s lucide run needs the font loaded to
*find* the glyph, check whether `find`/`click` locates text by content
independently of the font. If it cannot, load the font there too, and say which
was true.

**Both tests must still pass in both feature states**, and each must still fail
for the defect it guards. Show one of them (e.g. the above-modal test) failing
against a scratch edit that breaks reachability, **after** the selector change,
so the new selector is proven to be finding the button.

## Appearance — measure, report, do not guess

The height change reaches **every application with toasts**. Report, per
feature state:

- whether **idle** frames of a toast change (hash comparison, old vs new build);
- whether **hovered** frames change;
- the final button size, and that the **toast's own size** is unchanged.

These go verbatim into the migration guide, so the guide states facts rather than
the RFC's predictions.

## Covenant

```bash
git diff <base> -- crates/snora-design crates/snora-style
```

quoted and empty.

## CHANGELOG

- **Fixed:** the toast close button's pointer target was **25.00 × 23.40**, under
  snora's mandated 24 × 24, on the default path; now ≥ 24 × 24 (state the measured
  size). Say plainly that snora's height assertion covered token-derived controls
  only and this control used literals. Say that no published claim covered it,
  and that whether a given application failed WCAG 2.5.8 depends on its layout
  (the spacing exception).
- **Changed:** lucide `X` on the toast close button under `lucide-icons`,
  matching RFC-100's notice and chip; text "×" otherwise.
- Opened from RFC-100's review, on the owner's instruction; the observation came
  from the dev team.

## Gate suite

The usual, plus `cargo test -p snora` under default features,
`--no-default-features --features widgets,design` (RFC-100's R-1 step), and
`--all-features`; `cargo +1.88 check --workspace --all-features --locked`; RFC-093's
channel register tests.

## Acceptance criteria

1. Target ≥ 24 × 24, both feature states, **shown failing against 23.40**.
2. Centred by natural-size-then-centre; the 1em font control is present under
   lucide and **shown failing** without the font.
3. Lucide `X` under the feature, "×" otherwise; the drift test (1d) exists and is
   **shown failing**.
4. Both `render_semantics` toast tests pass in both states via a test-local
   constant, with **no new public item**; one is shown still able to fail after
   the selector change.
5. `close_button_style` output unchanged; RFC-093's register passes.
6. Appearance measured and reported (idle / hovered / toast size).
7. Covenant diff quoted and empty; CHANGELOG as above; nothing under `docs/`.
