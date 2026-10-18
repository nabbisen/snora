# Implementation handoff — RFC-108: the default renderer is never tested, and contrast is never read from pixels

**RFC.** `rfcs/accepted/108-the-default-renderer-is-never-tested-and-contrast-is-never-read-from-pixels.md`
**Release target.** **This handoff: Unit 0, R-1 and R-2, in 0.53.0** (Q-5). R-3 and
R-4 get their own handoff once Unit 0's numbers are ruled on.
**Rulings.** All as suggested.
- Q-1: if wgpu cannot run in CI, use tiny-skia plus a local wgpu run at each cut.
- Q-2: prefer lavapipe.
- Q-3: measure the scale-1 route.
- Q-4: fix by geometry, never relax a floor.
- Q-5: as above.

**Touch list.**
- `crates/snora/tests/common/mod.rs`;
- new rendered-test file(s) under `crates/snora/tests/`;
- `crates/snora/Cargo.toml` (dev-dependency `png` only);
- for Unit 0 only, a **throwaway branch's** `ci.yaml`.

**Not** any style function, colour, `snora-design` or `snora-style`, and **not** `main`'s
`ci.yaml`. A Q-4 defect, if found, is reported, not fixed in this handoff.

## Already established (architect's probes, 2026-10-19, scratch tests since deleted)

- **The pixel route.** `Snapshot::matches_image(path)` writes a lossless RGBA PNG
  when the file does not exist. Its name is `<stem>-<renderer>.png`, where
  `<renderer>` is `renderer.name()` (`iced_test-0.14.0/src/simulator.rs:265–300,
  332–344`).
  - With `ICED_TEST_BACKEND=wgpu` the file is `…-wgpu.png`; with `tiny-skia`, it is
    `…-tiny-skia.png`.
  - The name delegates through the fallback renderer
    (`iced_renderer-0.14.0/src/fallback.rs:701`), so it names the renderer that
    actually drew.
- **Exact colour.** A 60 × 60 box filled `rgb(0.2, 0.4, 0.6)` decoded to
  `(51, 102, 153, 255)` on both renderers.
- **Scale is fixed at 2.0** in `Simulator::snapshot` (`simulator.rs:221`), so frames
  are 2048 × 1536 for the default 1024 × 768.
- **The tab bar diverges** (`app_tab_bar`, 3 tabs, active 1, LTR, unstyled):
  - light: 4,213 pixels differ, all in physical rows 69–75, max delta 127;
  - at column 1500 the rule is wgpu rows 74–75 solid `(0,0,0)`, against tiny-skia
    `73: 128`, `74: 0`, `75: 127`;
  - dark: tiny-skia `136 / 230 / 137` against wgpu `43 / 229 / 229`;
  - the underline (x = 216) is wgpu rows 70–73 solid `(107, 117, 255)`, against
    tiny-skia 70–72 solid with partial rows at 69 and 73.
- **wgpu ran locally**, one test, serialised, in 0.34–0.37 s (tiny-skia 0.06–0.11 s).
  RFC-099's crash was in parallel, so keep `--test-threads=1` for wgpu.
- CI triggers are `pull_request`, `push` to `main`, and `workflow_dispatch`. A
  branch push alone does not run CI.

## Unit 0 — feasibility (on a throwaway branch, never merged)

On branch `rfc-108-unit-0`, add **one** job to that branch's `ci.yaml`.
- It runs on `ubuntu-latest`, installs `mesa-vulkan-drivers` (lavapipe), and runs
  `ICED_TEST_BACKEND=wgpu cargo test -p snora --all-features -- --test-threads=1`,
  including the `#[ignore]`d bounded-work test, with `--release` as main does.
- Trigger it with `gh workflow run ci.yaml --ref rfc-108-unit-0`, **5 times**.
- If Vulkan fails, repeat with GL (llvmpipe: `libgl1-mesa-dri`, `WGPU_BACKEND=gl`).

**Record, per driver:**

1. **Which renderer drew**, proven by a snapshot file name, not inferred. R-1's
   helper is the instrument, so write it first.
2. Wall time of the suite, against `main`'s tiny-skia run of the same tests.
3. Crashes or flakes over the 5 runs.
4. Tests that fail only on wgpu, with their output.
5. **Q-3:** whether `iced_test::screenshot(program, theme, viewport, 1.0, duration)`
   renders a minimal `Program` that wraps `app_tab_bar`, and if so, the tab rule's
   rows at scale 1 on tiny-skia.

Then delete the branch. **These are numbers for the architect, not a decision.**

## R-1 — `common::frame_pixels`

Next to `frame_hash`, and built the same way: a fresh directory under
`CARGO_TARGET_TMPDIR`, `matches_image`, read back the one file written, and decode it
with `png` (dev-dependency `png = "0.18"`, already in the lockfile via iced).

- **Returns** `(renderer: String, width, height, rgba: Vec<u8>)`, where the renderer
  comes from the file name.
- **Asserts** the renderer equals `ICED_TEST_BACKEND` (default `tiny-skia`), so a
  silent fallback fails the test.
- **Corrects** `frame_hash`'s comment and the module header ("the only way to read
  rendered pixels").

## R-2 — contrast read from pixels, non-text elements

For each of these, find the element by **behaviour or `Id`**, not tree position.
Take its region's **modal colour** (the most common RGBA) as the drawn colour, and
assert it against the **same floor its style test uses**:

| Element | Its style test today |
|---|---|
| tab bar bottom rule | `tab_bar_bottom_rule_meets_non_text_floor` |
| active tab underline | `tab_indicator_meets_non_text_floor` |
| sidebar active fill vs rail | `sidebar_active_highlight_meets_non_text_floor_against_rail` |
| chrome container border | `chrome_container_style_border_meets_non_text_floor` |
| tooltip body border, both paths | `unstyled_…` / `styled_tooltip_body_…_border_meets_non_text_floor` |
| toast fill, every intent, both paths | `toast_body_text_meets_aa_all_intents_both_themes`, against the text colour |
| chip selected and disabled fills (design) | `design/chip/tests.rs` |

**For any element under 2 logical px** (the rule and the underline), also assert
**peak coverage**: the strongest row's colour clears the floor against the page.
At scale 2 that is the solid core row.

**Text is out of scope, and the test file says why:** glyph edges are
antialiased by design, and the floor applies to the style colour, which the
style tests already assert.

Run under tiny-skia, as CI does. Under the wgpu runs of Unit 0, R-2 runs as well,
which makes it the first contrast check on the default renderer.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`.
**Never `git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| R-1 renderer | request `ICED_TEST_BACKEND=wgpu` on a machine or container with no usable adapter, or patch the expected name |
| R-2 modal colour, one per row | set that element's style colour below its floor (e.g. the rule to `background.weak.color`) |
| R-2 peak coverage | draw the rule at 0.5 px |

**If a breakage cannot make a row fail, say so and replace the assertion.** Do not
ship it as written. Three handoff tests in a row could not fail.

## Acceptance criteria

1. Unit 0's five numbers, per driver tried, in the review request. The branch is
   deleted.
2. R-1 shown failing on a renderer mismatch, and `frame_hash`'s comment corrected.
3. Every R-2 row shown failing, then passing, on tiny-skia, plus its result under
   Unit 0's wgpu runs.
4. No style function, colour or frozen crate changed: quote the `--stat` diff.
5. CHANGELOG under Added: contrast read from drawn pixels, and the renderer asserted.
   Not REQ-005 "met": that is ruled after R-3.
