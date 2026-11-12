# Handoff — 0.62: what the audit found, fixed and guarded

**Ruled.** The architect, at the audit's review (`.git-exclude/reviewed/audit-0-62-a/review-result.md`),
under the owner's "what the audit finds decides 0.62" (2026-11-12).
**The audit's tables are in this folder:** [`claims-a.md`](claims-a.md) and [`claims-b.md`](claims-b.md).
Row IDs below are theirs.
**Release target.** 0.62.0. **Three units, each its own review, in this order.** Land each as `main`
requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

**For every new test:** check first that it can fail, with a breakage (named `cp` backups,
`sha256sum -c`; never `git checkout` or `restore`), or say why none exists, as RFC-127 Unit 1
did. **`render_semantics.rs` is unmodified** (gate 5). Normative pointer tests go in
`covered_pointer.rs` or a new `overlay_laws.rs`.

## Unit 1 — the two fixes, and the pins for corrected sentences

**F-1. The sheet's width** (B-1, O-036, O-039). `crates/snora/src/overlay/sheet.rs`, `render_horizontal`:
when the sheet is on the right, bind `portions()`'s `(spacer, sheet)` the right way round.
- **A new `sheet_geometry.rs`** reads the sheet panel's bounds, as `menu_overlay.rs::bounds_of` does:
  - four edges × LTR and RTL;
  - `OneThird`, `Half`, `TwoThirds`, `Ratio(…)` and `Pixels(…)`, at two window sizes;
  - the defaults from `Sheet::new` (O-035 to O-042).
- **Unit tests in `sheet.rs`:** the surface style's corners, for the four edges × two directions
  (O-038).
- **O-049:** the engine-path sheet has its card (fill and border), and the engine-path dialog has
  none.
- **Breakage:** the unswapped binding, which is 0.61's code.
- **CHANGELOG `[Unreleased]`, Fixed:** a sheet on the right (`End` under LTR, `Start` under RTL) had
  the spacer's width since 0.4.0. `OneThird` drew two thirds and `TwoThirds` one third. Say
  plainly what an application that compensated must change.

**F-2. F6 after a click** (A-2, R-029, A-032). `crates/snora-interaction/src/zones.rs`: with nothing
focused, count from the zone of **Tab's starting point**, which a click sets (RFC-121 R-2). That is
the mark `control.rs` offers to `sequential_focus`; read the same mark.
- **Tests in `f6_zones.rs`:**
  - click a sidebar control, then F6: the body;
  - then Shift+F6: the header;
  - a press elsewhere that clears the starting point, then F6: the header (rule 3);
  - a click inside a dialog, then F6: nothing moves (rule 5).
- **Breakage:** ignore the mark.
- **CHANGELOG, Fixed:** F6 after a click counted from no zone.

**F-3. Pins for corrected sentences:**
- **The dim's strength as a pixel** (A-1, L-002, L-009): an empty page pixel under a dialog's dim,
  and under a dim with no close message, on `snora::render` and `snora::design::render`: 255 → 143
  (44%). Break it with the constant.
- **`design::control` as a `design::button` variant** (B-3, O-027):
  - find the padding with which `design::control(&t, text(label)).style(… secondary …)` draws the
    same frames as `design::button::secondary(&t, label, …)`, idle, hovered, pressed and focused;
  - assert them equal for every variant;
  - **report the padding,** so the book can state it.

## Unit 2 — holes in what was thought guarded

- **H-1. The 24 px floor alone** (R-087): `every_control.rs` at host text 10 and 12, where
  `1.5 × text` is under 24, on both paths. The audit's breakage `MIN_TARGET` 24 → 10 failed
  nothing. It must fail this.
- **H-2. `Status::Disabled`** (R-099): a recording `.style` sees `Disabled` for a control without
  `on_press`, on both builders.
- **H-3. A dialog opened over a laid-out dropdown** (M-049): PROBE 15's scene, beside
  `a_modal_covers_the_open_menu_too`, **which is not edited.**
- **H-4. The L-7 table** (A-090): its doc comment says it records a stated limit and asserts
  nothing.
- **H-5. A notice's action label at 4.5:1** (N-005): `notice_accent_color` against its surface, for
  every tone × every preset. **If any pair fails, stop and report.** It is a defect, and the
  architect raises it.
- **H-6. A gate: no iced `button` *widget* in the snora crates' non-test source** (A-012), in
  `ci.yaml`'s Rust quality job. snora's controls are keyboard-reachable only if none is an iced
  button. Show it failing on a planted use.
  - **What it must allow:** the `button` *module's* types (`Status`, `Style`, `Catalog`), which
    `Control` reuses on purpose (`control.rs:54`, `state_register.rs:65`), doc comments, and
    `#[cfg(test)]` code (`toast.rs:1177` is in a test module).
  - **What it must refuse:** a call that builds the widget (`button(…)`, `Button::new`).
  - A grep that cannot tell these apart is not the gate. Say how yours does. On today's `main`,
    the architect found none of the refused form.
- **H-7. A gate: snora installs no subscription** (S-017). No `Subscription`,
  `keyboard::listen` or `event::listen` in `crates/*/src` outside `toast::subscription`. Show it
  failing on a planted use.

## Unit 3 — the normative claims that are unguarded or guarded in part

**The overlay laws, the keyboard map, and the public keyboard and control rustdoc.** Each ID is a
row in `claims-a.md` or `claims-b.md`, with the scene the audit proposed. Its probe is on
`audit-0-62-probes` where one exists.

| Suite | Rows |
|---|---|
| `modal_keyboard.rs` | L-063 / L-069 / K-005 / A-096 (Tab with a dialog and a toast: the gap, then the first control); L-039 (a character and an IME commit at a covered `text_input`); R-129; R-033 (the cover tests through `traverse`); A-077 (focus returns on closing a sheet and a menu with its backdrop); A-075 |
| `covered_pointer.rs` or `overlay_laws.rs` | L-066 (`header_menu` and `context_menu` together); L-013 (a sinkless sheet blocks pointer, wheel and Tab); L-024 / O-009 (a touch on a dim and on a backdrop closes once) |
| `shortcuts.rs` | K-011 / R-059 (Ctrl+Tab wraps both ways); K-003; K-024 / R-073; L-067's row 4; R-005; R-045; R-048; R-067; R-071; R-076; R-133; R-049 / R-050 |
| `pointer_and_keys.rs` | K-033 / R-014 / A-042 (a focused snora control leaves the other keys alone; the engine binds neither Tab nor F6); R-043 / A-017; R-086 |
| `f6_zones.rs` | R-012; R-019 (a zone whose only control is disabled is skipped); R-025; R-021 (after the rustdoc correction) |
| `public_control.rs` | R-095 / R-096 (the ring's width and gap in a frame, both paths); R-118 (a loud `.style` keeps the ring at 3:1); R-094; R-120; R-089; R-088 |
| `control_id.rs` | R-113; R-106 |

**The rustdoc corrections are yours, in this unit:**
- A-3: `snora::control`'s builder list gains `id`, and "Nothing else is public" goes;
- A-4: how `design::style::button::*` are used with `.style` (a closure that supplies the tokens);
- A-7: `focus_zone`'s rule 3 does not list a dialog's control;
- B-5: `Sheet`'s rustdoc, which says it has no card on the engine path.

**Not in 0.62:** every other "in part" or "unguarded, true" row of the guides. It is the backlog,
in the two `claims.md`.

## Acceptance criteria, each unit

1. Every row it names has a test, shown able to fail, or a stated reason why none exists.
2. `render_semantics.rs` is unmodified.
3. All of `ci.yaml`'s Rust quality gate commands (17), the Docs job's commands, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.
