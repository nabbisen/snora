# RFC-117 — A link does not look like a link

**Status.** Proposed.
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

**R-1 — `design::button::link` and `link_maybe`.** These have the same signatures as
`ghost` and `ghost_maybe`, and the same style function (`style::button::ghost`), so
no `snora-style` change is needed. One addition: **a 1 px rule under the label,
inside the button, in the label's colour**, drawn as RFC-109's crumb underline is.
- **Present in every state:** rest, hover and pressed. On hover the ghost fill
  appears behind it, as for the crumb.
- **Disabled** (`link_maybe(…, None)`): the rule dims with the label, by the same
  alpha.
- **Its contrast:** the rule is non-text, at ≥ 3:1 against the background it sits on.
  The label is accent text, already ≥ 4.5:1 on every preset. Assert both.

**R-2 — `ghost` is unchanged.** Ghost is a tertiary *action* ("Cancel", "Skip"), not
navigation. An underline there would make actions look like links (Q-1).

**R-3 — the keyboard route.** Like every snora control, a link is a button, and iced
0.14's buttons take no keyboard input (RFC-116). Its rustdoc says so, and points to
RFC-116's helpers and the application's own shortcuts. RFC-116's 1.0 gate covers it.

**R-5 — declare the accent's text use.** `Palette::usages` gains a second accent
entry, `ThresholdClass::Text` on the neutral surfaces, labelled for the ghost and link
labels. It costs one declaration, and the existing contrast suite then guards both
labels. *(Found while writing this RFC.)*

**R-4 — docs** (the architect's): `design/buttons.md` gains the link, says when to
use it rather than `ghost`, and states R-3.

## Open questions

**Q-1 — a new variant, or an underline on `ghost`?** orbok offered either. **Suggest
a new variant.**
- Underlining `ghost` would change every tertiary action in every design
  application, and make them read as links.
- A new variant changes nothing for anyone who does not use it, and names what it is.

**Q-2 — the unstyled path.** It has no ghost button either, and an engine-only
application builds its own buttons. **Suggest: design path only**, like every other
`design::button` variant.

**Q-3 — links inside running text.** A link in the middle of a sentence cannot be a
button: iced lays buttons out as blocks. That is `rich_text` with a linked span, which
iced draws with the font's underline. **Suggest: not this RFC.**
- The accessibility guide says that a link in running text should use `rich_text`
  with `underline(true)`.
- It also says that thickness and offset are the font's, unmeasured by snora.

**Q-4 — release.** **Suggest 0.54.0,** with Theme E. It is additive and small, and
orbok adopts it on release.

## Review against snora's principles

- **ABDD:** a non-colour cue for the one control whose meaning was carried by colour
  alone, without restyling existing actions.
- **Rules a reader can predict:** a link is underlined, and an action is not.
- **Verify claims:** checked that `ghost` draws no underline. Checking it also found
  the accent declared as non-text while drawn as text (R-5). The rule reuses an
  approach measured on both renderers.
- **Keyboard (owner's ruling, 2026-11-04):** the route is stated, and the gap is
  tracked under RFC-116.
- **Correspondence cost:** orbok hears at the targeted release (the owner's ruling),
  not before.

## What this is not

- Not a change to `ghost` or to any style function.
- Not inline links in running text (Q-3).

## Acceptance criteria

1. **Failing first:** a rendered test that finds a rule under a `link`'s label, at
   rest and on hover. It fails until `link` exists.
2. The rule's contrast (≥ 3:1) and the label's (≥ 4.5:1) on all four presets, read from
   drawn pixels (RFC-108's route).
3. `ghost`'s tests unchanged.
4. R-5 **shown failing:** with the accent temporarily set to 4.0:1 on `surface`, the
   suite must fail on the new text entry (it passes today on the non-text entry
   alone).
5. CHANGELOG under Added (`link`) and Fixed (the accent's declaration).
