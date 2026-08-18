# Developer Handoff — RFC-066 sweep the dim assertion

**Governing RFC.** [RFC-066](../../proposed/066-the-dim-assertion-is-an-endpoint-check.md)
**Status.** Inherited from RFC-066 — Accepted (owner, 2026-08-18).
**Release target.** 0.38.0. **Test and documentation only** — no preset values,
no rendering.
**Implementation units.** One.

---

## 1. Task title

Replace the dim assertion's three-surface check with a sweep over the achievable
content range, and correct the two published figures it overstated.

## 2. Purpose

`dialog_card_distinguishable_from_modal_dim_all_presets`
(`crates/snora-design/src/tests.rs:173`) checks the card against the dim
composited over **three discrete surfaces**. The dim is painted over whatever
the application rendered — a **continuum** — and for two presets the worst case
is an interior minimum the three-surface check cannot see.

Nothing fails. The method is wrong, and it produced optimistic numbers.

## 3. The target figures

Sweeping greyscale content, worst case of `max(border ǀ dim, fill ǀ dim)`:

| preset | true worst | at content | position | currently recorded |
|---|---|---|---|---|
| `light` | **3.2424** | 1.000 | endpoint | 3.24 ✓ |
| `dark` | **3.1609** | 0.000 | endpoint | 3.18 |
| `high_contrast_light` | **4.5827** | 0.822 | **interior** | 7.37 |
| `high_contrast_dark` | **4.4515** | 0.051 | **interior** | 5.25 |

**Re-derive these; do not copy them.** If yours disagree, yours win and the
disagreement is the finding.

## 4. Q-3 is decided: 1000 steps, and here is why that number

Convergence, measured:

| steps | `dark` | `hc_dark` | `hc_light` | `light` | max Δ vs previous |
|---|---|---|---|---|---|
| 10 | 3.1609 | 4.9091 | 4.7939 | 3.2424 | — |
| 100 | 3.1609 | 4.4565 | 4.6012 | 3.2424 | 0.0766 |
| 1000 | 3.1609 | 4.4515 | 4.5827 | 3.2424 | 0.0051 |
| 10000 | 3.1609 | 4.4497 | 4.5827 | 3.2424 | 0.0018 |

**1000 lands within 0.002 of a 10 000-step answer** at negligible cost. Note
also that the two *binding* presets are endpoint minima and are exact at any
resolution — resolution only affects the reported margin on the two presets that
are nowhere near the bar. Record this reasoning; a bare `1000` invites someone to
change it on taste.

## 5. Q-2 is decided: sweep, not the analytic solution

The minimum of `max(f, g)` lies either at an endpoint or where `f == g`, and
that crossing is solvable — three evaluations instead of a thousand.

**Take the sweep anyway.** The analytic form is exact and is more code to get
subtly wrong, in a test whose entire purpose is to be trusted. A thousand float
operations per preset is free. **Record in a comment that the analytic form
exists and was declined for robustness, not overlooked** — otherwise this reads
as the naive choice.

## 6. Q-1 is decided: replace, do not keep both

The sweep **subsumes** the three-surface check: those three surfaces are three
specific points inside the swept range. Keeping both means maintaining a weaker
assertion alongside a stronger one — and the weaker one has just demonstrated
what it costs, by reporting 7.37 where the truth was 4.58.

**But keep the failure message locatable.** `content 0.822` is not something a
reader can act on. On failure, report:

- the worst content value and both channel contrasts at it;
- **and** the figures at the three named surfaces, as context.

A failure that says only "the minimum is 2.9 at content 0.63" tells a maintainer
nothing about which part of their palette to look at.

## 7. Why greyscale is the whole sweep — record this

`why not sweep all of RGB?` is the obvious next question and the answer is not
obvious, so it must be a comment rather than tribal knowledge.

The dim composites channelwise in sRGB: `dim_over_i = α·base_i + (1−α)·content_i`.
Contrast depends only on relative luminance, which is monotonic in each channel.
So for any content in the RGB cube, `L(dim_over)` lies between its values at
`content = black` and `content = white` — and greyscale sweeps that interval
continuously.

A 3D sweep adds ~10⁶ points and **no coverage**. If someone later "improves"
this to a colour cube, they have spent CI time for nothing.

## 8. Correct the published figures

RFC-065's numbers appear in more than one place. **Grep for them** rather than
working from this list:

```bash
grep -rn "7\.37\|5\.25" docs/ CHANGELOG.md rfcs/done/065-*.md
```

`high_contrast_light` 7.37 → 4.58 and `high_contrast_dark` 5.25 → 4.45. Where a
figure sits in shipped history (CHANGELOG entries for a release that already
went out), **do not rewrite it** — add a correction note in the current entry
instead. Released notes are a record of what we said, not a wiki.

## 9. Change scope

| File | Purpose |
|---|---|
| `crates/snora-design/src/tests.rs` | replace the assertion (§3–§7) |
| `docs/src/contributing/accessibility-checklist.md` | the sweep rule, if the composited-surface item needs it |
| whatever §8's grep finds | corrected figures |
| `CHANGELOG.md` | **Changed**, with the correction |

## 10. Explicit non-change scope

Do **not**:

- **Change any preset value.** Everything passes. If your re-derivation finds
  otherwise, **report before repairing** — RFC-036's carve-out and its
  failing-first order apply, and this is not the RFC to exercise them in.
- **Change `DIM_ALPHA`.** 0.44 was chosen against endpoint figures, and the
  sweep's minima (4.45, 4.58) sit far above 3:1. The value is not implicated.
- **Change the either-signal rule.** `max(border, fill)` stands — and it is
  what makes the interior minimum exist at all, so removing it would hide the
  thing this RFC is about.
- **Sweep RGB** (§7).
- **Rewrite shipped CHANGELOG entries** (§8).
- Modify `render_semantics.rs`.

## 11. Required tests

```bash
cargo test -p snora-design                   # the swept assertion
cargo test -p snora --lib --all-features
cargo test -p snora --test render_semantics  # MUST pass unmodified
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
```

**Also demonstrate the sweep catches what the old check missed.** Temporarily
perturb a palette value so the interior minimum drops below 3.0 while the three
named surfaces still pass, show the old-style check passing and the sweep
failing, then revert and prove the tree is byte-identical. That is the evidence
that this RFC did anything — without it, both assertions merely pass.

## 12. Required evidence

- The re-derived table for all four presets, with the content value at each
  minimum and whether it is interior or endpoint.
- The convergence check justifying the step count (§4).
- **The perturbation demo from §11**, with the revert proven.
- The §8 grep, and every corrected figure.
- `render_semantics` output and `git diff --stat -- crates/snora/tests/` empty.
- `git diff` showing no preset value changed.

## 13. Acceptance criteria

RFC-066 §Acceptance criteria 1–6. The two that carry the task:

- **2** — the sweep's worst case per preset captured and matching §3, or the
  discrepancy reported.
- **4** — the greyscale-sufficiency reasoning recorded *next to the sweep*, not
  in a commit message.

## 14. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/066-the-dim-assertion-is-an-endpoint-check/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the perturbation demo (§11). Everything else in this
change passes both before and after, so that demo is the only evidence the sweep
is stronger than what it replaced.
