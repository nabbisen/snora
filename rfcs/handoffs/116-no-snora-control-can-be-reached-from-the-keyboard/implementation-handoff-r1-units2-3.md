# Implementation handoff — RFC-116 R-1, Units 2 and 3: `snora-control`, and every control on it

**RFCs.**
- `rfcs/accepted/116-…`: its rulings of 2026-11-07, R-1 and the new crate;
- `rfcs/accepted/115-…`: its amendment of 2026-11-05, the sizing rules, which live in
  `control`.

**Release target.** 0.55.0.
**Order.**
- After R-1 Unit 1 is accepted. Each unit below is its own review.
- On `rfc116-unit0`, rebased onto `main` once 0.54.0 is cut. Merged into `main` only
  after Unit 3's review.

## Unit 2 — the crate, and nothing else moves

- **`crates/snora-control`:**
  - holds `control`, moved out of `snora-widgets`, and `snora::keyboard::traverse`
    stays where it is;
  - depends on `iced_core` directly (Unit 0's finding), and on `iced` under RFC-112's
    rule: drawing features fixed, everything else through its own default-on
    `iced-default`;
  - **`snora` and `snora-widgets` depend on it, non-optionally.**
- **Crate doc:** it exists to serve snora's own crates, is versioned in lockstep with
  them, and applications should use `snora`. Its public items are only what the two
  dependents need.
- **Workspace hygiene:**
  - the workspace dependency entry;
  - `[package.metadata.docs.rs]`;
  - `scripts/` that list crates (version-sync, publish order, check-workflows);
  - `release.yaml` publishes it, since `cargo publish --workspace` orders it, but
    check it;
  - **RFC-112's opt-out probe stays clean,** with no `zbus` or `mundy`;
  - the feature matrix gains its rows.
- **Proof that nothing moved:** the sidebar's tests, Unit 0's and Unit 1's, pass
  unmodified. Engine-only and widgets size probes are reported against `main`.

## Unit 3 — every snora control on `control`, with RFC-115's sizing inside it

**The rollout.** Each of these replaces `button` with `control`:
- tabs and breadcrumb crumbs;
- the toast's close button;
- the notice's action and dismiss buttons;
- chip filter and chip remove;
- menu items, and the header menu trigger;
- every `design::button` variant.

No public signature changes.

**Sizing, in `control`'s layout** (RFC-115's amendment):
- **target ≥ max(24 px, 1.5 × text, the minimum target size)**, where text is the
  renderer's `default_size()` on the unstyled path, or the size of the text the
  control carries;
- **the minimum target size** is a new token group, `Tokens::targets`, with `min: f32`,
  `#[non_exhaustive]`, at **24 in all six presets**. Design-path wrappers pass it to
  `control`; the unstyled path has no token;
- **icons:**
  - a pictogram is 1 em of the text it sits with;
  - the × close mark is 1.125 em;
  - the `Tooltip` rail's icons are 1 em of the label size.

  This ends `icon_element`'s literal 14 (`icon.rs:8`).

**Keyboard.** Every control passes Unit 0's probe:
- Tab reaches it, through Unit 1's end-to-end route, or its named halves;
- Enter and Space press it;
- the ring draws at ≥ 3:1.

A disabled control is not a focus target (Unit 1).

**Tab in a real window (from Unit 1's review).** `iced_winit`'s `Action::Widget` arm
requests no redraw, so a Tab-driven focus change might not show the ring until the
next event. Wire `traverse` into one example, and check **by hand, in a real window**
that the ring appears on the Tab press itself. Record what you saw. If it is late,
record focus beside status and compare on the next event.

**Document that Tab at the last control leaves the group, with no wrap.** That is
iced's `focus_next`, and correct for a window with other controls.

**Re-run RFC-115's Unit 0 inventory** (its probe is in
`.git-exclude/review-request/115-size-inventory/evidence/`). **Only these rows may
change:**
- at 24 px text: toast close, chip filter, chip remove, which are the three k = 1.5
  rows;
- unstyled chrome icons, 14 → 16 at the default text size.

Anything else that moves is a finding: stop and report it.

**Docs (rustdoc):** every "pointer and touch only" sentence from `efd83b1` and RFC-114
is rewritten to say what is now true. The architect rewrites the book, and withdraws
gate 11's "open" when Unit 3 is accepted.

## Breakage rows (each unit)

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| a control's target ≥ 1.5 × text | drop the `1.5 ×` term: the 24 px-text rows fail |
| the minimum target token is honoured | set a preset's `min` to 44, then drop the term: a 30 px control must fail |
| a pictogram is 1 em | restore `icon_element`'s literal 14: the 24 px-text icon rows fail |
| × is 1.125 em | use 1 em for ×: the toast close glyph row fails |
| each control is focusable | drop `Focusable` from one control's wrapper: only its row fails |

## Acceptance criteria

1. Units 2 and 3 each reviewed separately, with every breakage shown failing.
2. The inventory re-run differs from Unit 0's only in the predicted rows.
3. All twelve quality-gate commands locally, and the branch CI 19 of 19 jobs.
4. CHANGELOG, `[Unreleased]`:
   - Added: `snora-control`, `Tokens::targets`, keyboard access;
   - Changed: icon sizes;
   - Fixed: the WCAG 2.1.1 gap.

   The architect writes the migration guide.
