# RFC 066 — The dim assertion checks three surfaces; the dim composites over a continuum

**Status.** Proposed
**Tracks.** Accessibility / measurement integrity. Continues RFC-065.
**Reported by** **tekstide** (2026-08-18), as a question about our method rather
than a finding about our code — and it was right.
**Touches.** `crates/snora-design/src/tests.rs`,
`docs/src/reference/` or `docs/src/contributing/accessibility-checklist.md`,
`CHANGELOG.md`. **No preset value changes expected.**
**Release target.** 0.38.0.

## Summary

RFC-065 asserts the dialog card against the modal dim composited over **three
discrete surfaces** — `background`, `surface`, `surface_raised`. But the dim is
painted over whatever the application rendered, which is a **continuum**, and
the worst case is not always at an endpoint.

Swept, two of four presets have an **interior minimum** that the three-surface
check does not see. Nothing fails today. But the recorded margins for both
high-contrast presets were overstated by 40–60%, and a future palette edit could
pass the assertion while the true minimum sits below 3:1.

## The finding, verified

tekstide's argument, from their own codebase where the same shape appears:

> **Endpoints are not a sweep.** Both our extremes pass; the failure is a
> minimum *between* them. No pair list, however exhaustively derived from the
> role set, finds that — because the role set does not enumerate the compositing
> parameter.

Swept over greyscale content in 2001 steps, taking the worst case of the best
identifying channel — `max(border ǀ dim, fill ǀ dim)`:

| preset | true worst | at content | position | RFC-065 recorded |
|---|---|---|---|---|
| `light` | **3.24** | 1.000 | endpoint | 3.24 ✓ |
| `dark` | **3.16** | 0.000 | endpoint | 3.18 ✓ |
| `high_contrast_light` | **4.58** | 0.822 | **interior** | 7.37 |
| `high_contrast_dark` | **4.45** | 0.051 | **interior** | 5.25 |

The interior minima are exactly the crossing tekstide described: the point where
the border and the fill contrast equally, so neither channel is carrying the
boundary.

**Everything still passes.** This is not a defect report; it is a defect in the
*method*, and the numbers it produced for two presets were wrong in the
optimistic direction.

## Why greyscale is a sufficient sweep

Worth recording, because "why not sweep all of RGB?" is the obvious next
question and the answer is not obvious.

The dim composites channelwise in sRGB: `dim_over_i = α·base_i + (1−α)·content_i`.
Contrast depends only on relative luminance, and luminance is monotonic in each
channel. So for content anywhere in the RGB cube, `L(dim_over)` is bounded by its
values at `content = black` and `content = white` — and greyscale content sweeps
that interval continuously.

**A greyscale sweep therefore spans the entire achievable luminance range of the
composited dim.** A 3D sweep would add 10⁶ points and no coverage.

## Scope

1. **Replace the three-surface check with a sweep** over the achievable content
   range, per preset, asserting the worst case of the best identifying channel.
2. **Correct the recorded figures** for `high_contrast_light` and
   `high_contrast_dark` wherever RFC-065's numbers were published.
3. **Record why greyscale suffices**, so the sweep is not later "improved" into
   a 3D one.

## Non-goals

- **No preset value change.** Everything passes; if the sweep finds otherwise on
  re-derivation, that is a finding to **report before repairing**, under
  RFC-036's carve-out and its failing-first order.
- **No change to `DIM_ALPHA`.** 0.44 was chosen in RFC-065 against the endpoint
  figures; the sweep's minima (4.45, 4.58) are comfortably above 3:1, so the
  value is not implicated.
- **No change to the either-signal rule.** `max(border, fill)` stands, and is in
  fact what makes the interior minimum exist at all.
- **No 3D colour sweep.**
- **No change to the unstyled path**, which draws no card.

## Open questions

**Q-1 — replace the three-surface check, or keep it alongside?** The sweep
subsumes it: the three surfaces are three specific points inside the swept
range. Keeping both means two assertions where one is strictly stronger.
Suggest **replace**, and report the three named-surface figures as context in
the failure message so a failure still says something a reader can locate.

**Q-2 — sweep, or solve the crossing analytically?** The minimum of
`max(f, g)` lies either at an endpoint or where `f == g`. That crossing is
solvable, giving an exact answer at three evaluations instead of N. A sweep is
simpler and its cost is trivial in a unit test; an analytic solution is exact
but is more code to get subtly wrong. **Suggest the sweep**, with a stated step
count and a comment recording that the analytic form exists and was declined for
robustness, not ignorance.

**Q-3 — what step count?** 2001 was used for the investigation. The function is
continuous with a single interior minimum per preset, so resolution only needs
to be fine enough that the sampled minimum is within rounding of the true one.
State the chosen count and why.

## Acceptance criteria

1. The dim assertion sweeps the content range rather than checking three
   surfaces, for all four presets.
2. The sweep's worst case per preset is captured and matches this RFC's table,
   or the discrepancy is reported.
3. Any published RFC-065 figure that the sweep contradicts is corrected —
   specifically `high_contrast_light` 7.37 → 4.58 and `high_contrast_dark`
   5.25 → 4.45.
4. The greyscale-sufficiency reasoning is recorded next to the sweep.
5. Q-1 answered; if both checks are kept, the reason is recorded.
6. `render_semantics` passes unmodified; no preset value changed.

## Compatibility and security

**Compatibility.** Test-only and documentation. No API, no rendering, no preset
values. Nothing a consumer can observe changes.

**Security.** None.

## Credit

tekstide, who raised it as a question about our method — *"we do not know your
rendering well enough to say which, so this is a question, not a finding"* —
while closing their own correspondence thread. It was the only item in that
letter that could affect our code, and it did.
