# RFC 030 — Documentation, Examples, and Design Workbench

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `docs/`, `examples/design_*`, workbench.

## Summary

This RFC defines documentation and example requirements for v0.20.

## Goals

- Explain minimal/default/custom usage.
- Explain boundary and non-goals.
- Document tokens and style bridge.
- Provide compileable examples.
- Provide live design workbench for visual-fit QA.

## Non-goals

- No screenshot framework in v0.20.
- No polished marketing site requirement.
- No final recipe catalog in v0.20.

## Documentation structure

Recommended:

```text
docs/src/design/
  overview.md
  philosophy.md
  feature-flags.md
  tokens.md
  palettes.md
  high-contrast.md
  typography.md
  iced-style-bridge.md
  buttons.md
  cards.md
  semantic-accessibility.md
  accessibility-checklist.md
  migration.md
```

## Required examples

```text
examples/minimal_layout
examples/design_default
examples/design_tokens_only
examples/design_custom_tokens
examples/design_light_dark
examples/design_high_contrast
examples/design_workbench
```

## Design workbench

`examples/design_workbench` should include:

- light/dark/high-contrast toggle;
- LTR/RTL toggle if practical;
- button states;
- card variants;
- readable text samples;
- focus-state test area;
- line-height stress samples;
- high-contrast border samples.

## Visual fit checklist

Workbench must inspect:

- focus affordance / documented absence of custom focus ring (iced 0.14);
- focus ring clipping;
- high-contrast borders;
- text clipping;
- button vertical alignment;
- card padding;
- disabled state readability.

## Data lifecycle

```text
v0.20 foundation docs
  -> v0.21 primitive docs
  -> v0.22 recipe docs
  -> v0.23 stable subset docs
```

## Process handling

No public feature should merge without:

- API docs;
- guide docs if user-facing;
- example or workbench coverage;
- migration note if relevant.

## Internal design

Workbench should not become a second framework.

Possible layout:

```text
examples/design_workbench/
  main.rs
  sections/
    buttons.rs
    cards.rs
    typography.rs
    palettes.rs
```

Split only if needed.

## Acceptance criteria

- Docs explain philosophy amendment.
- Docs show minimal path.
- Docs show default design path.
- Workbench exists.
- Examples compile.
- Visual-fit checklist exists.
