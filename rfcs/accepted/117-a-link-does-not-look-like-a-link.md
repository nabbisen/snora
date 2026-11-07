# RFC-117 — A link does not look like a link

**Status.** Accepted 2026-11-04. Every open question ruled as suggested (see Rulings). **R-5 shipped in v0.54.0;** `link` ships in 0.55.0 with RFC-116 R-1.
**Handoff.** [`handoffs/117-…`](../handoffs/117-a-link-does-not-look-like-a-link/implementation-handoff.md).
**Revised** 2026-11-04, before acceptance, after the architect reviewed it against
the owner's fundamental philosophy (see "Review against snora's principles").
**Raised.** 2026-11-04, architect, from orbok's request 3
(`.git-exclude/upstream/receive/0.54.0/orbok-snora-request-0.54.0-easy-to-read-2026-11-04.md`).
The owner approved opening it. Theme E (older users).
**Release target.** 0.54.0 (Q-4). **Priority.** An improvement requested by an adopter
for older users, and small.

---

## The finding

**snora has no link control.** orbok uses `design::button::ghost` as an inline link
("Choose another folder"). The ghost style (`snora-style/src/button.rs:138`) is
accent-coloured text with no fill or border at rest, and **no underline**. Checked.

**Older users do not read coloured text as clickable.** An underline is the
convention for "this is a link". Without it, the only cue is colour, which is the
cue that fails first as eyesight changes. WCAG 1.4.1 (Use of Color) asks for a cue
besides colour when colour marks a link within text.

**The accent is drawn as text, but declared as non-text.** `style::button::ghost`
sets the label's `text_color` to `palette.accent`. That is text, which carries the
4.5:1 floor. But `Palette::usages` (RFC-063) declares `accent` only as
`ThresholdClass::NonText` (3:1), for the filled primary button. Measured today, the
accent clears 4.5:1 on every preset's neutral surfaces (lowest 5.63, Dark
`surface_raised`). So nothing is broken now, **but nothing guards it.** A palette
edit that took the accent to 4.0:1 would pass the gate while every ghost label failed
AA. A link would inherit the same gap.

**snora already draws one underline, and knows how.** RFC-109 gave ancestor
breadcrumbs a 1 px rule inside the button, in the label's colour, snapped and
measured on both renderers. It chose a rule element over `rich_text`'s underlined
spans because a span's underline takes its thickness and offset from the font,
unmeasured on either renderer (RFC-109 Q-2).

## Proposal

**R-1 — `design::button::link(tokens, label, on_press)`.** The ghost style
(`style::button::ghost`, so no `snora-style` change) **plus an underline in the
label's colour, present in every state:** rest, hover and pressed.
- **The underline follows the text, line by line,** when a label wraps (RFC-105:
  caller text wraps and never clips). How it is drawn is Unit 0's question (Q-5).
- **Contrast:** the underline is non-text, at ≥ 3:1 against the background; the label
  is text, at ≥ 4.5:1. Both are asserted on drawn pixels.

**R-2 — there is no disabled link** (revised in review). A dimmed, underlined phrase
still says "you can go here", and then does nothing. That is exactly the
misunderstanding this RFC exists to remove. So there is no `link_maybe`. When the
destination is unavailable, the application shows plain text, or nothing, and the
rustdoc says so. This is the one deliberate asymmetry with `ghost_maybe`, and it is
stated, not accidental.

**R-3 — one rule decides link or action** (revised in review), in the rustdoc of both
variants and in `design/buttons.md`:

> *A **link** takes the reader somewhere: another view, a document, a chooser. An
> **action** does something where the reader already is: Cancel, Skip, Save. Use
> `link` for the first and `ghost` for the second.*

orbok's "Choose another folder" opens a chooser, so it is a link under the rule. That
it needed thinking about is why the rule is written down.

**R-4 — `ghost` is unchanged.** Underlining it would make every tertiary action read
as a link.

**R-5 — declare the accent's text use.** `Palette::usages` gains a second accent
entry, `ThresholdClass::Text` on the neutral surfaces, for the ghost and link labels.
*(Found while writing this RFC.)* Today the accent is declared only as non-text,
although both labels draw it as text.

**R-6 — keyboard.** A link is a control, so RFC-116's R-1 makes it focusable and
operable with Enter, with snora's focus ring. Until that ships, its rustdoc states
that it is pointer-only, and points to RFC-116.

## Open questions

**Q-1 — a new variant, or an underline on `ghost`?** **Suggest a new variant** (R-4).

**Q-2 — the unstyled path.** **Suggest design path only**, like every other
`design::button` variant. An engine-only application builds its own buttons.

**Q-3 — links inside running text.** **Suggest: not this RFC.** A link in mid-sentence
is `rich_text` with a linked span, not a button. If Unit 0 measures `rich_text`'s
underline as sound, the accessibility guide recommends it there, with the same
figures.

**Q-4 — release.** **Suggest 0.54.0,** if Unit 0 settles Q-5 quickly. Otherwise the
release after. orbok adopts it on release, and said there was no hurry.

**Q-5 — how the underline is drawn: Unit 0 decides by measurement.**
- **RFC-109's 1 px rule element** is measured and snapped. But it is one rule under
  one box, so it cannot follow a wrapped label's lines. A two-line link would show one
  underline, under the second line only, or across the whole box. Either misleads.
- **`rich_text` underlined spans** follow every line. RFC-109 set them aside because
  their thickness and offset were the font's, *unmeasured*. Since RFC-108, pixels are
  measurable.
- **Suggest:** Unit 0 renders a one-line and a two-line link with `rich_text` on both
  renderers, and measures the underline's thickness, its gap from the glyphs and its
  contrast.
  - If it is at least 1 px everywhere, and clears 3:1, use `rich_text`: it is the
    robust one.
  - If not, use the rule element, and the label does not wrap. That would be the one
    exception to RFC-105, stated in the rustdoc.

  Unit 0 reports, and the owner rules.

## Review against snora's principles

- **Finally clean and robust (owner, 2026-11-04):**
  - the underline must survive wrapping, so the drawing method is chosen by
    measurement, not habit (Q-5);
  - one variant, with no disabled form to explain.
- **Nobody can misunderstand it (owner, 2026-11-04):**
  - no link that looks live and is not (R-2);
  - one written rule for link versus action (R-3);
  - `ghost` keeps meaning "action".
- **ABDD:** a non-colour cue for the one control whose meaning was carried by colour
  alone.
- **Keyboard (owner's ruling):** covered by RFC-116's R-1 (R-6).
- **Verify claims:** checked that `ghost` draws no underline, which led to R-5.
- **Correspondence cost:** orbok hears at the targeted release (the owner's ruling).

## What this is not

- Not a change to `ghost` or to any style function.
- Not inline links in running text (Q-3).

## Acceptance criteria

1. **Unit 0:** the measurements in Q-5, on both renderers, as their own review.
2. **Failing first:** a rendered test that finds an underline under a `link`'s label,
   at rest and on hover, and under **each line** of a wrapped label (or, if Q-5 rules
   the rule element, that the label does not wrap). It fails until `link` exists.
3. The underline's contrast (≥ 3:1) and the label's (≥ 4.5:1) on all four presets,
   read from drawn pixels.
4. R-5 **shown failing:** with the accent temporarily at 4.0:1 on `surface`, the suite
   fails on the new text entry.
5. `ghost`'s tests unchanged. CHANGELOG under Added (`link`) and Fixed (the accent's
   declaration).

---

## Rulings, 2026-11-04

All suggestions accepted, as revised in `387d29f`.

- **Q-1:** a new `link` variant; `ghost` is unchanged.
- **Q-2:** design path only.
- **Q-3:** inline links stay out of scope; guidance follows Unit 0's figures.
- **Q-4:** 0.54.0 if Unit 0 settles Q-5 quickly, otherwise the release after.
- **Q-5:** Unit 0 measures `rich_text`'s underline on both renderers, and the owner
  rules on it. There is no disabled link, and the link-or-action rule is written into
  the rustdoc and the book.

## Ruling, 2026-11-07 (Q-5): snora draws the underline, one per line

Unit 0 (`.git-exclude/reviewed/117-accent-text-and-underline/`) measured `rich_text`'s
underline:
- every line underlined;
- but a fixed hairline at every text size (3 rows on tiny-skia, 2 on wgpu);
- and a 0-row gap at 16 px, touching the descenders.

**The owner ruled for the architect's recommendation:**
- `link` is a widget built on RFC-116's `control`, which it needs anyway for the
  keyboard;
- it lays out its own paragraph and reads one rectangle per line through
  `Paragraph::span_bounds` (`iced_core/src/text/paragraph.rs:66`, shared by both
  renderers);
- **it draws an underline under each line, with a thickness and gap that follow the
  text.**

**The exact thickness and gap rule** is proposed from Unit 0b's measurement of a
prototype, and then ruled.

**Release:** with RFC-116 R-1 (0.55.0). There is still no disabled link, and R-3's
link-or-action rule stands.
