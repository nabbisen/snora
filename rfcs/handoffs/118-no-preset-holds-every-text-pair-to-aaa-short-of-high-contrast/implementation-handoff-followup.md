# Implementation handoff — RFC-118 follow-up: one preset list, and `light_aaa`'s focus

**RFC.** `rfcs/accepted/118-no-preset-holds-every-text-pair-to-aaa-short-of-high-contrast.md`.
Read its "Rulings, 2026-11-07".
**Priority: first.** The 0.54.0 cut waits for this. It is small.

## 1. `Tokens::presets()`

- **A public function returning every named preset, in a fixed order:**
  1. `light`, `dark`;
  2. `light_aaa`, `dark_aaa`;
  3. `high_contrast_light`, `high_contrast_dark`.

  Each comes with its name as a `&'static str`. The return type is yours. Choose one
  an application could use for a theme picker, and say why.
- **Its rustdoc:** the order, what each family guarantees (RFC-118's Q-5 table, in
  one line each), and that the list is complete.
- **Every hand-kept preset list in the workspace iterates it instead.** That is nine
  places, including the six `snora/tests/*.rs` files still at four presets:
  `drawn_contrast`, `breadcrumb_ancestor_rule`, `sidebar_tooltip_body`, `side_bar_fit`,
  `dismiss_remove_controls`, `tab_bar_edges`. Grep for `high_contrast_dark()` and
  `Tokens::light()` to find any others, and list them.
- **A test that the list is complete:**
  - it names each preset constructor exhaustively;
  - so adding a preset function without adding it to `presets()` fails.

  Choose the mechanism, and show it failing.

**If any of the six `snora` test files fails** once it sweeps `light_aaa` and
`dark_aaa`, **stop and report.** That would be a real finding about the new presets,
not something to tune here.

## 2. `light_aaa`'s `focus` equals its accent

Light's `focus` is its `accent`, and `light_aaa`'s should match its own new accent.
- Assert `light_aaa.palette.focus == light_aaa.palette.accent`.
- Assert focus's 3:1 floor still holds.

`dark_aaa` is unchanged: Dark's focus was never its accent.

## Acceptance criteria

1. Both items done. The completeness test and the focus assertion are each shown
   failing.
2. All twelve of the quality gate's commands, run locally before the push.
3. CHANGELOG, under the existing RFC-118 entry in `[Unreleased]`: `Tokens::presets()`,
   and the focus change.
