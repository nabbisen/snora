# Accessibility checklist

This checklist is required for any Snora Design primitive — button, card,
chip, notice, progress, or similar widget helper. Complete every section
before requesting a review. Leave an explicit `N/A + reason` where a section
genuinely does not apply.

The checklist covers what Snora Design can influence. It does not guarantee
that arbitrary application content built on top of these primitives is
accessible; see [§ What applications still own](#what-applications-still-own).

---

## What Snora Design can help with

- **Contrast-tested color palettes.** All built-in token palettes pass the
  mandatory WCAG AA contrast pairs (automated, see
  [§ Contrast](#contrast)).
- **ABDD layout discipline.** Logical edges (`Edge::Start`, `Edge::End`,
  `LayoutDirection`) ensure consistent directional behavior under LTR and RTL.
- **Semantic control use.** Policy requires native iced interactive widgets
  wherever available (see
  [`semantic-accessibility.md`](semantic-accessibility.md)).
- **High-contrast preset.** Two high-contrast token presets are provided for
  users who need stronger visual separation.
- **Focus-ring vocabulary.** `FocusTokens` exist for applications that already
  own focus as their own state today — not only for future iced versions and
  custom widgets. The constraint is narrower than "focus rings are
  unavailable in iced 0.14": iced cannot tell a *standard* button/container
  style closure that it is focused; an application's own closure that
  already knows its focus state is unaffected. See
  [`semantic-accessibility.md`](semantic-accessibility.md).

The allowed claim is:

> Snora Design provides accessibility-oriented defaults and ABDD layout
> discipline.

---

## What applications still own

Snora Design does not and cannot guarantee:

- that arbitrary application content is accessible;
- full screen-reader semantics (beyond what iced exposes);
- OS accessibility-setting synchronization (reduced motion, system contrast
  mode) — this is not implemented in v0.20;
- complete keyboard navigation of custom application flows;
- translation, locale formatting, and bidirectional text shaping.

---

## Checklist sections

### Contrast

```text
[ ] Foreground / background pairs for this primitive are drawn from palette
    roles that have verified mandatory contrast (>= 4.5:1 for body text).
[ ] Any non-text boundary this primitive relies on to identify a component —
    a border, a focus ring, or any other visual boundary that is not purely
    decorative (WCAG 2.1 SC 1.4.11) — meets >= 3.0:1 against every adjacent
    surface it can appear on. This is a rule about the class of non-text
    boundaries, not one role: RFC-058 found `border` untested and failing
    (1.19-1.43:1 in light/dark) precisely because this rule had only ever
    been written against `focus`.
[ ] Any contrast figure published outside this repository — CHANGELOG,
    migration guide, release notes, correspondence — is the **worst case
    across the surfaces the role renders on**, and *says so*. Quoting the
    worst case is correct for a conformance claim; quoting it under a label
    naming one surface is not. A downstream team (orbok, 2026-08-18) read a
    disagreement into our 0.34.0 figures because a table header said "vs
    `surface`" while the `dark` row quoted `surface_raised` — both numbers
    right, the label wrong. They asked for the note explicitly: it tells a
    reader which pairing to reproduce before concluding anything.
[ ] If this primitive introduces a **new** `Palette` role, its intended
    surfaces and threshold class are declared in `Palette::usages`
    (`crates/snora-design/src/palette.rs`, RFC-063) — the compiler
    enforces this: a role added without a declaration fails to compile
    there (`E0027`), not silently ships untested. `mandatory_pairs` is
    derived from the declaration; there is no separate pair list to
    edit. If this primitive reuses an **existing** role on a surface it
    is not yet declared to render on, add that surface to the role's
    declaration and confirm `cargo test -p snora-design` passes.
[ ] If the primitive uses an alpha/translucent color, it is composited over
    the tested background before the contrast ratio is computed.
    **This rule is correct and stays armed, but untested by anything
    snora ships (RFC-061, tekstide Q3): every colour across all four
    built-in presets is `Color::rgb(...)`, not one `rgba` — verified by
    `grep -rn "rgba" crates/snora-design/src/presets/` returning
    nothing. The compositing path applies to applications introducing
    their own translucent tokens, not to any preset role today.** If
    this primitive is the first to introduce one, `composite_over` is
    correct but has no prior in-repo exercise to point to as proof — it
    is your primitive that first tests it for real.
[ ] Disabled states are noted as exempt from mandatory contrast
    (WCAG 1.4.3 exception) but are still legible.
```

### High contrast

```text
[ ] The primitive renders acceptably with the high_contrast_light and
    high_contrast_dark token presets. Visually verified (manual or workbench).
[ ] Borders and separators are visible at high contrast (the high-contrast
    palette uses full-black / full-white borders).
[ ] No element is invisible or illegible under high-contrast tokens.
```

### Focus visibility

```text
[ ] The focus-ring limitation for this iced version is documented if
    the primitive uses a standard iced button or container.
    (iced 0.14: button::Status has no Focused variant; container has no
    interaction status. See semantic-accessibility.md for the full
    statement.)
[ ] If a focus ring IS expressible (custom widget or future iced version),
    it meets the non-text boundary contrast requirement (see § Contrast)
    against adjacent colors and uses the focus token
    (tokens.focus.ring_color, ring_width, ring_offset).
[ ] The absence of a custom focus ring is documented as a known limitation,
    not left undiscovered.
```

### Keyboard reachability

```text
[ ] The primitive uses a native iced interactive widget (iced::widget::button,
    etc.) that iced makes keyboard-reachable by default where possible.
[ ] If the primitive is built from a non-interactive container plus a mouse
    handler, the limitation is documented and justified.
[ ] Basic activation via Enter / Space is inherited from iced for
    button-like primitives. No additional wiring is needed unless the
    primitive adds custom activation behavior.
```

### Semantic construction

```text
[ ] See semantic-accessibility.md for the full rule table.
[ ] The primitive RFC/PR answers the five semantic construction questions:
    (1) What native iced primitive is used?
    (2) Is it keyboard reachable?
    (3) How is focus visible?
    (4) What semantic limitation remains?
    (5) What example demonstrates usage?
```

### Pointer target size

**The two axes are enforced differently, and a reader should not assume
otherwise (RFC-061).** A control's target height is `line_box +
2 × vertical_padding`, and both terms are token values — the same
property that makes the contrast suite possible without a renderer, so
the **height axis is mechanically asserted**:
`pointer_target_height_meets_24px_for_every_role_and_padding_step` in
`crates/snora-design/src/tests.rs` checks every `TextRole` × `Spacing`
step combination (36 combinations, all four presets), not only the ones
a prefab control actually uses — enumerating "the ones a control uses"
would re-derive a hand-maintained list of call sites, the same failure
mode RFC-058/059/060 each hit once this cycle.

A control's target **width** is `content_advance +
2 × horizontal_padding`, and `content_advance` depends on the rendered
string, the font, and the shaping engine — snora cannot compute it
without a renderer, and `render_semantics` asserts composition, not
pixel geometry. **The width axis is review-only, not asserted, and this
is a limitation to work around at review time, not a solved problem.**
A primitive with a short, narrow label (an icon-only or single-glyph
button, in particular) needs its width checked by hand or by a rendered
probe — see the checklist item below.

```text
[ ] Height clears 24 logical pixels (mandatory) — verified by
    `cargo test -p snora-design`, not by inspection; if the assertion is
    green, this item is satisfied.
[ ] Width clears 24 logical pixels (mandatory, but NOT mechanically
    checked) — measure by hand or with a rendered/font-metrics probe for
    any primitive whose visible label is short (a single glyph or icon
    is the case most likely to fail; RFC-061 found `chip`'s dismiss "×"
    button at 15.0px wide against the shipped fallback font before it
    was fixed). Do not assume padding alone clears the floor — it does
    not always: `spacing.sm` padding around that same "×" glyph reaches
    only 23.0px, still short.
[ ] 44×44 is the *preferred*, not mandatory, minimum — not every
    role/padding combination meets it. See
    `docs/src/contributing/accessibility-checklist.md`'s companion
    reference for which combinations do (25 of 36 in the current token
    set); do not assume 44 is met just because 24 is.
[ ] Spacing tokens (tokens.spacing.sm or larger) are used for padding
    rather than zero or near-zero values that would collapse the target.
```

**The thinnest height margin in the current token set: `label` role at
`xs` padding clears 24 by 0.8 logical pixels** (24.8px) — the tightest
combination in the 36-entry matrix. A future reduction to either
`Typography::default_roles().label` or `Spacing::comfortable().xs`
could push this below the mandatory floor; the height assertion would
catch it, but the margin is recorded here too, the same way RFC-058
recorded `dark`'s 4.526:1 contrast margin — a fact the next token edit
should meet knowingly, not discover by a red CI run.

**44×44 preferred-bar status, per role/padding combination** (36 total;
✓ = meets 44px, current token set):

| Role \ Step | xs | sm | md | lg | xl | xxl |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| `body` | | | ✓ | ✓ | ✓ | ✓ |
| `body_small` | | | | ✓ | ✓ | ✓ |
| `label` | | | | ✓ | ✓ | ✓ |
| `title` | | | ✓ | ✓ | ✓ | ✓ |
| `heading` | | ✓ | ✓ | ✓ | ✓ | ✓ |
| `display` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

25 of 36 combinations meet 44px; all 36 meet the mandatory 24px floor.

### Typography and line-height

```text
[ ] The primitive uses text roles from the token typography scale rather
    than magic pixel values.
[ ] Line-height multipliers (stored in TextRole.line_height) are available
    today via iced::widget::text::LineHeight::Relative — see
    docs/src/design/typography.md. Snora's own prefab widgets do not yet
    apply it to the text they render internally; wiring it in (and/or
    adding *_line_height() style helpers) is deferred, not blocked.
[ ] Text in notices, labels, and help content uses at least body or
    body_small roles, not a custom size smaller than 12 logical pixels.
```

### Directionality (RTL/LTR)

```text
[ ] Any new direction-sensitive placement uses Edge::Start / Edge::End /
    LayoutDirection rather than hardcoded Left / Right.
[ ] The ABDD checklist (abdd-checklist.md) is completed for all
    direction-sensitive aspects of this primitive.
```

### Reduced motion

```text
[ ] Any animation or transition in this primitive (hover fade, toast slide,
    progress pulse) is noted as not yet gated on an OS reduced-motion
    preference in v0.20.
[ ] The limitation is recorded in this checklist if relevant.
    (OS reduced-motion synchronization is not part of v0.20 scope.)
```

### Disabled state readability

```text
[ ] The disabled visual state is distinct from the active state (typically:
    reduced alpha on background and text).
[ ] The disabled text color, while not required to meet body-contrast ratios
    (WCAG 1.4.3 exception), remains visually identifiable as a control.
```

### Loading, empty, and error states

```text
[ ] If the primitive has a loading, empty, or error variant, the text or
    icon in that state is informative ("Loading…", "No items", "Error: …"),
    not invisible or icon-only.
[ ] Error messages use plain language (see below).
```

### Plain-language wording

```text
[ ] Labels, tooltip text, and status messages avoid jargon where possible.
[ ] Destructive actions (delete, revoke, reset) use plain wording in the
    danger button label and, where present, in a confirmation dialog.
[ ] All UI strings in examples are in plain English (or clearly marked as
    placeholders for localization).
```

### Known limitations

```text
[ ] Any limitation not covered by the above sections is recorded here
    with a severity label:
    - INFO: noted but does not block the primitive.
    - DEFERRED: tracked for a future RFC.
    - BLOCKED: cannot be addressed until an external dependency (iced API,
      OS API) changes.
```

---

## Relationship to other checklists

- **ABDD checklist** (`abdd-checklist.md`) — required for all
  direction-sensitive changes; this checklist defers to it for RTL details.
- **Semantic accessibility** (`semantic-accessibility.md`) — defines the
  primitive construction rules that back the Semantic construction section
  above.
- **Visual-QA checklist** — run the design workbench
  (`cargo run -p snora-example-design-workbench`) and inspect against the
  visual-fit items listed in `docs/src/design/v021-primitives.md` and each
  primitive's RFC.
