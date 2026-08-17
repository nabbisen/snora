# Developer Handoff — RFC-058 border contrast

**Governing RFC.** [RFC-058](../../proposed/058-border-contrast-is-untested-and-failing.md)
**Status.** Inherited from RFC-058 — Accepted (owner, 2026-08-17).
**Release target.** 0.34.0 (minor — preset values change; `design`-path
appearance changes).
**Implementation units.** One. Independent of RFC-059, which ships alongside.

---

## 1. Task title

Assert `border` and `text_muted` contrast in `mandatory_pairs`, watch the new
assertions fail on `light`/`dark`, repair those two presets, and generalise the
checklist's 3:1 rule.

## 2. Purpose

`crates/snora-design/src/tests.rs` asserts twelve contrast pairs. `p.border` and
`p.text_muted` appear in **zero** of them. The untested role is failing:
`border` measures 1.19–1.43:1 against surfaces in `light` and `dark`, against
SC 1.4.11's 3:1 for non-text boundaries.

Reported by tekstide; every figure verified independently.

## 3. The ordering is the whole integrity of this task

RFC-036 forbids changing a preset value, with one carve-out: *"only where a
contrast test proves the change fixes an accessibility defect, recorded as
**Fixed** in the CHANGELOG."*

This is that carve-out's **first exercise**, so the order is not a preference:

1. **Add the assertions.** Commit-able on their own.
2. **Run them. Watch `light` and `dark` fail.** Capture the output.
3. **Then** change the border values until they pass.
4. Record measured before/after in the CHANGELOG under **Fixed**.

Adding the assertion after the palette edit satisfies the letter and destroys
the proof the carve-out requires. **Do not reorder these.**

No gate reopening is needed — this is the permitted path, not a forbidden
change. D-3/D-4 stay closed.

## 4. The numbers, computed — do not re-derive or eyeball

Current, WCAG 2.1 relative luminance:

| preset | border/background | border/surface | border/surface_raised |
|---|---|---|---|
| `light` | 1.39:1 | **1.28:1** | 1.39:1 |
| `dark` | 1.43:1 | 1.32:1 | **1.19:1** |
| `high_contrast_light` | 21.00 | 21.00 | 21.00 |
| `high_contrast_dark` | 21.00 | 21.00 | 19.80 |

**Which pair binds, and the target:**

| preset | binding pair | requirement |
|---|---|---|
| `light` | **`surface`** (L = 0.9192) | border relative luminance **≤ 0.2731** |
| `dark` | **`surface_raised`** (L = 0.0173) | border relative luminance **≥ 0.1518** |

In `light`, `surface_raised == background == pure white`, so optimising against
`background` passes the easy pair and fails `surface`. **Solve for the binding
pair.**

`text_muted` already passes — 4.83:1 light, 5.44:1 dark. Its assertion is a
ratchet, not a repair.

## 5. This is a large visual change, not a nudge

Current border luminance is ≈ **0.707** in `light`. The target is **≤ 0.273**.
That is a very light grey becoming a mid grey — borders will read as
substantially more present, not subtly so. `dark` moves ≈ 0.030 → ≥ 0.152, also
substantial.

**Do not soften it to reduce the visual delta.** 3:1 is the bar; a value that
"looks closer to today" and measures 2.6:1 reproduces the defect with a test
that now passes for the wrong reason.

Do state it honestly as an appearance change (§7).

## 6. The trap — the obvious alternative is the expensive one

Someone will propose adding a second role: keep `border` decorative, add
`border_strong` at 3:1 for identifying boundaries. tekstide's decorative-exempt
caveat invites exactly that.

**RFC-036 forbids it.** Its forbidden list includes *"Adding, removing,
renaming, or retyping a `Palette` role"* — which requires **reopening D-3 and
D-4 in the same change**, explicitly, with no rationalising afterward.

So the asymmetry is the opposite of intuition:

| Approach | Cost |
|---|---|
| Change `border`'s value | **permitted** by the accessibility carve-out |
| Add a `border_strong` role | **forbidden**; reopens two 1.0 gates |

**Do not add a role in this task.** If you conclude one is genuinely needed,
stop and escalate — it is a separate RFC with a gate cost, not a detail.

## 7. Change scope

| File | Purpose |
|---|---|
| `crates/snora-design/src/tests.rs` | the new assertions (step 1) |
| `crates/snora-design/src/presets/light.rs`, `dark.rs` | the repair (step 3) |
| `docs/src/contributing/accessibility-checklist.md` | generalise the 3:1 rule |
| `docs/src/guides/migration-0.33-to-0.34.md` (new) | the appearance change |
| `CHANGELOG.md` | **Fixed**, with measured before/after |

The migration guide is required: preset borders change, so every `design`-path
consumer's rendering changes. State it as a **defect repair**, and that the
direction is one-way — borders become more visible, nothing becomes harder to
see.

**Do not touch the high-contrast presets.** They pass at 19.8–21:1.

## 8. Also required

**Generalise the 3:1 rule.** `accessibility-checklist.md`'s *Contrast* section
mandates only `>= 4.5:1 for body text`; the 3:1 non-text rule sits only under
*Focus visibility*, attached to one role. Move it into *Contrast* as a rule
about non-text boundaries **as a class**. A rule stated once against a single
usage is why this went missing.

**Answer Q-3: sweep the other roles.** `Palette` has 18 roles; twelve pairs are
asserted. Establish whether a third role carries an untested obligation, and
report either the finding or "none". Cheap while the file is open, and tekstide
found two by looking.

**Q-2: consider `NON_TEXT_MIN = 3.0`** rather than reusing `FOCUS_MIN`. They
are the same number today; coupling border contrast to a constant named for
focus will read wrongly the first time either needs to move. Your call — state
which you chose and why.

## 9. Explicit non-change scope

Do **not**:

- **Reorder §3.** Assertion first, failure captured, then repair.
- **Add or rename a `Palette` role** (§6).
- **Change `AA_TEXT` or `FOCUS_MIN`'s values.**
- **Change the high-contrast presets.**
- **Add a pointer-target-size assertion.** Separate question (tekstide Q4).
- **Change any non-`border` preset value** unless Q-3 finds a second defect —
  and if it does, report before repairing.
- Modify `render_semantics.rs`.

## 10. Required tests

```bash
cargo test -p snora-design                      # the suite that must fail, then pass
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics     # MUST pass unmodified
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

`render_semantics` asserts composition, not colour, so it should be unaffected.
If it fails, something structural changed and that is a finding.

## 11. Acceptance criteria

RFC-058 §Acceptance criteria 1–7. The two that carry the task:

- **2** — **failing-first evidence**: the new assertions failing on
  `light`/`dark` *before* any palette edit, output captured. Without this the
  carve-out's condition is unproven and the value change is unauthorised.
- **6** — Q-3's sweep answered, not skipped.

## 12. Required evidence

- The new assertions.
- **The failing run** (step 2) and the passing run (after step 3), both in full.
- Measured before/after ratios for all six affected pairs.
- The two preset diffs, with the chosen values' computed luminance.
- Q-3's sweep result; your Q-2 decision.
- The migration guide and the CHANGELOG **Fixed** entry.
- `render_semantics` output plus `git diff --stat -- crates/snora/tests/`
  showing it empty.

## 13. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/058-border-contrast-is-untested-and-failing/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the chosen border values, and whether they pass the
**binding** pair rather than the easy one (§4). A value that passes
`border/background` and fails `border/surface` would look like a fix and be one
only on the pair nobody was worried about.
