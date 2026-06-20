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
- **Focus-ring vocabulary.** `FocusTokens` exist for future iced versions and
  custom widgets that expose focus state; the limitation in the current version
  is documented.

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
[ ] Any new palette pair added for this primitive is listed in the
    contrast-tests module in snora-design and passes `cargo test -p snora-design`.
[ ] If the primitive uses an alpha/translucent color, it is composited over
    the tested background before the contrast ratio is computed.
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
    it meets the >= 3:1 contrast ratio requirement against adjacent colors
    and uses the focus token (tokens.focus.ring_color, ring_width,
    ring_offset).
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

```text
[ ] Interactive controls have a minimum tap/click area of 24×24 logical
    pixels. 44×44 is the preferred minimum for finger-sized targets.
[ ] Spacing tokens (tokens.spacing.sm or larger) are used for padding
    rather than zero or near-zero values that would collapse the target.
```

### Typography and line-height

```text
[ ] The primitive uses text roles from the token typography scale rather
    than magic pixel values.
[ ] Line-height multipliers (stored in TextRole.line_height) are used where
    the rendering path supports them. In iced 0.14, line-height is
    vocabulary-only; the limitation is documented.
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
