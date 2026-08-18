# Developer Handoff — RFC-065 the modal dim

**Governing RFC.** [RFC-065](../../proposed/065-the-modal-dim-is-an-unmeasured-surface.md)
**Status.** Inherited from RFC-065 — Accepted (owner, 2026-08-18).
**Release target.** 0.37.0 (minor — `design`-path appearance change).
**Implementation units.** One.

---

## 1. Task title

Make the modal dim a measurable surface, assert the dialog card against it,
repair `light`, and record the surface axis beside RFC-063's role axis.

## 2. Purpose

RFC-063 closed the *role* axis. This is the *surface* axis, and today the
vocabulary cannot express it: `Palette::usages` names `Palette` surfaces, while
the dim is derived at render time in the `snora` crate from a constant beside
the renderer — invisible to `snora-design`, which is iced-free by hard
constraint and is where the contrast suite lives.

Measured, `light` fails: the dialog card is distinguishable from its own dimmed
backdrop at **2.85:1** by either available signal.

## 3. The either-signal rule — get this right first

Most assertions in the suite are one foreground against one background. **This
one is not.** Under SC 1.4.11 the card is identifiable if **either** its border
**or** its fill clears the bar:

```text
max(contrast(border, dim), contrast(surface_raised, dim)) >= NON_TEXT_MIN
```

Asserting both individually would fail three presets that are genuinely fine —
`dark` passes on fill alone (border 1.00), `high_contrast_light` passes on
border alone (fill 2.85). **Encode the `max`, and say why in a comment**, or the
next reader will "fix" it into two assertions and break a correct design.

## 4. Q-1 is decided: option (b), with one constraint

Move the derivation into `snora-design` as a pure function over `Tokens`, and
have the engine call it.

**Why not (a)** — asserting in `snora`'s own tests: it splits contrast
assertions across two crates. One suite is why the contrast rules have held; two
is how a rule gets applied unevenly.

**Why not (c)** — extending `RoleUsage` to carry computed surfaces: most general,
most invasive, and this RFC does not yet know whether a second derived surface
exists. Do not build the general mechanism for one instance.

**The constraint that makes (b) worth doing:** the new function must be the
**single source**. `design::render::dim_color` must *call* it, not reimplement
it. If the rule (white base in dark presets, black in light) or the alpha exists
in two places after this change, the RFC has added a function and kept the drift
it was meant to remove.

So:

- `DIM_ALPHA` moves with the rule. It currently lives at
  `crates/snora/src/design/render.rs:78` and is asserted at
  `crates/snora/src/design/render/tests.rs:102` — **that test must keep
  passing**, adapted to the new location, not deleted.
- `dim_color` becomes a thin adapter: call the `snora-design` function, convert
  `snora_design::Color` → `iced::Color`.
- `snora-design` already has `composite_over` (`contrast.rs:65`), which per
  tekstide's Q3 has **no in-repo caller today**. This is its first. Use it
  rather than open-coding the composite.

**One thing to restate, not drop.** `DIM_ALPHA`'s comment says it *"matches the
unstyled path's literal (`Color::from_rgba(0.0, 0.0, 0.0, 0.4)`)"*. If Q-2
changes the value, that symmetry breaks **deliberately** — rewrite the comment
to say the two paths have diverged and why, rather than leaving a claim that is
no longer true.

## 5. Q-2 is decided: 0.44, and report the margins

| `DIM_ALPHA` | `light` | `dark` | `hc_light` | `hc_dark` |
|---|---|---|---|---|
| 0.40 (today) | **2.85 FAIL** | 3.18 | 7.37 | 5.25 |
| 0.42 | 3.04 | 3.40 | 6.91 | 4.89 |
| **0.44** | **3.24** | **3.64** | 6.48 | 4.56 |

0.42 clears every preset and is the smallest change. **Take 0.44 anyway.**

RFC-058 set this precedent explicitly when choosing border values: clear the
binding pair *"with a margin… not hugging the boundary — chosen deliberately so
f32 rounding at the actual assertion site can't tip a passing preset into
failure."* 3.04 is a 1.3% margin. 3.24 is 8%.

**Re-derive the table yourself** rather than copying it, and report the measured
margin for all four presets. If your numbers disagree with mine, yours win and
that disagreement is the finding.

**Flag for review, do not decide alone:** a stronger dim also obscures more of
the content behind the modal. That is a UX consequence, not a contrast one, and
0.40 → 0.44 is a 10% relative increase. State it in the review request so the
owner sees the trade rather than only the ratio.

## 6. Failing-first, as with every carve-out repair

1. Add the assertion.
2. **Run it. Watch `light` fail at 2.85.** Capture the output.
3. Then change `DIM_ALPHA`.
4. Record measured before/after for all four presets.

Do not reorder. This is an appearance change justified by a measurement; the
measurement has to exist first or the justification is retroactive.

## 7. Change scope

| File | Purpose |
|---|---|
| `crates/snora-design/src/` | the dim derivation + `DIM_ALPHA` (§4) |
| `crates/snora-design/src/tests.rs` | the either-signal assertion (§3) |
| `crates/snora/src/design/render.rs` | `dim_color` becomes an adapter |
| `crates/snora/src/design/render/tests.rs` | adapt the existing alpha test, keep it |
| `docs/src/contributing/accessibility-checklist.md` | composited/derived surfaces as a class |
| `docs/src/contributing/api-governance.md` | the surface axis beside RFC-063's role axis |
| `docs/src/guides/migration-0.36-to-0.37.md` | **new** — appearance change |
| `CHANGELOG.md` | **Fixed**, with measured before/after |

## 8. Explicit non-change scope

Do **not**:

- **Touch the unstyled path.** It dims with a literal
  `Color::from_rgba(0.0, 0.0, 0.0, 0.4)` and draws no card, so the
  component-boundary clause does not attach the same way. Leave it, and do not
  "fix" both for symmetry.
- **Add a shadow to the dialog card.** RFC-039 removed it deliberately —
  shadows carry almost no information in high-contrast presets.
- **Add or rename a `Palette` role.** RFC-036 forbids it without reopening
  D-3/D-4; the dim is deliberately derived, not a token.
- **Change `border`'s or `surface_raised`'s values.** Their existing asserted
  pairs still pass; this is a fourth surface, not a regression in the first
  three. If you find yourself editing a preset, stop and report.
- **Change `NON_TEXT_MIN`.**
- **Build the general computed-surface mechanism** (Q-1 option c).
- Modify `render_semantics.rs`.

## 9. Required tests

```bash
cargo test -p snora-design                      # must fail at step 2, pass at step 4
cargo test -p snora --lib --all-features        # incl. the adapted dim_color test
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

`render_semantics` asserts composition, not colour, so it should be unaffected.
If it fails, something structural moved and that is a finding, not a fixup.

## 10. Required evidence

- The new derivation, and proof it is the **single** source — show
  `dim_color` calling it, and grep that neither the base-colour rule nor the
  alpha appears twice.
- **The failing run** (§6 step 2) and the passing run, both in full.
- Measured before/after for all four presets, both signals, with margins.
- The either-signal assertion, with its comment.
- The adapted `render/tests.rs` alpha test, still passing.
- The migration guide and the CHANGELOG **Fixed** entry.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/` empty.

## 11. Acceptance criteria

RFC-065 §Acceptance criteria 1–7. The two that carry the task:

- **2** — failing-first evidence at 2.85 on `light`, before any `DIM_ALPHA`
  change.
- **4** — Q-1's chosen shape recorded *with its reasoning*, and the
  single-source constraint demonstrated rather than asserted.

## 12. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/065-the-modal-dim-is-an-unmeasured-surface/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** whether the derivation is genuinely single-source
after the move, and the either-signal assertion's comment. The failure mode is a
future contributor reading `max(...)` as a mistake and splitting it — which
would fail three presets that are correct.
