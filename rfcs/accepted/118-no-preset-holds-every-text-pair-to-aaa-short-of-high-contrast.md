# RFC-118 — No preset holds every text pair to AAA, short of high contrast

**Status.** Proposed. **Revised 2026-11-04** after the architect reviewed it against
the owner's fundamental philosophy (see "Review against snora's principles").
**Raised.** 2026-11-04, architect, from orbok's request 4
(`.git-exclude/upstream/receive/0.54.0/orbok-snora-request-0.54.0-easy-to-read-2026-11-04.md`).
The owner approved opening it. Theme E (older users).
**Release target.** 0.55.0 (Q-4). **Priority.** An improvement requested by an adopter.
Every preset meets AA, which is snora's stated floor.

---

## The finding

orbok now holds every text pair it renders to WCAG AAA (7:1, criterion 1.4.6). It
asked for an accent reaching 7:1 in Light and Dark. **Measured, 2026-11-04**, with
`snora_design::contrast::contrast_ratio`, on each preset's background / surface /
surface_raised:

| Role | Light | Dark |
|---|---|---|
| `text_primary` | 17.81 / 16.44 / 17.81 | 16.89 / 15.55 / 14.04 |
| `text_secondary` | 8.80 / 8.12 / 8.80 | 9.17 / 8.44 / 7.62 |
| **`text_muted`** | **4.93 / 4.55 / 4.93** | **5.44 / 5.01 / 4.53** |
| **`accent`** (= `info`) | **6.70 / 6.19 / 6.70** | **6.78 / 6.24 / 5.63** |
| **`success`** | **5.02 / 4.63 / 5.02** | 10.78 / 9.92 / 8.96 |
| **`warning`** | **5.43 / 5.01 / 5.43** | 11.31 / 10.41 / 9.40 |
| **`danger`** | **6.54 / 6.03 / 6.54** | **6.77 / 6.23 / 5.62** |

Each fill's own text colour on that fill is also under 7:1 in Light (accent 6.70,
success 5.02, warning 5.43, danger 6.54, info 6.70). In Dark, accent, danger and info
are (6.82, 6.71, 6.82).

**orbok's figures reproduce exactly** (accent 6.70 Light, 6.78 Dark). **But the accent
is not the whole gap.** `text_muted` sits at 4.53–5.44 in both presets, and four
roles miss 7:1 in Light. Raising only the accent would leave orbok's own audit listing
the rest.

**The high-contrast presets already meet 7:1 on every pair above:** the lowest is
8.38 (`success`, high-contrast light). But they are built for maximum contrast, and
an application may want Light's or Dark's character at AAA, without the starkness.

**What the covenant allows.** New preset functions are additive. Changing a preset's
colour *values* is allowed, as 0.34's border repair did, under RFC-036's
accessibility carve-out. Changing a token's
*meaning*, or adding a `Palette` role, is forbidden without reopening the gate.

## Proposal

**R-1 — two AAA presets: `Tokens::light_aaa()` and `Tokens::dark_aaa()` (Q-1, Q-2).**
- Each keeps its parent's hues and surfaces as far as it can, and darkens or lightens
  the text roles until **every text pair `Palette::usages` declares is ≥ 7:1**.
- That covers text roles against each surface, and each fill's own text.
- The hierarchy between `text_primary`, `text_secondary` and `text_muted` is kept in
  order, though compressed: it can no longer drop below 7:1.

**R-2 — assert it from the declaration, not from a list.** *Depends on RFC-117's R-5:*
today `Palette::usages` declares the accent as non-text only, although the ghost and
link labels draw it as text. Without R-5, this sweep would skip the very pair orbok
asked about. One test iterates the AAA
presets and every `Palette::usages` text pair, and asserts ≥ 7:1. Non-text pairs keep
3:1. The composited surfaces (the modal dim and the dialog card) are covered, because
`all_presets()` already sweeps every preset.

**Two more postconditions, asserted** (added in review):
- **The text hierarchy keeps its order:** `text_primary` ≥ `text_secondary` ≥
  `text_muted`, measured against each surface. An AAA preset that made "muted" text
  stronger than "secondary" would invert what the roles mean.
- **Each role's hue stays within a stated distance of its parent preset's,** so
  `light_aaa` still reads as Light, and is not a different theme with Light's name.
  The bound is set when the palette is designed, and stated in the docs table.

**R-3 — Light and Dark are unchanged.** They remain AA presets, which is their stated
floor. The book says so, and points AAA applications to R-1's presets or the
high-contrast pair (Q-3).

## Open questions

**Q-1 — new presets, or raise Light and Dark?**
- **Raising Light and Dark** changes every design application's appearance. It also
  compresses the text hierarchy for everyone, which `text_muted` exists to provide.
- **New presets** change nothing for anyone who does not choose them.

**Suggest new presets.** That fits orbok's own wording, "or your guidance if you'd
rather keep the presets as they are (for example, an AAA variant)".

**Q-2 — scope: the accent only, or every text pair?** orbok asked for the accent, but
they hold *every* text pair to AAA. **Suggest every declared text pair**, or the
preset would fail the first audit that uses it.

**Q-3 — or guidance only?** The cheapest answer is "use the high-contrast presets",
which meet 7:1 today. **Suggest offering both:** the AAA presets for Light's and
Dark's character, and the high-contrast pair for maximum contrast. The book names
both.

**Q-4 — release.** **Suggest 0.55.0.** Two new palettes need design work and a
review of their own, and orbok said there was no hurry. 0.54.0 already carries
RFC-112 and Theme E's other four RFCs.

**Q-5 — names, and telling the presets apart.** Two families that both mean "more
contrast" would confuse anyone choosing between them. So each name says what it
guarantees, and the docs give the axis in one table:

| Preset | Text | Non-text | For |
|---|---|---|---|
| `light`, `dark` | AA, 4.5:1 | 3:1 (borders 3.12–3.81) | general use |
| `light_aaa`, `dark_aaa` | **AAA, 7:1** | 3:1, as their parents | Light's or Dark's character at AAA text, e.g. older readers |
| `high_contrast_light`, `high_contrast_dark` | AAA and more (≥ 8.38) | borders at 21:1 | maximum contrast |

**Suggest `light_aaa` / `dark_aaa`.**
- **Rejected in review: a two-axis constructor,** such as `Tokens::new(Scheme,
  Contrast)`. It is tidier in the abstract, but it would supersede, and so deprecate,
  the four frozen preset functions every design application calls. That is churn on
  the covenant surface for no gain to a reader of the table above.

**Q-6 — a transform instead of presets?** For example `tokens.with_aaa_text()`,
raising any token set, custom ones included, to 7:1. **Rejected in review, because
it cannot keep its promise.** Against a mid-tone fill, the better of black and white
can be as low as √(1.05 / 0.05) ≈ 4.58:1 (RFC-111's bound). So 7:1 is unreachable
without changing the fill itself, and that changes what the fill means. A transform
would have to fail at run time or quietly break its guarantee. Hand-designed presets
meet it by construction, and a test proves it.

**Q-7 — the hierarchy is quieter at AAA.** With `text_muted` raised to 7:1, it sits
close to `text_secondary`, about 8.8:1 in Light. Colour then carries less of the
hierarchy. **Suggest:** say so in the docs. An AAA application should let size and
weight (`Typography`) carry hierarchy, rather than colour alone. That is better for
older readers anyway.

## Review against snora's principles

- **ABDD:** defaults stay AA, which is the stated floor. AAA becomes a choice that
  costs one call, and is asserted as completely as the defaults are.
- **Rules a reader can predict:** "an `_aaa` preset holds every declared text pair to
  7:1", one sentence backed by one test over the declaration.
- **Verify consumer claims:** orbok's figures reproduce. Measuring the whole palette
  showed that the gap is wider than their ask, and Q-2 follows from that.
- **Correspondence cost:** orbok hears at the targeted release.

## Review against snora's principles, second pass (owner's philosophy, 2026-11-04)

- **Finally clean and robust:**
  - presets that meet their guarantee by construction, rather than a transform that
    cannot (Q-6);
  - no churn on frozen preset functions (Q-5).
- **Nobody can misunderstand it:**
  - names that state the guarantee, and one table telling the three families apart
    (Q-5);
  - the hierarchy's order asserted, so "muted" never outranks "secondary" (R-2);
  - each preset recognisably its parent.
- **Safe:** the guarantee comes from the declaration, so a role added later is swept
  automatically.

## What this is not

- Not a change to Light, Dark or the high-contrast presets.
- Not a new `Palette` role.

## Acceptance criteria

1. R-2's test, **failing first** with `light_aaa` / `dark_aaa` temporarily returning
   Light / Dark, listing the pairs in the table above.
2. Every existing preset test passing for the new presets: surfaces, focus, targets.
3. A side-by-side table, in the design docs, of each AAA role against its parent.
4. CHANGELOG under Added.
