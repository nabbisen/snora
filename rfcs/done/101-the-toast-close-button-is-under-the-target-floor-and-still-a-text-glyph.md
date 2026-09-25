# RFC-101 — The toast close button is under the target floor, and still a text glyph

**Status.** Done — shipped v0.51.0. Q-1 = (a), Q-2 = (a), Q-3 = test-local constant (see Rulings).
**Raised.** 2026-09-17, architect. **Opened on the owner's instruction**, from an
observation in RFC-100's review: the dev team noticed the engine's toast close
button is also a text "×".
**Release target.** 0.51.0, alongside RFC-100.
**Priority.** Medium. It began as a consistency question and **measurement found
a target-size defect on the default path.**

---

## What was measured

The consistency question comes first, but the measurement changed this RFC's
weight. The toast was rendered through `render(AppLayout::new(..).toasts(..))` in
`iced_test`, under the workspace's tiny-skia configuration:

| | Measured |
|---|---|
| Toast close button | **25.00 × 23.40** px |
| "×" glyph box | 9.00 × 23.40 px |

Identical for `ToastIntent::Info` and `Error`. Intent changes colour only.

**The button is 0.6 px shorter than 24.** It is built as
`button(text("×").size(18)).padding([0, 8])` in `crates/snora/src/toast.rs`,
with **zero vertical padding**, so its height is just the text's line box: 18 × 1.3
= 23.4.

### Why nothing caught it

- **The height assertion is scoped to token combinations.**
  `pointer_target_height_meets_24px_for_every_role_and_padding_step` checks every
  `TextRole` × `Spacing` pair, and `accessibility-checklist.md` calls the height
  axis *"mechanically asserted"*. That is true of what it covers. **This button is
  not in it**: size 18 is a literal, not a `TextRole`, and zero padding is not a
  `Spacing` step. It is the one snora-owned control whose height was never inside
  the assertion.
- **RFC-061's scope was the design primitives** (it fixed `chip`'s remove width).
  The engine's toast was never examined.
- **It is the only interactive control the engine renders itself.** Dialog, sheet
  and menu wrap content the application supplies. So "the engine's controls meet
  the floor" was a claim about a single button, and nothing checked it.

### What this does and does not mean for conformance

- **snora's own standard is not met.** The checklist mandates 24 × 24 for snora's
  controls, with no exceptions.
- **Whether a given application fails WCAG 2.5.8 is not settled by this
  measurement.** 2.5.8 excuses an undersized target when a 24 px circle centred on
  it intersects no other target. A lone toast close button inside 12 px of toast
  padding may qualify. Stacked toasts, or a layout that places targets close, may
  not. **snora should not rely on an exception that depends on the application's
  layout.**
- **No published snora claim is falsified.** No page says the toast close button
  meets 24 × 24, and the one assertion that exists says truthfully what it covers.
  **But orbok's record rests on our documentation for 2.5.8**, and the release
  process marks them *"Tell them first."* They should hear this from us when it
  ships, whatever the wording of our docs.

## The original question: the glyph

After RFC-100, an application with `lucide-icons` gets lucide `X` on notice
dismiss and chip remove, and a text "×" on every toast. The consistency argument
that brought `chip` into RFC-100 applies unchanged.

**This is harder than RFC-100 in three ways:**

1. **The engine cannot use RFC-100's helper.** `close_glyph` and
   `icon_element_sized` live in `snora-widgets`. The engine depends on it only
   under `widgets`, while `toast.rs` renders in engine-only builds too. snora's
   `lucide-icons` feature does pull in `lucide-icons` directly, without `widgets`.
2. **Two `render_semantics.rs` tests click the close button by its text**
   (`ui.click("×")`, at lines 408 and 504). CI runs
   `cargo test -p snora --all-features`, so the moment the glyph becomes lucide
   under that feature, both tests stop finding their target.
3. **RFC-093's channel register reads `close_button_style`** to assert what the
   toast's colour channel carries. The glyph change must not alter that style, and
   the register must still pass.

## Proposal

**R-1 — bring the close button to 24 × 24, measured.** Keep the glyph at its
current size and give the button a height of at least 24, with the glyph centred.
The rendered text column is about 43 px tall, so **the toast itself does not grow**.
Assert it with a rendered target test in both feature states, reusing RFC-100's
corner-click `pointer_target` probe. Show that test failing against today's
23.40.

**R-2 — lucide `X` under `lucide-icons`, text "×" otherwise.** The shape is Q-1.
Colour stays with `close_button_style`, and RFC-093's register must still pass.

**R-3 — close the gap in the checklist's claim.** `accessibility-checklist.md` says
the height axis is mechanically asserted. It should say what that covers,
token-derived controls, and that **a snora-owned control with literal sizing must
carry its own rendered target test**. The toast close button becomes the example,
as `app_side_bar` became the example for width in RFC-099.

## What this is not

- **Not a covenant change.** `close_button_style` and `toast_style` are in the
  engine (`crates/snora/src/toast.rs`), not in `snora_style`. `snora-design` and
  `snora-style` are untouched, and D-3/D-4 stand.
- **Not a tooltip.** Toasts are transient and expire. Nobody asked for one, and
  RFC-100's tooltip was for persistent controls.
- **Not a change to toast text sizes** (16 and 14 remain literals). That is a
  separate question about whether the engine should read typography tokens, and
  the engine renders without `design`.

## Appearance

- **R-1 changes every application that shows toasts**, on the default path. The
  close button grows from 23.40 to at least 24 px tall, and the toast's size does
  not change. Idle frames may be pixel-identical if the button's idle background
  is transparent. Hover and pressed frames will not be. **The implementation
  measures this and reports it**, and the migration guide states it plainly
  rather than guessing in advance.
- **R-2 changes only applications with `lucide-icons`.**

## Open questions

**Q-1 — how does the engine render the lucide glyph?**

| Option | Shape | Cost |
|---|---|---|
| **(a) Render it in the engine** | `text(char::from(lucide_icons::Icon::X)).font(Font::with_name("lucide"))` under snora's `lucide-icons` | A few lines duplicating what `snora_widgets::icon` does. Works in engine-only builds |
| (b) Only with `widgets` too | Use `snora_widgets::icon` when present, "×" otherwise | An application with `lucide-icons` but not `widgets` keeps "×" — a surprising feature interaction |
| (c) Share a helper below both crates | Move lucide rendering down | Nowhere to put it: `snora-core` is iced-free by CI gate |

**Suggest (a).** Make the duplication honest: a comment at each site pointing at
the other, and **a test asserting both sites render the same codepoint in the
same font**, so they cannot drift silently. Whether that test can run depends on
`widgets` being present, and the handoff should say how it is gated.

**Q-2 — how does the button reach 24?**

| Option | Shape | Risk |
|---|---|---|
| **(a) A centring container at a minimum size** | The mechanism RFC-061 (`chip`) and RFC-099 (sidebar) already use | One more literal (24), stated as the floor it is |
| (b) Vertical padding | e.g. `[1, 8]` → 25.4 | Tied to iced's default 1.3 line height; a line-height change silently breaks it |

**Suggest (a).** It is the mechanism this codebase already uses twice. It does not
depend on a font metric, and it centres the lucide glyph too. Its 1em box, 18 × 18,
would otherwise sit at the top of a 24 px button, which is exactly RFC-099's defect.

**Q-3 — how do the two `render_semantics` tests find the button once the glyph
can change?** Suggest they select the glyph through a feature-conditional
constant shared with the implementation. For *those* tests that is not
tautological: they assert reachability above a modal and under RTL, not which
glyph is drawn. The glyph itself is asserted separately, by R-2's own test.

## Acceptance criteria

1. The toast close button is **at least 24 × 24** by rendered measurement, in both
   feature states, **shown failing against today's 25.00 × 23.40**.
2. The glyph is centred, by natural-size-then-centre. Under `lucide-icons` the
   font is loaded and RFC-100's 1em control is present, since the process-global
   font trap applies here too.
3. Lucide `X` under `lucide-icons`, text "×" otherwise. If Q-1 (a), a test shows
   the engine and `snora-widgets` render the same codepoint and font.
4. Both `render_semantics` toast tests still assert what they assert, in both
   feature states. RFC-093's channel register still passes.
5. `accessibility-checklist.md` narrows "height is mechanically asserted" to
   token-derived controls and names the literal-sized case (R-3).
6. `git diff` on `crates/snora-design` and `crates/snora-style` quoted and empty.
7. CHANGELOG under **Fixed** (target) and **Changed** (glyph under
   `lucide-icons`). The migration guide states the measured appearance change,
   and **orbok is told directly at release** (the release process's
   *"Tell them first"*).

---

## Rulings, 2026-09-17

**Q-1 — (a).** The engine renders lucide `X` itself under snora's `lucide-icons`
(`text(char::from(lucide_icons::Icon::X)).font(Font::with_name("lucide"))`, the
same construction `snora_widgets::icon` uses). Each site carries a comment
pointing at the other, and a rendered test shows both draw the same codepoint in
the lucide font.

**Q-2 — (a).** The button reaches 24 × 24 through a centring container at a
minimum size, the mechanism RFC-061 and RFC-099 already use, not through vertical
padding tied to iced's 1.3 line height.

**Q-3 — the recommendation's intent, in a form that does not add public API.**
The recommendation said the two `render_semantics` tests should select the close
glyph through *"a feature-conditional constant shared with the implementation"*.
**From an integration test under `crates/snora/tests/` that is impossible without
a public item**, because those tests see only snora's public API. Adding one for a
test's convenience is the thing RFC-099's review declined: its fit test mirrored
`RAIL_WIDTH` and `BUTTON_SIZE` rather than making them public. **Resolved by the
architect within the ruling's intent — keep the tests asserting reachability
whichever glyph is drawn:** a **test-local** `#[cfg(feature = "lucide-icons")]`
constant (`"\u{e1b2}"`) with a `"×"` fallback. For these tests it is not
tautological, because they assert reachability, and R-2's own test asserts the
glyph. The owner may overrule this.

### Sequencing

RFC-101's tests should reuse RFC-100's corner-click `pointer_target` probe and its
1em font control, which live in `crates/snora/tests/dismiss_remove_controls.rs`.
**RFC-100 is not yet committed** (its R-1 is pending). RFC-101's implementation
starts after RFC-100 lands.
