# RFC-100 — Dismiss and remove controls use a text glyph where an icon exists

**Status.** Done — shipped v0.51.0. Q-1 = both, Q-2 = (a), Q-3 = label size.
**Raised.** 2026-09-17, architect, on a request from **orbok**.
**Release target.** 0.51.0.
**Priority.** Low. orbok's own words: *"No hurry. It is cosmetic, and the control
works."* Measurement below agrees.

---

## The request

orbok has replaced every text glyph it used as an icon with a lucide glyph. In a
`Notice` row, the dismiss control is the one text glyph left on screen:
`button(text("×").size(label_size))` in `design/notice.rs`. They ask for:

1. **Lucide `X` for the dismiss control when `lucide-icons` is enabled**, sized and
   centred like the other icon buttons, with the text "×" kept as the fallback.
2. **The `.dismiss_label(..)` customisation point the module doc already names**,
   so the control can carry a localised label — they would pass "Dismiss" /
   「閉じる」 — *"for a tooltip or assistive technology"*.

They are not working around it, so whatever ships is what they will see.

## What was measured, and what could not be

A rendered probe, using the `iced_test` harness RFC-099 established, rendered
`Notice` with a title, an action and a dismiss control in all four presets.
Every preset gave identical results:

| | Measured |
|---|---|
| Dismiss button | **27.00 × 28.20** px |
| "×" glyph box | 7.00 × 18.20 px |
| Glyph box centre vs. action label box centre (y) | 22.10 vs 22.10 — **identical** |

**It is not a conformance defect.** 27.00 × 28.20 clears WCAG 2.5.8's mandatory
24 × 24. It sits below the preferred 44 × 44, which is advisory. So orbok's
"small as a click target" is fair as a comparison, not as a violation. Unlike
`chip`'s remove "×", which RFC-061 found at 15 px wide, the notice's control is
carried over the floor by iced's default 10 px horizontal button padding.
**No conformance claim moves.**

**The centring observation could not be measured, and is recorded as orbok's.**
The two text *boxes* centre at exactly the same y. orbok's point is about **ink**:
the "×" shape sits wherever the text font's metrics place it inside an 18.2 px
line box, and that differs from a lucide glyph's placement. The layout harness
measures boxes, not ink. RFC-099 already showed that a box can be centred while
the glyph inside it is not, so a matching box centre is not evidence either way.
The claim is plausible from font metrics and is not verified here.

## The scope is wider than the request

**`chip`'s remove control has the same text "×".** orbok mentions using lucide `X`
"to remove a chip" in their own code. If `Notice` switched to lucide and `chip`
did not, a lucide-enabled application would show a lucide `X` in one snora
primitive and a text "×" in the next. That is the inconsistency orbok just
finished removing from their own UI.

The plumbing already exists: `snora_core::Icon::Lucide` and
`snora_widgets::icon::icon_element_sized` render lucide glyphs under the
`lucide-icons` feature. **No design primitive uses them yet.**

## The trap in ask 2

The module doc names `.dismiss_label(msg, label)` as a future customisation
point, and states the constraint in the same paragraph: *"iced 0.14 does not
expose a separate accessible label for buttons."* snora has no accessibility tree
(`accessibility.md`; iced 0.14 has no AccessKit integration).

**So a label passed today can reach a tooltip, and cannot reach assistive
technology.** orbok's WCAG 2.1 AA conformance record cites snora nineteen times
and has absorbed three withdrawn claims already (`text_muted` at 0.34.0, the
dialog-card border at 0.39.0, and 1.4.1 at 0.41.1). An API named `dismiss_label`
that a record then cites as an accessible name would be a fourth, planted at
birth.

## Proposal

**R-1 — use the icon when it exists.** Under `lucide-icons`, `Notice`'s dismiss
control and `chip`'s remove control render lucide `X` through the existing
`icon_element_sized`, centred in the button. Without the feature, both keep the
text "×", unchanged.

**R-2 — keep the target at or above 24 × 24, measured.** The switch changes glyph
metrics under both controls. `chip`'s RFC-061 width fix was computed from the
text glyph's advance, and it must still clear the floor with a lucide glyph.
Assert both controls' rendered bounds in both feature states with the `iced_test`
harness, as `accessibility-checklist.md` now directs.

**R-3 — a localised tooltip, named so it cannot be mistaken for an accessible
name.** Shape is Q-2.

## What this is not

- **Not a covenant change.** `notice` and `chip` are design primitives, and
  RFC-036 excludes those from its frozen surface. They follow the promotion
  lifecycle instead. Nothing in `snora-design` or `snora_style` changes;
  `style::button::ghost` is used, not modified. D-3/D-4 are unaffected, and the
  handoff should require the empty diff to be quoted, as RFC-099's did.
- **Not an appearance change for most applications.** Only applications that
  enable **both** `design` and `lucide-icons` see anything move. Their visual
  baselines that include a notice dismiss control or a removable chip are
  invalidated, and the migration guide must say so in those terms.
- **Not an assistive-technology feature**, whatever R-3 is called.

## Open questions

**Q-1 — `Notice` only, as asked, or `Notice` and `chip` together?**
Suggest **both**. A lucide-enabled application would otherwise see snora
inconsistent with itself, and the cost is one additional control using the same
helper. `chip` carries the only extra risk, RFC-061's computed width, which R-2
measures.

**Q-2 — what to call the tooltip, and whether to ship it at all?**

| Option | Shape | Risk |
|---|---|---|
| **(a) `.dismiss_tooltip(..)`** | A tooltip text, named for what it does | None; the name makes no accessibility claim |
| (b) `.dismiss_label(..)`, as the module doc named it | Same behaviour | "Label" reads as an accessible name; a conformance record can cite it as one |
| (c) Defer | Nothing until iced exposes accessible names | orbok's localisation need goes unmet for a hint that is useful today |

Suggest **(a)**, and **amend the module doc's `.dismiss_label` sentence in the
same change**. The doc advertises a name this RFC declines. Its rustdoc must say
plainly that the text is a visual tooltip and is not exposed to assistive
technology under iced 0.14.

If `chip` is in scope (Q-1), give its remove control the matching
`.remove_tooltip(..)`, so the two primitives stay symmetric.

**Q-3 — how large should the glyph be?** "Like your other icon buttons" cannot
mean the sidebar's 48 × 48: that is a rail control, and it would bloat an inline
notice row. Suggest sizing the glyph to the row's label size and keeping the
target at or above 24 × 24 through the existing padding and width mechanisms.
The handoff should not introduce a new size constant without a stated reason.

## Acceptance criteria

1. Under `lucide-icons`: both controls render lucide `X`. Without it: text "×",
   **unchanged**. Both feature states are exercised in CI's feature matrix
   (`widgets-design-lucide-icons` and `widgets-design` already exist).
2. Rendered-bounds tests show both controls clearing 24 × 24 in both feature
   states and all four presets, and **fail when the target is shrunk below the
   floor** in a scratch edit.
3. The glyph is centred in its button, tested by the **natural-size-then-centre**
   method from RFC-099. A box-centre comparison alone is not acceptable, because
   RFC-099 showed it cannot fail.
4. Q-2's tooltip exists as ruled; its rustdoc states it is not an accessible
   name; the module doc's `.dismiss_label` sentence is corrected.
5. `git diff` on `crates/snora-design` and `crates/snora-style` quoted and empty.
6. CHANGELOG: an appearance change for `design` + `lucide-icons` users only,
   crediting orbok; migration guide says which baselines move.

---

## Rulings, 2026-09-17

**Q-1 — both.** `Notice`'s dismiss control and `chip`'s remove control change
together.

**Q-2 — (a).** The localised text is a **tooltip**, named for what it does. The
module doc's `.dismiss_label` sentence is corrected in the same change.

**Q-3 — label size.** The glyph is sized to the row's label size. The target
stays at or above 24 × 24 through the existing padding and width mechanisms, and
no new size constant is introduced without a stated reason.

### Two facts found after acceptance, and how they are resolved

**`chip::removable` is a free function, not a builder.** Its signature is
`removable(tokens, label, selected, on_toggle, on_remove) -> Element`. The
`.remove_tooltip(..)` *method* that Q-2's recommendation described cannot exist
on it; that recommendation assumed a builder shape without checking.
`removable(` has 5 call sites in this repository alone, and changing its
signature would break those and every consumer's. **Resolved by the architect
within Q-2's ruled intent — a matching, honestly named tooltip on both
controls:** add **`removable_with_tooltip(tokens, label, selected, on_toggle,
on_remove, tooltip)`**, and make `removable` delegate to it with no tooltip, so
one implementation serves both. The change is additive. The alternatives were a
breaking parameter or turning `removable` into a builder, and each costs every
caller a migration for a low-priority, cosmetic RFC. The owner may overrule
this.

**The tooltip position must not depend on layout direction.** Neither
primitive receives a `LayoutDirection`. Of iced's `tooltip::Position` variants,
`Left` and `Right` would be wrong under one direction or the other. Use `Top`
or `Bottom`, and state which.

### A trap in testing lucide glyphs, recorded because it would pass silently

Lucide glyphs render through `Font::with_name("lucide")`, which resolves only
once `lucide_icons::LUCIDE_FONT_BYTES` has been loaded. A plain
`iced_test::simulator(element)` never loads it. **A test that forgets would
measure a fallback glyph, and a natural-size-then-centre comparison would still
pass**, because the in-button glyph and the reference glyph would both be
missing from the font in the same way. The handoff therefore requires loading
the font through `Simulator::with_settings` and a control proving the lucide
glyph was actually rendered.
