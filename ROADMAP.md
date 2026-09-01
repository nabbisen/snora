# Roadmap

This document records the directions snora's maintainers expect to
take in upcoming releases, with rough priority and reasoning. It is
**not** a commitment — order and scope can change in response to
real-world usage and feedback. Items move from this document into
[CHANGELOG.md](CHANGELOG.md) when they ship.

For released history, see [CHANGELOG.md](CHANGELOG.md). For the
*why* behind closed design decisions, see
[`docs/src/contributing/design-decisions.md`](docs/src/contributing/design-decisions.md).

## Guiding principles (unchanging)

These constrain what *can* be on the roadmap:

- **Snora is a framework, not a UI components library.** New
  components only ship if they fit the skeleton + overlay model,
  serve typical desktop chrome (header / sidebar / body / footer
  / tabs / breadcrumbs / status), and do not pull snora toward
  being a generic widget library.
- **ABDD is non-negotiable.** Every layout-affecting addition must
  use logical edges (`Edge::Start` / `Edge::End`) and respect
  `LayoutDirection`. We do not accept widgets with hardcoded
  left/right.
- **`snora-core` stays iced-free.** Anything that needs iced goes
  into `snora` (engine) or `snora-widgets` (visuals).
- **Vocabulary over flags.** New configuration is expressed via a
  named enum, not a `bool` or magic constant.
- **No silent drops.** If an `AppLayout` field is populated, the
  engine renders it; a missing close sink only suppresses the
  click-outside backdrop, never the content.

## Post-0.17: toward 1.0

**Eight** of ten 1.0 gates are satisfied. The remaining path:

1. One iced major upgrade completed. ⬜
3. At least one third-party or production-grade app. ⬜
7. Public API freeze review completed. ✅ v0.18
9. Binary-size and compile-time trends (≥2 data points). ✅ **v0.37.0** —
   split into 9a/9b at v0.29.0 and both now satisfied. 9a (binary size) on
   four post-fix rows. 9b (compile time) on four `design_overhead_ratio`
   rows, closed **with its sensitivity stated rather than ticked clean**:
   the ratio moved −4.44% across a release that changed no executable
   code, so it detects a regression above roughly 10% and cannot see a 5%
   one — a ~5× improvement on the absolute columns it replaced (23–30%
   spread over the same releases), and materially weaker than 9a, whose
   series moved −0.0008% on the same kind of control. Closed because no
   better number is available: RFC-050 examined and rejected
   repeat-runs/median-of-N, and nothing else is queued. This gate had been
   reopened or clock-reset four times since v0.25.3.

Work on these proceeds alongside any v0.18+ feature work. There is no
scheduled date for 1.0.

## Snora Design System (complete — RFC-020 … RFC-040)

The Snora Design System shipped across five minor releases:

- **v0.19** — Foundation groundwork (RFC-020–030): iced-free `snora-design`
  crate, token model, WCAG AA contrast tests, iced style bridge, pilot
  button/card helpers, CI quality gates, accessibility and governance docs.
  `snora-design` was `publish = false` in v0.19.
- **v0.20** — Activation (RFC-031): `snora-design` published; release gate
  satisfied; `design` confirmed opt-in.
- **v0.21** — Primitives (RFC-032): notice, chip, and progress.
- **v0.23** — Recipes and governance (RFC-033, RFC-034): four initial recipes,
  API governance process.
- **v0.24** — Architect-review cleanup: chip contrast fix, measurement
  methodology improvements, documentation corrections.

All RFC-020 through RFC-034 are in `rfcs/done/`.

**Appearance milestone (RFC-037 … RFC-040) — complete.** A second design
phase, opened after real-world use showed that an application built on
snora looked flat and unresponsive: snora's own chrome never participated
in the token system. v0.26.0 delivered the colour half (an emitted
`iced::Theme` that chrome follows transitively); v0.27.0 the surfaces and
geometry (dialog card, derived modal dim, chrome spacing and radius).

**Elevation was never delivered and is not planned.** `Tokens` carries no
shadow or elevation scale, and adding one is a frozen-surface change under
RFC-036's covenant that no RFC in this milestone chose to pay for. Early
scoping listed it; that was an assumption about a vocabulary that does not
exist.

The `design` feature remains **opt-in** (`default = ["widgets"]`), and from
v0.27.0 the new surfaces are opt-in *per call site* on top of that — with
`design` inactive, rendering is unchanged. Making `design` default-on
requires an explicit size/build-cost review and a future RFC or release
decision — measurement alone does not automatically flip the default.

Future design work is governed by
[`api-governance.md`](docs/src/contributing/api-governance.md) and the
D-gates.

## Scheduled — the 2026-09-01 external audit

An external architect audited both the specification and the codebase.
Thirty-nine findings. **Eight were verified by the snora architect and all
eight held** — the two Criticals reproduced empirically (F-01 by a failing test,
F-13 by measurement), six more confirmed against source. **The remaining
thirty-one are taken on the audit's own evidence and are marked as such**;
each RFC requires its implementer to re-derive the figures it acts on rather
than inherit them.

One count was restated (F-29: 57 unrun tests by our measurement, not 68) and one
finding contradicted a standing ruling of ours and was right anyway (F-39 — see
RFC-087).

**Two are Critical and both are shipped today, in the default configuration.**

| Release | RFCs | Why this grouping |
|---|---|---|
| **0.40.2** | 087, 089 | **No crate code.** CI coverage, gate-5 correction, documentation sweep. Ships first because it costs nothing and removes noise from the two that matter |
| **0.41.0** | 084, **085** | **Both Criticals, together.** The plan split them so 085's new contrast suite would not delay 084's validated fix — **the suite was finished before the cut, so the condition ended and the split with it.** Holding 085 for a separate release would have cost a cycle for a reason that no longer applied (RFC-087's own finding, applied to ourselves) |
| **0.41.1** | 087, 089 | CI coverage and the documentation sweep. **Was 0.40.2 and did not ship** — 087 was scheduled *first* because it is what should have caught both Criticals, and it is now the oldest unstarted item from the audit |
| **0.42.0** | 086, 088 | Toast contrast and the `iced` feature removal — neither submitted yet, both needing their own measurements |

**Why 084 and 085 ended up in the same release after all.** The split was
right when it was made: 085 needed a contrast suite built in `snora-widgets`,
which had never had one, and bundling would have held a validated Critical fix
behind weeks of new infrastructure. **The suite was finished before 0.41.0 was
cut.** The condition ended, so the split ended — keeping it would have delayed a
Critical for a reason that had stopped being true, which is precisely what
RFC-087 was opened about. A deferral that outlives its condition is the failure;
noticing costs nothing.

**What the audit found about us, not just about the code.** Our contrast
assertions guard the token layer thoroughly — RFC-058, 063, 065, 066, 071, 081
all tightened `snora-design` — and the widget layer, which actually paints text,
has no contrast assertion of any kind. Every containment test is positive, so a
dialog that dismissed itself passed all of them. **The suites' reach ends one
crate short of the pixels**, and that is RFC-085's real subject.

## Recently shipped

- **0.41.0** — **Both Criticals from the external audit.** A click on a dialog's
  own text dismissed it: `render_dialog` wrapped content in `center()` with no
  `opaque`, while `sheet.rs` did it correctly three files away. Four surfaces
  had the same omission — the dialog, the no-sink dim, scroll, and the toast
  stack. **Every render-semantics test before this was positive-only**, which is
  why a dialog that dismissed itself passed all of them; six negative
  assertions, derived from Law 8's own table, now exist and **gate 5 is
  reopened** (eight of ten → seven). Separately, the widget layer paired colours
  across token families — menu text at **1.89:1**, a sidebar icon at **1.51:1
  under `high_contrast_dark`**, the preset that exists for low-vision users, now
  9.96 and the best of the four. **The contrast suite that found them is the
  first this project has had in `snora-widgets`**, and it states plainly what
  its own derivation cannot reach.


- **Gate 9 split (recorded v0.29.0).** Binary-size trend monitoring is
  **satisfied** — four rows on one runner and methodology, and the series
  tracks real change. Compile-time monitoring stays **open**: the same four
  rows spread 21–27%, and the documentation-only 0.28.1 moved them 8–11%
  with zero code changed, so the variance exceeds any per-release signal.
  Recorded as 9a/9b rather than ticked whole — RFC-041 exists because a gate
  was once declared satisfied on data that did not support it.
- **0.40.0** — One workspace line did three kinds of damage, found from a **live
  docs.rs build failure** on the published `snora-core` (RFC-083). The workspace
  declared `lucide-icons` with its `iced` feature, every member inherited it, and
  lucide's own manifest turns that feature into **iced with `advanced`** — so the
  dependency-free vocabulary crate pulled the whole GUI stack, its docs.rs page
  would not build, and **`iced/advanced` was silently enabled for every consumer
  using `lucide-icons`** while our governance page said it was *"not
  stable-by-default"*. Nothing in snora used the feature: `snora-widgets`'
  own comment tells us not to call the method it exists to provide. One line
  removed it; a CI gate now holds `snora-core` iced-free under every feature
  combination, matching the gate `snora-design` has always had.
- **0.39.3** — A zero-code release, cut deliberately. Its only change is
  test-side: the 12px text-size and 24px pointer-target floors each iterated a
  hand-written list of six `Typography` roles, so a seventh would have escaped
  both silently; both now destructure exhaustively and a new role is a compile
  error. **The release exists as much for its measurement row** — 0.39.2 crossed
  a `rustc 1.97.1 → 1.98.0` boundary, and a second same-compiler row is what
  makes the next real release readable against anything.
- **0.39.2** — Four findings from **tekstide**, who declined adoption and closed
  their channel, then sent the most precise report of the cycle. The **12-pixel
  text floor** — stated in two guides as *"the floor is 12 logical pixels,
  nothing else"* — was asserted by nothing, while both its neighbouring
  mandatory floors are enforced (24px pointer targets per role and padding step;
  contrast thresholds as a compile error). It is asserted now for all four
  presets, and the docs say what the assertion **cannot** cover: a consumer's
  own `Tokens`, whose fields are public and covenant-frozen. Separately, the
  decision register listed tekstide as a live concrete-app trigger for focus
  trapping **after they withdrew**, a high-contrast checklist bullet was
  mandatory and unsatisfiable in the same breath, and `menus.md` said nothing
  about in-menu keyboard traversal either way. Every register row now carries
  the date its evidence was last confirmed.
- **0.39.1** — `migrations.md` promised *"each minor release ships a focused
  migration guide"* and **six minors had none** — two of them from this month,
  written off by applying a different document's rule without checking what the
  index said. Three documents carried three rules, and a fourth statement turned
  up in the same file as the first. They now say one thing, unconditionally:
  **every minor ships a guide**, and one for a minor that broke nothing says so
  in a sentence. A derived check found **18** gaps where a hand audit found six.
  Separately, a release was being written up four times, and the letters and the
  guide were answering the same question for the same audience (RFC-080). The
  guide is now canonical and a letter carries only what is specific to a team —
  **because RFC-067's re-check reaches a future adopter through the guide and
  never through a letter they were not sent.**
- **0.39.0** — `snora::keyboard::cycle_zones` returned a type the facade did not
  export (RFC-076). A consumer depending only on `snora` could call it and match
  `Some(_)` but could not name `Cycle` — and our own doc comments told them to
  reach into `snora_core` instead, which is how the gap survived a release.
  `pub use snora_core::focus;` closes it; a sweep of all **25** `snora-core`
  public types confirms no other instance. Separately, **arama measured the
  dialog card over photographic content and found the border invisible against
  the dim** — 1.02:1 — with the dim-to-fill step carrying the separation
  (RFC-077). Swept over the full content range, `border ǀ dim` reaches **1.00:1
  in every preset**; `dim ǀ fill` never drops below 3.16. So RFC-058's repair is
  real but works at the card's *inner* edge, not the outline, and RFC-066's
  `max(…)` assertion is load-bearing on one branch. **No palette value, no
  `DIM_ALPHA`, no assertion changed.** Both found by **arama**.
- **0.38.3** — The release checklist named two files by hand, so every version
  snippet outside them had drifted (RFC-074) — `crates/snora/src/lib.rs` told
  **docs.rs readers of 0.38.2 to depend on 0.25**, and `reference/widgets.md`
  showed `0.6`, thirty-two minors stale. The checklist line now invokes a check
  that derives the expected minor from `Cargo.toml` and scans crate doc comments
  as well as the book. Separately, the **1.0 gate register disagreed with itself
  about gate 9b** in three live statements while its own table said closed
  (RFC-075) — the register other pages are told to defer to — and
  `api-governance.md`'s frozen style-bridge enumeration named **15 of 22**
  public functions, missing the six line-height helpers and `theme()`. The
  enumeration is deleted rather than extended: the covenant's own sentence
  already defines the surface completely. Sixteen findings from a commissioned
  audit; **no crate code changed**, doc comments only.
- **0.38.2** — Three pages that outlived the facts they stated, found in a
  pre-cut audit rather than by any test (RFC-073). `migrations.md` linked three
  pre-0.7 guides that were absent from `SUMMARY.md`, so mdBook never built them
  and every reader clicking got a **404 on the published site** — invisible to
  source-level link checking, since all three files exist on disk.
  `build-cost-budget.md` said **gate 9b was open** in three places; it closed at
  v0.37.0. And the accessibility checklist still called the `*_line_height()`
  helpers "deferred, not blocked" a release after they shipped. A fourth,
  unrelated instance of the same shape surfaced during the work and was fixed
  alongside. **No crate code changed at all.** `scripts/check-built-links.py`
  now exists as a manual tool — deliberately not a CI gate.
- **0.38.1** — Three assertions and pages that had drifted from the code they
  describe. `engine-surfaces.md` had published the **pre-repair** border figures
  since 0.34.0 and the **pre-RFC-065** dim figures since 0.37.0, and the
  `VISIBILITY_FLOOR` those numbers justified was so slack that `light`'s border
  could have regressed from 3.38:1 to 1.31:1 and still passed — the 0.34.0
  accessibility repair was unprotected by the very test that measured it. The
  floor is now `NON_TEXT_MIN` (3.0), unsplit, with both worst cases and the
  release each was measured at recorded beside it (RFC-071). Separately, the
  typography scale had never been stated against **iced's own default
  line-height, `Relative(1.3)`** — so `title` matches it exactly and three roles
  are deliberately tighter, while our readability guidance implied applying any
  role was uniformly an improvement (RFC-070, found by **orbok**). And contrast
  thresholds are now documented as **floors, never ceilings**, after a consumer
  asserted the opposite (RFC-072, from **knotra**). No palette value changed, no
  public API changed, nothing rendered differently.
- **0.38.0** — The typography scale stopped being half-tooled (RFC-068), and
  the book stopped claiming verification it did not have (RFC-069). `TextRole`
  carries `size` and `line_height`; `snora-style::text` had six helpers for the
  first and none for the second, so applying line-height meant reaching through
  two struct fields and hand-building `LineHeight::Relative` — in snora's own
  published examples. Six `<role>_line_height` helpers now sit beside their size
  counterparts, guarded by a test exhaustive on **two** axes: a new `Typography`
  role and a new `TextRole` field are each a compile error until tooled.
  Separately, all **111** Rust fences in `docs/src` were `rust,ignore`, and the
  policy blamed snippet shape — the real cause is that the book has no library
  path, so nothing importing `snora` can compile at any size. Twelve fences on
  the frozen covenant surface are now `{{#include}}`d from a compiled
  `publish = false` member; three that showed a *type's shape* rather than usage
  were deliberately left as prose. **No rendered output changed.** Found by
  auditing after **knotra** measured no line-height applied anywhere across
  three crates.
- **0.37.2** — Release notes now name the **re-check**, not only the correction,
  when a claim is withdrawn (RFC-067). Two consumer-facing claims were withdrawn
  in 0.34.0 — an invented `text_muted` contrast exemption, and an over-scoped
  "a focus ring cannot be rendered on iced 0.14" — both explained thoroughly,
  and neither saying what a consumer who had *already acted on them* should do.
  **Five propagations across four consumers followed, every one found by the
  consumer**: two accessibility suites narrowed on our authority (one across 28
  call sites of user-facing text), and two downstream accessibility contracts
  citing a limitation that was not one. The documentation-scope rule gains a
  fourth case and the release checklist the question that fires it; this release
  carries the retroactive re-check for both. Reported by **knotra** while
  planning a migration, corroborated by **aaai**, **orbok** and **apimokka**.
- **0.37.1** — The dim assertion became a sweep (RFC-066). RFC-065 checked the
  dialog card against the dim over three *discrete* surfaces; the dim is painted
  over whatever the application rendered, a *continuum*, and for two presets the
  true worst case is an **interior** minimum where the card's border and fill
  contrast cross — invisible to a three-point check. Nothing failed, but two
  published figures were overstated: `high_contrast_light` 7.37 → **4.58**,
  `high_contrast_dark` 5.25 → **4.45**. Demonstrated rather than argued: a
  perturbation where all three named surfaces pass at 3.16–3.21 while the swept
  minimum is **2.98** — a boundary the old check would have shipped. Reported by
  **tekstide** as a question about our method, from a team that had already
  closed its correspondence thread.
- **0.37.0** — The modal dim became a measurable surface (RFC-065). RFC-063
  closed the *role* axis — no `Palette` role can be added without declaring
  where it renders — but not the *surface* axis: the dim is derived at render
  time and was invisible to a contrast suite that can only name `Palette`
  surfaces. Measured, `light`'s dialog card was distinguishable from its own
  dimmed backdrop at **2.85:1**, below SC 1.4.11's 3:1, by either available
  signal. The derivation moved into `snora-design` as a pure function so one
  definition serves both the engine and the suite, and `DIM_ALPHA` went 0.40 →
  0.44 — an appearance change: every modal on the `design` path dims slightly
  harder. Found by transposing a defect **tekstide** reported in their *own*
  codebase, where it passed; here it failed.
- **0.36.1** — Every `ignore`-fenced doctest audited (RFC-064). `ignore` was
  the fence anyone reached for and nothing asked why: of 19, 16 were promoted
  to their strongest reachable rung — one to a full run, having carried an
  `assert_eq!` that had never executed, and 15 to `no_run`, which compiles and
  catches API drift at no runtime cost. The audit found **three documented
  examples that did not compile at all**, invisible because nothing compiled
  them. The three that remain ignored now each carry a stated reason. The
  policy's claim that doctest counts were "tracked in the release checklist"
  was false in both halves — the count was stale and the checklist line did not
  exist; both are fixed, and the policy now covers all five crates rather than
  the two it was written for. Also recorded: gate **9b**'s literal closure
  condition is met, and the gate is deliberately held to 9a's four-row
  precedent rather than closed at its own weakest reading.
- **0.36.0** — Three accessibility gates that existed only as prose now have
  mechanisms behind them. The 24×24 pointer-target rule gains a height-axis
  assertion (RFC-061) — the width axis is marked review-only, because content
  advance is not token-derivable and claiming otherwise would promise a
  guarantee we cannot measure; `chip`'s dismiss control was measured against
  iced's real fallback font at 15.0px and given a token-derived square target.
  The contrast pair list is no longer hand-maintained (RFC-063): every
  `Palette` role now declares where it renders and at what threshold, enforced
  by exhaustive destructuring, so a nineteenth role cannot be added without
  answering the question nobody asked about `text_muted` for its entire life —
  35 pairs, 140 assertions, up from 19/76. And the feature-gating status table
  no longer contradicts its own threshold (RFC-062): it had recorded "within
  budget" beside a figure 3.2× over for ten minors. Every indicator now carries
  a measured value; the crate-split trigger is verified **not** fired. RFC-061
  and RFC-063 both originate with tekstide.
- **0.35.0** — Frame-level keyboard navigation (RFC-060) and a compile-time
  trend signal that is actually a signal (RFC-050). `snora-core::focus` adds
  pure, iced-free zone vocabulary — `FocusZone`, `Cycle`, `ZonePresence`,
  `next_zone` — cycling the four skeleton slots in logical order, suspended
  while a modal is open. Snora deliberately **does not take Tab**: it supplies
  the decision and recommends F6/Shift+F6, leaving Tab meaning "next control".
  This closes a deferral that had sat at "out of v0.20 scope" for fourteen
  minors; modal focus *trapping* remains staged, now behind one measured
  question rather than "unproven". Alongside it, RFC-050 replaced the
  compile-time trend watch points with `design_overhead_ratio`, after five
  post-fix releases showed the absolute columns swinging 36–60% — including a
  documentation-only release that moved every one of them +36% to +55%.
- **0.34.0** — Border and muted-text contrast repaired (RFC-058), and two
  answers moved to where consumers read them (RFC-059). Both came from
  **tekstide**, a prospective adopter who evaluated snora end to end, declined,
  and sent their findings anyway. `border` was asserted in **none** of the
  twelve mandatory contrast pairs and shipped at 1.19–1.43:1 against WCAG 2.1
  SC 1.4.11's 3:1; asserting it surfaced a second untested role, `text_muted`,
  failing AA at 4.46:1 on one pair. First exercise of RFC-036's accessibility
  carve-out — assertion first, failure captured, then the value repaired. The
  `light`/`dark` border change is **visible**, and stated as such. Alongside it:
  a consumer-facing `design/stability.md`, because the token-surface guarantee
  they said would *"change our calculus more than any feature"* already existed
  — filed where only contributors read it.
- **0.33.1** — Typography made discoverable (RFC-057). A six-role text scale
  with size *and* line-height has existed since v0.20, tested and demonstrated
  in the workbench, and nothing told a developer it was there. Two new pages,
  plus removal of a **contributor checklist item asserting line-height was
  unusable in iced 0.14** — false, and a review gate rather than a stale
  comment, which is why it survived several design releases. Documentation
  only; nothing about rendering changed.
- **0.33.0** — `snora_widgets::design::{style, theme}` removed (RFC-056). The
  compatibility re-exports RFC-055 left behind while relocating the style layer
  are gone one release later, rather than deprecated: `#[deprecated]` on a
  `pub use` emits no warning at all, and the audience for one was hypothetical
  — nothing documents `snora-widgets` as a direct dependency. A compile error
  with a named replacement serves that reader better than a warning they may
  never see. **`snora::design::*` consumers are unaffected**; a migration guide
  covers the direct-import case. `snora-widgets` now exposes no style surface
  at all, which makes `architecture.md`'s description of it — optional prefab
  widgets, consumed through `snora` — true without qualification for the first
  time since the design system landed.
- **0.32.0** — `snora-style`, a fifth crate (RFC-055). The token→iced style
  bridge — `card_raised`, `to_iced_color`, the theme emitter — lived inside
  `snora-widgets` while having **three** consumers: the prefab widgets, the
  engine chrome's dialog card, and applications calling
  `snora::design::style::*` on their own iced widgets. One vocabulary,
  physically inside one of its three users, so the engine reached sideways and
  `design` could not be enabled without `widgets`.
  **`design` and `widgets` are now independent** — four expressible
  configurations where three existed, the new one being design without widgets.
  No public path changed and nothing is deprecated; the `snora-widgets`
  re-exports remain as compatibility shims. The default configuration is
  **byte-for-byte unchanged**, which is what proves the new crate is a genuinely
  conditional dependency.
- **0.31.0** — `snora::design::responsive_render` (RFC-053). Width exposure
  shipped in 0.28.0 rendered through the engine path unconditionally, so a
  `design`-path application adopting it silently lost the styled dialog card
  and the token-derived modal dim — responsive layout and design chrome were
  mutually exclusive. Reported by apimokka, for whom that dim *is* the
  accessibility fix their adoption existed to deliver, and who therefore could
  not adopt width exposure at all. **It also self-blocked**: RFC-046 deferred
  breakpoint behaviour pending real consumer thresholds, and the consumer who
  would supply them was blocked by this.
  **Also fixes** the compile-time measurement clean, which never reached the
  `release` profiles it was meant to invalidate (RFC-052). Gate 9b's clock
  **resets** — third methodology discontinuity; the gate moved further from
  closure, not nearer.
- **0.30.0** — `examples/responsive_body`: an engine-only responsive example
  composed the way snora's known consumers actually build (RFC-051).
  `responsive_render` shipped in 0.28.0 because a downstream team asked for
  it, yet every demonstration of it taught `AppLayout::side_bar` built from
  prefab widgets — which that team uses none of. The new example varies
  `body`'s own composition instead, with engine-only enforced by the compiler
  rather than by discipline. Additive; nothing else changed.
  **Also fixes** version snippets in the icons guide, stale since 0.28.0
  because the 0.29.0 release missed a checklist item that names that file.
- **0.29.0** — `snora-dialog-card` now names the card. Since 0.27.0 it was
  attached to the dialog's full-window centring container, so resolving it
  returned window-sized bounds and the actual card carried no identifier at
  all — a stable identifier that was present on every render and pointed at
  the wrong element. The centring container is now `snora-dialog`; the card
  keeps the `snora-dialog-card` name, re-pointed (RFC-049). **First rename
  exercised under the identifier compatibility policy** shipped in 0.28.0:
  it is a minor bump, announced in the CHANGELOG and migration guide rather
  than avoided. No deprecation bridge is possible for string identifiers, so
  a test asserting the old referent changes meaning silently — accepted only
  because no consumer had adopted 0.28.0 identifiers yet.
- **0.28.1** — From a second downstream field report (**arama**). snora's
  documentation contradicted itself about the dialog card: `overlays.md`
  promised "a centered modal card" and denied any card chrome eleven lines
  later, in the same file, since at least v0.25.0. The behaviour was correct
  and the card had shipped in 0.27.0 — the docs were what was wrong. Seven
  claim sites and four z-stack tables now distinguish what `snora::render`
  draws from what `snora::design::render` draws, and the dialogs guide
  surfaces the card instead of implying it will never exist (RFC-048). The
  rule whose absence caused this — **when a `design`-gated capability lands,
  every default-path page that denies it is in scope** — is now written down.
  Documentation only; no code changed.
- **0.28.0** — From the first substantive downstream field report.
  `snora::responsive_render` exposes the layout's available width — snora
  observed window size nowhere before, so every consumer wanting
  breakpoints wrote that plumbing themselves (RFC-046). Stable identifiers
  on every surface snora renders itself — the modal dim, backdrops, dialog
  card, sheet panel, toasts, skeleton regions — which are exactly the
  surfaces an application cannot label, because it never sees them
  (RFC-047). Identifier names are now a compatibility surface.
  **snora deliberately prescribes no breakpoint thresholds**; that is
  deferred until there is evidence about what real applications converge
  on, which this release makes gatherable.
- **0.27.1** — snora's assistive-technology position stated: it will
  integrate an accessibility tree when iced exposes one, and will not build
  a parallel abstraction meanwhile. The ABDD claim is bounded where it
  overclaimed — "accessibility correct from day one" now reads
  "layout-direction and visual accessibility … not assistive-technology
  support" (RFC-045). Documentation only.

- **0.27.0** — Appearance milestone complete. `snora::design::render`
  gives the dialog a real card and derives the modal dim from tokens —
  fixing a latent defect where the hardcoded 40% black scrim was
  *completely invisible* over `high_contrast_dark`'s pure-black background
  (RFC-039). `snora::design::widget::*` derives chrome spacing and radius
  from the token scales, replacing seven unrelated padding shapes and
  square corners (RFC-040). Both are opt-in **per call site**, not per
  feature flag: existing call sites keep their exact appearance.
  Measurement `runner_os` fixed — GitHub reserves `RUNNER_*`, so v0.26's
  override was silently ignored (RFC-044).

- **0.26.0** — Appearance milestone, first half. `snora::design::theme`
  emits an `iced::Theme` from Snora Design tokens, so stock iced widgets
  and the window background follow the same palette as snora's design
  primitives (RFC-038); DEC-02 amended to split theme-*producing*
  (permitted under `design`) from theme-*owning* (still declined), and
  RFC-020's long-unsatisfied "boundary statement in docs" criterion
  discharged (RFC-037); budget measurement methodology corrected — the
  size probes never called the features they measured, so
  `widgets_diff_bytes` had been reporting 0 (RFC-043).

- **0.25.3** — Documentation accuracy, measurement integrity, build
  reproducibility. Corrected in-tree docs that still described a
  three-crate workspace five releases after `snora-design` shipped
  (RFC-035); design-surface freeze review closing D-3/D-4 with an
  additive-only covenant (RFC-036); fixed measurement workflows that had
  **never** run on a release tag and reopened gate 9 (RFC-041); declared
  `rust-version = "1.88"` — the documented 1.85 was false; committed
  `Cargo.lock` with a weekly unpinned-build job (RFC-042). No public API,
  feature-flag, or runtime behavior change.
- **0.25.2** — Workspace resolver `"2"` → `"3"`; member globbing; version
  snippets. No source change.
- **0.25.1** — `snora::design::contrast` facade re-export.
- **0.25.0** — Measurement methodology fix (size-probe crates replacing hello/workbench diff); build-cost cold-clean fix; RFC-031 index fix; docs corrections.
- **0.24.0** — Architect-review cleanup: M-4 chip contrast fix, M-1/M-2/M-3/M-5/M-6/M-7 must-fixes, S-series should-fixes, all from v0.23 review.
- **0.23.0** — Recipes and dogfood process (RFC-033): four initial recipes (empty-state, background-task, error-recovery, result-card); RFC-034 governance formally closed. All 15 design-track RFCs complete.
- **0.22.0** — Code quality and doc audit: chip dedup, new tests (chip/notice/progress), three new design doc pages (notices/chips/progress), stale version refs cleaned.
- **0.21.0** — Notice, chip, and progress primitives (RFC-032); design workbench updated.
- **0.20.0** — Snora Design activation: `snora-design` published; RFC-031 release gate satisfied; `design` opt-in confirmed; migration guide 0.19→0.20.
- **0.19.1** — CI fixes: `measure-compile-time.sh` missing-space bug fixed;
  `binary-size.csv` schema corrected (7-field rows); Gate 9 ✅ (binary-size
  and build-cost both have ≥2 real CI data points). Remaining 1.0 blockers:
  gate 1 (iced upgrade) and gate 3 (third-party app).
  **This gate-9 claim was false and was retracted in 0.25.3 (RFC-041):**
  the workflows had never run on a release tag, and every row was `N/A`.
  Left here as written rather than edited, per the append-only record
  policy — see 0.25.3 above and 0.29.0 for where the gate actually stands.
- **0.19.0** — Snora Design System foundation (RFC-020–030, opt-in `design`
  feature): `snora-design` crate, style bridge, pilot button/card helpers,
  CI quality gates, accessibility docs, API governance, design workbench.
  `publish = false` until v0.20 activation. Docs gate fixed (book.toml,
  fence-tag policy). Migration guide: 0.18 → 0.19.
- **0.18** — Documentation maturity: contributing index; version snippets updated to 0.17; Gate 7 ✅ (API freeze review complete, 7/10 gates satisfied).
- **0.17** — 1.0 gate advancement: `Icon` gains `PartialEq`; two RTL
  render-semantics integration tests (10 total); keyboard.rs doc fence fix;
  first build-cost data points in all three CSVs; api-freeze-review.md
  fully updated (6/10 gates satisfied); Gate 2 ✅ (vocabulary stable
  v0.13–v0.16).
- **0.16** — Strategic evidence: alternate engine boundary doc; performance
  envelope + render-cost script; downstream feedback and feature-request
  issue templates; feedback-and-scope guide; README contribution section.
- **0.15** — Public surface and adoption maturity: docs.rs metadata;
  install.md version fix; versioning policy + migration template; decision
  index; starter application example.
- **0.14** — Interaction and boundary clarity: `dismiss_on_escape` helper;
  warning color const; overlay accessibility boundary; icons feature-gating
  policy; examples acceptance matrix.
- **0.13** — Design expansion: anchored popover study; API freeze review
  tracker; tooltip/persistent-toast evidence check.
- **0.12** — Semantic testing and ABDD maturity: RFC-011-D full acceptance;
  ABDD checklist; workbench example; compile-time tracking; doc test policy.
- **0.11** — Foundation hardening: main Rust CI; toast ordering bugfix;
  `AppLayout` `#[non_exhaustive]`; overlay semantics; render-semantics
  test harness; RFC directory.

## Longer-term: 1.0

Snora hits 1.0 when the API surface has been stable across a few
releases and we are confident it will not need a wholesale redesign.

The full readiness checklist is in
[`docs/src/contributing/api-freeze-review.md`](docs/src/contributing/api-freeze-review.md).

**Summary of 1.0 gates** (✅ = satisfied):

1. One iced major upgrade completed and lived on ≥1 minor. ⬜
2. Two consecutive minors without vocabulary churn. ✅ v0.13–v0.16
3. At least one third-party or production-grade app. ⬜
4. AppLayout construction policy decided. ✅ v0.11
5. Render-semantics tests cover z-stack, dismissal, toast, RTL. ✅ v0.17
6. Feature-matrix CI stable. ✅ v0.11
7. Public API freeze review completed. ✅ v0.18
8. Showcase/workbench example exercises all major surfaces. ✅ v0.12
9. Binary-size and compile-time trends monitored (≥2 data points). ✅
   v0.37.0 — 9a v0.29.0, 9b on four ratio rows, the latter closed with
   its measured sensitivity stated (see `api-freeze-review.md`).
   (reopened v0.25.3 — see above and RFC-041 / RFC-043)
10. No hidden feature-combination failures. ✅ (CI gate)

We are explicitly **not** rushing to 1.0. Pre-1.0 SemVer is serving
snora well; minor versions can carry small breaking changes when
justified, with deprecation bridges across two releases.

## Off the roadmap (deliberately not pursued)

These come up in discussion and are repeatedly declined. Listed
here so the answer is visible.

- **Form widgets** (`text_input` wrappers, validation primitives,
  `field` / `section`). iced's primitives do this; snora wrapping
  them adds layers without value. Form-heavy apps stay viable on
  snora — the AppLayout slots accept any iced element — but snora
  does not provide form shortcuts.
- **Data display widgets** (`data_table`, `chart`, `card_grid`).
  Out of snora's "framework" scope — these are UI library territory.
  Use iced canvas or a dedicated data-viz crate.
- **Decorative widgets** (`avatar`, `badge`, `chip`). Trivial enough
  to write in a few lines; absorbing them into snora would expand
  the surface without commensurate value.
- **A `snora-test` crate.** The
  [testing guide](docs/src/guides/testing.md) covers what `pub` fields
  on the vocabulary types already enable. A dedicated test-helper
  crate would freeze internal shapes into the public API.
- **Game-loop or real-time rendering support.** snora is
  retained-mode / event-driven. Real-time rendering belongs to iced
  canvas or a different framework.

If you have a use case that lands in one of the categories above
but you think snora *should* support it, open an issue with a
concrete scenario — these decisions are not absolute, just strongly
held defaults.

## How to influence this roadmap

- **Open an issue** describing your use case. Concrete app stories
  carry far more weight than abstract requests.
- **Send a PR** that demonstrates the design. Code is the most
  legible argument.
- **Reach out to the maintainer** at the email address in the
  workspace `Cargo.toml`.

The roadmap is updated alongside each release, typically in the
same PR that bumps the workspace version. Stale items are not a
sign of abandonment; they are a sign that something more pressing
arrived.
