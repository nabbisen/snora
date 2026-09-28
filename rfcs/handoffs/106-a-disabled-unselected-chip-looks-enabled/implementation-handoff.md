# Implementation handoff — RFC-106: a disabled unselected chip looks enabled

**RFC.** `rfcs/accepted/106-a-disabled-unselected-chip-looks-enabled.md`
**Release target.** 0.52.0, **before the cut**. Nothing else is in flight.
**Rulings.** Q-1: alpha 0.45. Q-2: the selected chip dims its label and border
too. Q-3: 0.52.0.
**Touch list.** `crates/snora-widgets/src/design/chip.rs`,
`crates/snora-widgets/src/state_register.rs`, `CHANGELOG.md`. **Not**
`snora-style`: `disabled_alpha` there is private, and the crate is frozen. **Not**
`docs/`.

## Already established

- **Today:** both disabled arms set only the fill's alpha to `0.5`
  (`chip.rs` ~l.196 selected `accent`, ~l.218 unselected `surface`), and leave
  the label and border unchanged. Unselected enabled vs disabled measures
  **1.00–1.05:1**.
- **Measured, label dimmed at 0.45** over the half-alpha disabled fill, across
  light / dark / hc_light / hc_dark: label enabled vs disabled **3.18 / 3.18 …
  6.03** (dark is the low end), disabled label vs its fill 2.20–3.74, border
  enabled vs disabled 2.09–6.27. The disabled label under 4.5 is permitted:
  WCAG 1.4.3 exempts inactive components.
- The register has `ChipSelectedDisabled => Luminance { min: 2.6 }` and
  `ChipUnselectedDisabled => Missing { .. }`, plus a pinned test that fails when
  the gap closes, and `states_without_a_cue`.

## Unit 1 — failing-first

In `state_register.rs`: `ChipUnselectedDisabled` becomes a `Luminance` cue on
the **label** channel (RFC-103's per-channel naming for controls whose fill
cannot carry the cue), asserted at **≥ 3.0**. **Today it fails** (the label
does not change, so 1.00:1). Record it.

## Unit 2 — the fix

- `const CHIP_DISABLED_ALPHA: f32 = 0.45;` in `chip.rs`, with a comment naming
  `snora_style::button::disabled_alpha` as the value it matches and why it is not
  shared (private, in a frozen crate).
- **Both** disabled arms dim `text_color` and the border colour by it.
- **The fill's own alpha stays at 0.5.** It is the existing cue for the selected
  chip, and changing it is not ruled. If measurement shows a reason to change it,
  say so rather than changing it.

## Unit 3 — the register

- Replace the `Missing` entry. Show the pinned test failing on the fix, as
  RFC-103 designed it to, then remove it in the same change.
- **Update `states_without_a_cue`** to expect no `Missing` state. Keep the test:
  it is what forces the next gap to be recorded.
- **Re-measure `ChipSelectedDisabled`** now that its label also dims. Its fill
  floor (2.6) is unchanged. If the label now also clears a floor, you may register
  the stronger of the two, naming the channel. Report both numbers either way.

## Check each assertion can fail

- the new label assertion, with the dim removed (it returns to 1.00);
- the selected chip's re-measured floor, with its dim removed.

Scratch discipline: bash script files that copy named backups and end with
`sha256sum -c`. **No `git checkout` / `git restore`.**

## Appearance, for the migration guide

A disabled chip, in either state, now shows a dimmed label and border. Report
the before and after colours for one preset.

## CHANGELOG

Under **Fixed**: a disabled unselected chip looked enabled (1.00–1.05:1); its
label and border are now dimmed at 0.45, the factor disabled buttons use, giving
≥ 3.0:1 against enabled in every preset. The selected chip's disabled treatment
matches. Found by RFC-103's state register.

## Acceptance criteria

1. The new label cue is asserted at ≥ 3.0 and **shown failing** today.
2. Both disabled arms dim label and border by `CHIP_DISABLED_ALPHA`; the fill is
   unchanged.
3. The `Missing` entry and its pin are replaced (the pin shown failing first);
   `states_without_a_cue` expects none.
4. The selected-disabled floor is re-measured and reported.
5. The covenant diff is quoted and empty.
