# RFC-108 — The default renderer is never tested, and contrast is never read from pixels

**Status.** Accepted 2026-10-19. Every open question ruled as suggested.
**Raised.** 2026-10-19, architect. Source: REQ-005 (tekstide: *"We ship where the
GPU path may not start, so a figure from one renderer does not transfer"*), Theme C.
**Release target.** Q-5. **Priority.** First of three opened together (RFC-108–110).
It is the largest, and Unit 0's measurement decides its shape.

---

## The finding

**CI renders only tiny-skia; users mostly get wgpu.** `.cargo/config.toml` sets
`ICED_TEST_BACKEND = "tiny-skia"` for every test (RFC-099). That was right: several
wgpu simulators at once crashed the test binary in the Vulkan loader. But wgpu is
iced's default renderer, so the rendered suite has never run on what most users see.
RFC-102's tab-bar fill, drawn by tiny-skia and not by wgpu, showed the gap is real.

**Contrast is asserted on style functions, never on drawn pixels.** That holds only
while the renderer draws the specified colour at full coverage. Measured on the
shipped tab bar at 0.52.0 (scratch probe, both renderers, same build, one machine):

| Element | wgpu | tiny-skia |
|---|---|---|
| bottom rule (1 px, `background.base.text`), light | 2 physical rows, solid black | 3 rows: 128 / 0 / 127 |
| the same, dark | 2 rows, 229 | 3 rows: 136 / 230 / 137 |
| active underline (2 px), both themes | 4 solid rows | 3 solid rows and 2 partial ones |

Over the frame, the light theme has 4,213 differing pixels (max channel delta 127),
all in rows 69–75. Dark has 7,105, of which 2,892 differ by ≤ 8 (text antialiasing).
**The mechanism:** the bar's height is fractional (37.8 px at the default text
size), so its rule sits off the pixel grid. wgpu snaps it; tiny-skia antialiases it
across the boundary.

**At scale 2, which is all the simulator renders, each element keeps a solid core
row, so its asserted contrast survives.** At scale 1, the common desktop default, a
1 px element at a fractional position has **no** solid row under tiny-skia. Its peak
coverage is below 100%, so a colour asserted at, say, 3.0:1 is drawn below it.
`iced_test`'s `Simulator::snapshot` hard-codes `scale_factor = 2.0`
(`simulator.rs:221`), so no test has ever rendered at scale 1.

*(Corrected 2026-10-19, from Unit 0's Q-3 measurement: the scale-1 prediction above
did not hold for the tab rule. Rendered at scale 1 through `iced_test::screenshot`
on tiny-skia, it is **one solid black row** (y = 37), not a partial one. The
prediction was the architect's, made without a measurement. Peak-coverage
assertions at scale 2 remain, as R-2 specifies.)*

**One earlier statement was wrong: pixels *are* readable.**
`Snapshot::matches_image(path)` writes the frame as a lossless RGBA PNG when the
file does not exist, named `<stem>-<renderer>.png` from `renderer.name()`. Verified:
a probe wrote `frame-tiny-skia.png` and `frame-wgpu.png`, and a filled box decoded to
exactly the colour specified, `(51, 102, 153)`, on both. Three places say otherwise:

- `crates/snora/tests/common/mod.rs` (`frame_hash`'s comment): "the only way to
  read rendered pixels";
- RFC-105's finding: "frame hashes, not pixel reads";
- REQ-003's 0.52.0 status: "the harness gives frame hashes, not pixels".

**And the file name proves which renderer drew.** That matters: the headless
renderer **falls back to tiny-skia silently** when wgpu cannot start. A "wgpu" CI job
on a runner with no usable adapter would pass, rendering tiny-skia throughout.

## Proposal

**Unit 0 — feasibility, measured before anything is built.** On a branch,
`ubuntu-latest` runs the rendered suite with `ICED_TEST_BACKEND=wgpu`,
`--test-threads=1` (RFC-099's crash was in parallel), and mesa's software Vulkan
(lavapipe), or GL (llvmpipe) if Vulkan fails. Record:

1. whether a snapshot's file name says `wgpu`;
2. wall time, against today's tiny-skia run;
3. any crash or flake over 5 consecutive runs;
4. which tests, if any, fail only on wgpu.

The result is a written number, not a judgement. Q-1 rules what each outcome means.

**R-1 — a pixel-read helper.** `common::frame_pixels(ui, theme, name) -> (renderer, w,
h, rgba)`, built on `matches_image` the way `frame_hash` is built on `matches_hash`,
with `png` as a dev-dependency. Its first use **asserts the renderer**: it fails if
the name is not the backend the run asked for, so a silent fallback becomes a test
failure. Correct the three statements above when it lands.

**R-2 — contrast read from pixels, for the elements whose contrast is asserted.**
For each style-function contrast assertion that has a rendered surface (tab rule and
underline, sidebar active fill, chip states, toast fills), take the **drawn** colour:
the most common colour in the element's region, not one sampled pixel. Assert it
against the same floor. Where an element is thinner than 2 px, also assert its
**peak** coverage, so the scale-1 case is covered (Q-3).

**R-3 — both renderers in CI**, if Unit 0 shows it can be done: the rendered suite,
including R-2, runs once per backend. The wgpu job is serialised and asserts its
renderer through R-1.

**R-4 — a parity check, deliberately loose.** The two renderers will not match pixel
for pixel; antialiasing differs by design. Per element, assert that both drew the
**same core colour** and the same rounded bounds. Edge rows are allowed to differ.
That is what RFC-102's defect would have failed: one renderer drew a fill the other
did not.

## Open questions

**Q-1 — if wgpu cannot run in CI.** **Suggest:** ship R-1, R-2 and R-4 against
tiny-skia anyway, which is the fallback and the harder case for thin elements. Run the
wgpu suite locally at each release cut, serialised, as a release-checklist step with
its output kept. Record REQ-005 as "partly, by procedure", not "met".

**Q-2 — which software driver.** **Suggest:** whichever Unit 0 shows produces `wgpu`
in the file name with no flake. Prefer Vulkan (lavapipe), the path most Linux users'
wgpu takes.

**Q-3 — scale 1.** The simulator cannot render it. `iced_test::screenshot(program,
theme, viewport, scale_factor, duration)` takes a scale factor but needs a
`Program`. **Suggest:** Unit 0 also measures whether a minimal `Program` wrapping
one surface renders through it. If it does, R-2 runs at scale 1 as well. If not,
R-2's peak-coverage assertion at scale 2 is the bound, and the limit is written down.

**Q-4 — when a pixel floor fails where the style floor passes.** **Suggest:** treat
it as a defect, fixed by geometry. Snap the element to whole pixels, or give it a
2 px minimum. Never relax the floor.

**Q-5 — release.** **Suggest:** Unit 0, R-1 and R-2 in 0.53.0, and R-3 and R-4 in
the release after Unit 0 rules, so 0.53.0 is not held for a CI experiment.

## What this is not

- **Not a change to any colour or covenant.** Measurement and tests only, unless
  Q-4 finds a defect; that would get its own ruling.
- **Not a pixel-diff gate.** No golden images; R-4 compares core colours and bounds.

## Acceptance criteria

1. Unit 0's four numbers, written, for each driver tried.
2. R-1 fails when its renderer is not the requested one, shown by requesting `wgpu`
   where it cannot start. The three wrong statements are corrected.
3. Each R-2 assertion is **shown failing**, by drawing the element 1 px thinner or
   at a colour below the floor, and restored from named backups with `sha256sum -c`.
4. If R-3 lands: the wgpu job's log shows the renderer assertion passing, not just
   tests passing.
5. R-4 is shown failing on a reintroduced RFC-102 fill (one renderer only).

---

## Rulings, 2026-10-19

**Q-1:** if wgpu cannot run in CI, ship R-1, R-2 and R-4 against tiny-skia, and run
the wgpu suite locally at each cut as a checklist step, with its output kept. REQ-005
is then "partly, by procedure". **Q-2:** whichever driver Unit 0 shows produces
`wgpu` without flake, with Vulkan (lavapipe) preferred. **Q-3:** Unit 0 measures the
`iced_test::screenshot` route to scale 1. If it fails, peak coverage at scale 2 is
the bound, written down. **Q-4:** a pixel floor failing where the style floor passes
is a defect, fixed by geometry, and the floor is never relaxed. **Q-5:** Unit 0, R-1
and R-2 in 0.53.0; R-3 and R-4 after Unit 0 rules.

**Done by the architect, 2026-10-19:** the "frame hashes, not pixel reads" statement
is corrected in REQ-003's status line and in RFC-105, each with a dated note. The third,
`tests/common/mod.rs`, is code and is in the handoff.
