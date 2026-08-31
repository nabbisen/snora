# Developer Handoff — RFC-085 widget-layer contrast

**Governing RFC.** [RFC-085](../../accepted/085-the-widget-layer-styles-are-unreachable-by-every-contrast-suite.md)
**Status.** Accepted (owner, 2026-09-01). **Critical.**
**Release target.** 0.42.0 — **minor.** Rendered appearance changes.
**Implementation units.** Two: **the suite first, then the fixes.**

---

## 1. Build the suite before fixing anything

**This is the one instruction that matters.** Three colour pairings are wrong;
the reason they are wrong and undetected is that **no suite in this project can
see the widget layer.** Fix the colours first and you have three patches and the
same blind spot.

**The suite must fail on all three findings before any of them is fixed.** That
is the acceptance bar, and it is the only proof the suite reaches where the
previous six contrast RFCs did not.

## 2. Q-1 ruled — the suite lives in `snora-widgets`

`snora-design` is iced-free by CI gate and cannot construct an
`iced::widget::button::Style`. `snora-widgets` already depends on iced.

**Use `snora-design`'s `contrast_ratio` rather than reimplementing it** — one
home for the maths, measured where the pairing happens.

## 3. Q-2 ruled — the subject is every style this crate produces

Not roles against surfaces; that is the token suite and it already passes.

**Every `button::Style` (and container style) this crate produces, in every
`Status`, measured against the background it is actually painted over.**

**Derive the set, do not list it.** RFC-063's precedent: a hand-written list of
style functions is the defect this project has removed four times this cycle. If
the set cannot be derived, say so and propose how — do not quietly hand-list.

**The background matters and is the subtle part.** `menu_button_style` sets
`background: None`, so it paints over the dropdown's surface, not over nothing.
Getting the "against what" wrong makes the whole suite meaningless.

## 4. Q-3 ruled — both paths, and record the cost

Measure **stock `iced::Theme`** as well as the `design` path. Most consumers
start on stock, and the architect measured stock failing.

**Record the cost in the suite's own doc comment:** this makes our assertions
depend on iced's palette derivations, which can change under us on an iced
upgrade. That is a real maintenance liability and a reader should meet it in
place, not discover it during a version bump.

## 5. Unit 2 — the three fixes

Only after the suite fails on them.

- **F-13** `menu_button_style` uses `primary.weak.color` — a **background
  tier** — as `text_color`. Architect's measurement: **1.89:1** light,
  **2.20:1** dark at rest; 3.73/3.70 hovered. `background.base.text` gives
  21.00/11.00.
- **F-14** `sidebar_button_style` pairs `background.base.text` with a
  `primary.weak.color` background when active. Fails **both** ways: 1.89:1
  against the rail on stock (and it is the *only* active cue — WCAG 1.4.1), and
  2.01:1 icon-on-highlight on `design`, **1.51:1 under `high_contrast_dark`**.
- **F-15** chrome borders below the non-text floor everywhere.

**Re-measure every figure yourself.** These are the architect's numbers, taken
from the audit and spot-checked; the project rule is that a figure you did not
derive is one you cannot defend.

**Q-4 ruled: `high_contrast_dark` at 1.51:1 is a blocker on its own.** The
preset that exists for low-vision users must not be the worst one.

## 6. Explicit non-change scope

- **No token value changes.** `snora-design` is correct and heavily asserted;
  `git diff -- crates/snora-design/src/presets` must be empty.
- **No new `Palette` role.** RFC-036 freezes that surface and the right colours
  already exist — every fix here is a re-pairing, not an addition.
- **Do not extend `Palette::usages`.** It declares *role* usage; render-time
  pairings are not roles, and forcing them in would corrupt a mechanism that
  works.
- **Do not touch `snora-design`'s suites.**

## 7. Required evidence

- **The suite failing on all three, before any fix** — this is the deliverable
- Every figure re-derived: both paths, all four presets, before and after
- The derivation of the style set (§3), with its method
- `git diff --stat -- crates/snora-design` — **expected empty**
- `render_semantics` unaffected

## 8. Acceptance criteria

1. A widget-layer contrast suite exists in `snora-widgets`, derived not listed,
   and **fails on F-13, F-14 and F-15 before they are fixed.**
2. All three fixed; every figure stated as your own measurement.
3. `high_contrast_dark` no longer the worst preset anywhere.
4. Q-3's iced-palette dependency recorded in the suite's doc comment.
5. No token value, no new role, `snora-design` untouched.
6. Migration guide states the appearance change and that reference images are
   invalidated, per 0.34.0's precedent.

## 9. Required review-request format

`.git-exclude/review-request/085-the-widget-layer-styles-are-unreachable-by-every-contrast-suite/`,
`README.md` entry point, evidence under `evidence/`.

**Requested review focus: the suite's reach.** Show it failing on all three
first. Three colour fixes are worth little; a suite that would have caught them
in 2026-03 is worth the release.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
