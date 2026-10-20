# RFC-109 — Ancestor breadcrumbs look exactly like the current page

**Status.** Done — shipped v0.53.0. Every open question ruled as suggested (see Rulings).
**Raised.** 2026-10-19, architect. **Owed:** RFC-103 Q-3 was ruled as suggested,
"register the leaf as `Position`, and file the link-affordance question
separately", and it was never filed. Found while listing open themes.
**Release target.** 0.53.0 (Q-4). **Priority.** Small.

---

## The finding

**At rest, an ancestor crumb and the leaf are drawn in the same colour.**
- The ancestor is a `button` whose resting arm in `crumb_button_style` (`crumb.rs`)
  sets **no background** and `text_color: palette.background.base.text`.
- The leaf is plain `text` that inherits the theme's base text colour. That is also
  `palette.background.base.text` (`iced_core` `theme.rs:334`).

So the colours are equal by construction, on both paths: the design variant passes
only geometry (`breadcrumb_geometry`) and shares the style function.

**What tells them apart today:** position (the leaf is last), the separators, and a
`background.weak` fill **on hover**. Each of those fails a reader somewhere:

- **Position** says which crumb is current, not which ones can be pressed.
- **Hover** needs a pointer already over the crumb, so it offers nothing to someone
  scanning the trail.
- **A keyboard user gets nothing at all:** iced 0.14 exposes no focus state to
  button styling, so a focused crumb looks like an unfocused one.

The state register records `BreadcrumbLeaf => Cue::Position` and has **no entry for
the ancestors' "pressable" state**, which is why nothing failed.

This is not a WCAG 1.4.1 failure in the strict sense, since no information is
carried by colour alone: there is no distinction at all. But it is the affordance
breadcrumbs exist to give, and every web and desktop convention gives it at rest.

## Proposal

**R-1 — ancestors carry a resting cue that is not colour alone.** The suggested cue
is an **underline**, a 1 px rule under the label in the label's own colour (Q-1). It
is a shape, so it survives greyscale, colour-blindness and high-contrast themes,
and it is the convention users already read as "link". Hover keeps the
`background.weak` fill. The leaf stays plain.

**R-2 — register it.** A new `State::BreadcrumbAncestor => Cue::Shape { asserted_by:
… }`, asserted by **rendered bounds**: the rule exists under every ancestor and under
no leaf, as the tab underline is asserted (RFC-102/103). Because the register is
exhaustive, the new state fails to compile without a cue.

**R-3 — the rule's geometry is snapped.** It is drawn at a whole-pixel offset with
a 1 px minimum, so it does not fall into RFC-108's fractional-coverage case. That
RFC measured the tab bar's 1 px rule smeared to ~50% by tiny-skia at a fractional
position.

## Open questions

**Q-1 — which cue?** **Suggest the underline.**
- A different colour for ancestors (`primary`) would be a colour-only cue, needing
  3:1 against the leaf plus a non-colour cue on focus, and iced 0.14 cannot style
  focus.
- Emboldening the leaf marks where you are, which `Position` already does, not
  what is pressable.

**Q-2 — how is the underline drawn?** **Suggest a rule element under the label,
inside the button**, like the tab underline. That keeps it inside the hit target, and
reuses the snapped, measured approach. `rich_text` underline spans are the
alternative, but their thickness and offset are the font's, not ours, and are
unmeasured on both renderers.

**Q-3 — opt-out?** **Suggest none.** It is a cue snora draws for a state snora
draws, which is RFC-103's rule. A consumer who wants no underline has a stronger
reason to say so than we have to guess; that would reopen it.

**Q-4 — release.** **Suggest 0.53.0.** It is an appearance change on the default
path: a migration-guide item, and a baseline change for orbok, which renders crumbs.

## What this is not

- Not a colour change; the label colour is untouched.
- Not a focus ring; that remains blocked on iced (RFC-060 Q-2).

## Acceptance criteria

1. The register's new entry, **shown failing** with the rule removed (no shape under
   an ancestor), then passing. It also fails if a rule appears under the leaf.
2. Measured on both paths and all four design presets: rule bounds, the rule's
   colour against the page (≥ 3.0:1, WCAG 1.4.11 for a UI cue), and the crumb row
   height before and after.
3. The pointer target is unchanged or larger, checked with `pointer_target`.
4. CHANGELOG under Changed, and the migration guide notes the appearance change.

---

## Rulings, 2026-10-19

**Q-1:** an underline. **Q-2:** a rule element under the label, inside the button,
snapped to whole pixels. **Q-3:** no opt-out. **Q-4:** 0.53.0.

**Rulings during review, 2026-10-19:**

- **"Snapped to whole pixels" is met by `container::Style::snap`, not by layout.**
  The rule's offset includes the label height, `default_text_size × 1.3`, which the
  host owns (RFC-104), so no padding can make it whole in advance. Snapping happens
  at draw time instead, where the physical grid is known. Measured drawn, the rule
  reaches its exact style colour in a fully covered row in all six contexts
  (11.05–21.00:1).
- **The rule keeps its resting colour on hover.** A container's style closure sees
  only the theme, so it cannot follow the button's status in iced 0.14. The handoff's
  "matches the label in both states" was not achievable. On hover the rule
  (`background.base.text`) is drawn over the hover fill (`background.weak.color`).
  The architect measured that pairing at **≥ 4.51:1 on all 26 contexts** (22 stock
  themes and 4 presets; lowest Kanagawa Lotus), and it is asserted as part of
  acceptance.

*(Corrected 2026-10-19, round 2: the first ruling above is wrong in its reason.
`snap` does **not** make the rule land on whole pixels in anything snora tests. In
iced 0.14, `container::Style::snap` reaches `renderer::Quad::snap`, which
`iced_wgpu` reads and `iced_tiny_skia` 0.14.0 ignores. The test harness is
tiny-skia, where the rule's drawn rows are byte-identical with and without the
flag. "A fully covered row in all six contexts" is the peak row that any 1 px rule
at a fractional offset has. On wgpu, measured in RFC-108's Unit 0, snapping does
remove the smear. The flag stays, for that reason.
*That sentence is itself wrong: see the 2026-10-20 correction in RFC-108. On wgpu,
snapping changed no drawn pixel.* Found by the dev team, who had
made the original claim and withdrew it.)*
