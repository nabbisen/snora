# Public API freeze review

This page tracks readiness for declaring Snora 1.0. It is maintained
alongside the codebase: update it in any PR that changes a checked item.

**Current status (v0.43.0):** Eight of ten core gates satisfied. Remaining
blockers: gate 1 (iced major upgrade) and gate 3 (confirmed third-party
production app). **Gate 5 re-ticked 2026-09-02**, having been reopened
2026-09-01 (RFC-084) — see its own row for why it was wrong for 24 minors and
what closed it. **Gate 9 closed at v0.37.0** —
9a at v0.29.0, 9b on four `design_overhead_ratio` rows, the latter with its
sensitivity stated in the row below rather than ticked clean. Design-track
D-gates tracking in progress; see table below.

## Crate-level surface

| Item | Status |
|---|---|
| `snora-core` has no iced dependency | ✅ verified every release |
| `snora-widgets` depends on core + iced, not on `snora` | ✅ |
| `snora` re-exports intended vocabulary and widgets | ✅ |
| Feature flags documented and CI-tested | ⬜ **Reopened 2026-09-03 (RFC-094): the claim is not true of one documented combination.** The feature matrix covers nine combinations, all with `widgets` or none. `crates/snora/Cargo.toml` documents `design` as *independent of `widgets`* — *"everything else works with `design` alone"* — and `--no-default-features --features design` appears in **no CI job**. It is not broken: it compiles and its suites pass (42/17/7, checked). So this is one missing matrix entry, not a defect hunt, and the row re-ticks when the entry exists. *(re-derived 2026-09-03, RFC-094)* |
| Engine-only build (`--no-default-features`) supported | ✅ |

## Type names and enum variants (audit v0.17.0)

Types audited: `AppLayout`, `LayoutDirection`, `Edge`, `Dialog`, `Sheet`,
`SheetEdge`, `SheetSize`, `Toast`, `ToastIntent`, `ToastLifetime`,
`ToastPosition`, `Menu`, `MenuItem`, `MenuAction`, `SideBar`, `SideBarItem`,
`Tab`, `TabBar`, `TabAction`, `Crumb`, `BreadcrumbAction`, `Icon`.

| Question | Status |
|---|---|
| Names clear, stable, LTR-assumption-free | ✅ all use `Start`/`End` logical edges |
| Variants use logical concepts where appropriate | ✅ `SheetEdge`, `Edge`, `ToastPosition` use `Start`/`End` |
| Defaults sensible under LTR and RTL | ✅ `TopEnd`, `Ltr`, `Bottom` all correct |
| No variant too app-specific | ✅ all types are framework-level |
| `Debug`, `Clone` present on all public types | ✅ verified by CI (derives required for `PartialEq` impls) |
| `PartialEq` on value types | ✅ `LayoutDirection`, `Edge`, `SheetEdge`, `ToastIntent`, `ToastPosition`, `ToastLifetime`, `TabAction`, `BreadcrumbAction`, `MenuAction` — all ✅. `Icon` gets `PartialEq` in v0.17.0. `Dialog`/`Sheet`/`AppLayout` contain `Node` (cannot derive without bound — correct). |
| `SheetSize` missing `Eq` | ✅ intentional — `Ratio(f32)` / `Pixels(f32)` contain `f32` |

Type-names audit: **complete as of v0.17.0.**

## Builder method review

| Item | Status |
|---|---|
| Every public field has a `#[must_use]` builder | ✅ RFC-011-C audit |
| Builder names are consistent | ✅ |
| `AppLayout` construction policy decided | ✅ RFC-011-C |

## Feature flag review

| Item | Status |
|---|---|
| `widgets` is the coarse default feature | ✅ |
| `lucide-icons` / `svg-icons` behavior documented | ✅ RFC-014-D, icons.md |
| Feature matrix CI covers supported combinations | ✅ RFC-011-A |
| Per-widget feature gates unjustified (or intentionally added) | ✅ |

## Semantic contract review

| Item | Status |
|---|---|
| Z-stack **consequences** documented and tested | ✅ RFC-011-D/E, RFC-012 — **narrowed 2026-09-03 (RFC-094): the row said "order" and the tests do not check order.** `modal_with_no_close_sink_still_blocks_pointer_at_dim` and `modal_dim_with_close_sink_blocks_wheel_scroll` (RFC-084) assert pairwise consequences of the layer sequence, not the sequence itself; **swapping layers 5 and 6 in `render()` would fail no test in the tree.** These are also gate 5's own z-stack evidence, so this row was never independently derived. `dialog_and_sheet_coexist_sheet_content_reachable` is weaker than it looks: its dialog and sheet do not spatially overlap, so it proves both are reachable, not that 6 paints over 5. *(re-derived 2026-09-03, RFC-094)* |
| Overlay interaction semantics documented | ✅ RFC-011-E |
| Toast ordering documented and tested | ✅ RFC-011-B — holds. `top_positions_render_reverse_chronological` and `bottom_positions_render_chronological` assert `render_order_for` by equality across every `ToastPosition`, which is the complete shape for a value claim; **the absence of a negative assertion here is not gate 5's defect**, because ordering has nothing to block. Narrow gap recorded rather than treated as one: `render_toasts`'s two match arms are not independently tested, so swapping them would fail nothing. *(re-derived 2026-09-03, RFC-094)* |
| Toast lifecycle helpers documented and tested | ⬜ **Reopened 2026-09-03 (RFC-094): false as written.** The row covers two helpers. `sweep_expired` is genuinely tested — `sweep_drops_only_expired_transient` asserts both that live and persistent toasts remain and that a dead transient is removed. **`subscription` has no test at all**: nothing in any `#[test]` calls it, and its only other appearances are `rust,no_run` doctests, which check that the example compiles rather than that an empty or all-persistent queue yields `Subscription::none()`. *(re-derived 2026-09-03, RFC-094)* |
| ABDD checklist adopted | ✅ RFC-012-A |
| Direction-sensitive integration tests | ✅ RFC-017 — **3** RTL render-semantics tests (`sheet_end_edge_reachable_under_rtl`, `toast_dismiss_reachable_under_rtl`, and `toast_body_click_does_not_reach_content_beneath_under_rtl`, added 0.43.0). **Justified, and the row now carries the ratio it was missing:** three RTL tests exist against **roughly fifteen LTR scenarios** in `render_semantics.rs`, so this is coverage, not parity. Dialog-dismissal blocking, wheel-scroll blocking, no-close-sink degradation, menu-backdrop dismissal and dialog+sheet coexistence have **no RTL variant** — if mirroring broke one of those alone, nothing would catch it. The row read as parity because it stated a count and not a denominator. *(re-derived 2026-09-03, RFC-094)* |
| `keyboard::dismiss_on_escape` tested | ✅ 7 unit tests (RFC-014-A) — **holds, re-counted independently.** `keyboard.rs` has 11 `#[test]` functions: 7 call `dismiss_on_escape`, 4 call `cycle_zones`. Four of the seven assert `None`, so this row is justified by evidence rather than by luck. *(re-derived 2026-09-03, RFC-094)* |

## Documentation review

| Item | Status |
|---|---|
| README one-liner is accurate | ✅ |
| Getting started path is current | ✅ v0.15 — version updated to 0.14 |
| Reference vocabulary matches source | ✅ audited v0.18 — all 22 core types present, all 13 widget functions covered, all 4 defaults correct |
| Migration guides cover breaking pre-1.0 changes | ✅ 0.10→0.11 guide + template |
| Docs distinguish ABDD from full i18n/accessibility | ✅ Laws 7–8, overlays.md, direction guide |
| docs.rs feature annotations | ✅ RFC-015-B — `snora` has `[package.metadata.docs.rs]` |
| Versioning policy documented | ✅ RFC-015-A |

## Release hygiene review

| Item | Status |
|---|---|
| CHANGELOG is complete | ✅ |
| ROADMAP is current | ✅ |
| Binary-size first data point recorded | ⬜ every row through 0.25.2 is `N/A` or a non-CI sandbox run; the tag-automation bug (RFC-041) meant CI never populated real values as this row assumed |
| Compile-time first data point recorded | ⬜ same as above; see RFC-041 |
| CI passes on clean branch | ✅ RFC-011-A |
| mdBook build and test green | ✅ RFC-012-D — holds, and is the most continuously re-derived row here: `ci.yaml`'s `docs` job runs `mdbook build` **and** `mdbook test` on every PR and push. Note it is `ci.yaml`'s copy that runs `mdbook test`, not `docs.yaml`'s, which only builds and deploys. *(re-derived 2026-09-03, RFC-094)* |

## 1.0 gates (current status)

| Gate | Status |
|---|---|
| 0. *(Not a gate — a 1.0 decision recorded here so it is not rediscovered)* **`Emphasis` and `Size` have no consumers.** Both shipped in v0.19 (RFC-020…RFC-030) as shared variant vocabulary, are re-exported through `snora::design`, and **nothing reads either** — checked 2026-09-02, not inherited. `Tone` is read by `notice` and `progress` (not buttons or chips, which the old module doc claimed); `Density` is a `Tokens` field. Frozen public surface under RFC-036's additive-only covenant, so they cannot simply be dropped. **The 1.0 question: give them consumers, or remove them in the break.** Recorded in `crates/snora-design/src/variants.rs`'s own module doc. | — |
> **Rows carry the date they were last re-derived (RFC-094, Q-2).** A row with no
> date has **never** been re-derived since it was first ticked — which is the
> useful signal, and why the marker is inline rather than a column: 33 blank
> cells would have been noise, while 33 undated rows are a statement. *(Q-2 said
> "a dated column"; the column was my guess at the mechanism and the date is the
> mechanism. Recorded as a deviation rather than pretending it was the plan.)*
>
> **Swept 2026-09-03 (RFC-094): seven test-backed rows.** Two were wrong, one
> overstated, one under-described, three held. The remaining 33 ✅ rows are
> listed in RFC-094's own report and were deliberately not swept — a sweep that
> tries to cover everything does not finish.
>
> **The finding that started it:**
>
> Gate 5 was ✅ for 24 minors because every render-semantics test behind it was
> positive-only — reachability, never containment. RFC-084 found that and
> corrected gate 5. **It corrected gate 5 and stopped there.** At least two other
> rows rest on the same body of tests and the same era of evidence:
> *"Z-stack order documented and tested"* (RFC-011-D/E, RFC-012 — ticked roughly
> thirty minors before any negative assertion existed) and *"Direction-sensitive
> integration tests"* (two positive-only RTL tests until 0.43.0 added a negative
> one).
>
> Both are **arguably justified today** — gate 5's negative assertions cover
> z-stack, and 0.43.0's RTL containment test covers direction. But they became
> justified by work done for a different gate, **accidentally, not because anyone
> re-checked them.** A tick that is right by luck is indistinguishable from one
> that is right by evidence, until someone looks.
>
> Not re-derived here, and deliberately not re-ticked or un-ticked: that is the
> owner's judgement, and the sweep is the work. Recorded so the next person does
> not have to notice it independently.

| 1. One iced major upgrade completed and lived on ≥1 minor | ⬜ |
| 2. Two consecutive minors without vocabulary churn | ✅ v0.13–v0.16 |
| 3. At least one third-party or production-grade app | ⬜ **verdict open; evidence updated v0.33.0.** The v0.18.1 entry (a build-failure report from `nabbisen/logolig`) is superseded. Three integrations now exist: **apimokka** (desktop GUI for apimock-rs, public repository, on 0.29.0, engine + `design`, zero `snora::widget::*` call sites), **arama** (image/video browser, on 0.25.0), and **orbok** (AI-driven document search, on 0.25.1, `widgets` + `design`, the only consumer exercising the prefab widgets and chrome geometry). Between them they have driven RFCs 045–056 across eight releases. What remains a judgement rather than a fact: whether any of these is *third-party* — all three are adjacent projects, not unaffiliated adopters — and whether "production-grade" is met by an application whose own visual-verification pass is still outstanding. Decide those two words before ticking this. |
| 4. AppLayout construction policy decided | ✅ v0.11 |
| 5. Render-semantics tests cover z-stack, dismissal, toast, RTL | ✅ v0.43.0 — **re-ticked 2026-09-02, on evidence rather than on the fix.** Was marked ✅ at v0.17 and should not have been; corrected 2026-09-01 (RFC-084). Every render-semantics test before RFC-084 was positive-only — a button inside an overlay is reachable, a corner click dismisses — and none asked whether pointer input that should be *blocked* actually is. It was not, in four places at once (F-01 through F-04, an external architect's audit): a click inside the dialog dismissed it, a modal with no close sink blocked nothing, the dim did not block scrolling, and clicking a toast pressed the widget beneath it. All four fixed and negative assertions added in 0.41.0 — see `crates/snora/tests/render_semantics.rs`'s own module doc for the Law-8 derivation these assertions came from. **The owner ruled on 2026-09-02 that this gate holds ⬜ until RTL has a negative assertion** — three of its four dimensions had one; RTL had only reachability tests, the same positive-only shape that made the original tick wrong, surviving in the one dimension nobody revisited. **`914fe92` closed it**, adding `toast_body_click_does_not_reach_content_beneath_under_rtl`. All four dimensions now assert that something is blocked, not only that something is reachable: z-stack (`modal_with_no_close_sink_still_blocks_pointer_at_dim`, `modal_dim_with_close_sink_blocks_wheel_scroll`), dismissal (`dialog_click_does_not_dismiss_modal`, `no_close_sink_means_no_dismiss_but_content_renders`), toast (`toast_body_click_does_not_reach_content_beneath`), RTL (the new one). Each was verified by removing the mechanism it guards and confirming it fails — this gate is ticked on tests that have been seen to fail, which is the distinction its own history is about. |
| 6. Feature-matrix CI stable | ✅ v0.11 — **not re-derived, and adjacent to a row that was reopened 2026-09-03.** This gate claims the matrix job is *stable*, which it is; the review-table row *"Feature flags documented and CI-tested"* claims *coverage*, and was reopened because `--features design` alone has none. Different claims, so this gate is untouched — but they share a mechanism, and RFC-094's whole finding is that rows sharing a mechanism get re-derived together or not at all. Out of RFC-094's scope (Q-1); named here so the adjacency is on the record. |
| 7. Public API freeze review completed | ✅ v0.18 — all sections green; API declared ready pending gates 1, 3, 9 |
| 8. Showcase/workbench example exercises all major surfaces | ✅ v0.12 |
| 9a. **Binary-size** trend monitored (≥2 data points) | ✅ v0.29.0 — four post-fix rows on one runner and methodology (0.27.0, 0.27.1, 0.28.0, 0.28.1, all `ubuntu-latest`, same rustc). The series tracks real change: `widgets_diff_bytes` 44,928 → 45,056 → 46,592 → 46,720. Across the documentation-only 0.28.1, engine size moved **−0.0008%** — signal dominates noise. |
| 9b. **Compile-time** trend monitored (≥2 data points) | ✅ **v0.37.0 — closed on four `design_overhead_ratio` rows (0.35.0, 0.36.0, 0.36.1, 0.37.0), and closed with its sensitivity stated rather than ticked clean.** ✅ here means **the ratio only**; the six absolute millisecond columns remain runner-dominated and are raw record, not a trend (RFC-050). **Measured sensitivity: the ratio moved −4.44% across 0.36.0 → 0.36.1, a release that changed doc comments and no executable code at all** — so its noise floor is ~4.4%, which is 79% of the 5.57% total spread observed across the four rows. It detects a regression above roughly 10%; it cannot see a 5% one. Over those same four releases the absolute columns spread 23.3–30.0%, so the ratio is a ~5× improvement on what it replaced. **It is materially weaker than 9a**, and the comparison should not be glossed: 9a's series moved **−0.0008%** across its own documentation-only control, roughly 5,000× less. Closed anyway because **no better number is available** — RFC-050 examined and rejected repeat-runs/median-of-N (CI minutes per release for a signal that fails no build, and it addresses within-runner jitter when the dominant effect is between-runner speed), and nothing else is queued. Holding open would not have been waiting for better data; it would have been declining to decide, on a gate already reopened or clock-reset four times since v0.25.3 (RFC-041, RFC-043, RFC-052, and this RFC-050 methodology change). |
| 10. No hidden feature-combination failures | ✅ (CI gate) |

**Gates satisfied: 2, 4, 6, 7, 8, 9, 10 = seven of ten** (gate 9
whole — both 9a and 9b closed, per the table above; gate 5 reopened
2026-09-01, RFC-084 — see its own row for why).

Gate 9 is deliberately recorded as **split** rather than ticked or held
whole. Its two measurements are in genuinely different conditions, and
collapsing them either way would misstate one of them: ticking it claims a
compile-time trend the variance contradicts, holding it whole denies
binary-size work that does exactly what the gate asks.

### What gate 5 does and does not establish

`render_semantics` is what backs snora's headline compatibility claim —
that with the `design` feature inactive, rendered output is unchanged. It
is a real gate: it must pass **unmodified** across every release, and an
implementer who needs to edit it has changed composition and must escalate
rather than adjust the test.

It asserts **composition**: layer order, which surfaces materialise, which
are dismissible, and how direction mirrors them. It does not compare
pixels, and nothing in CI does.

**As of v0.33.0 the guarantee has one pixel-level confirmation, from arama.**
They split their upgrade into two commits — version bump alone, then
`design::render` adoption — specifically so the first could be verified in
isolation, then captured the same dialog, preset and thumbnail at **0.25.0**
and **0.29.0** with the render call unchanged:

```text
md5  daae7534fc2a219d58e145339a9ea236   before-01-high_contrast_dark.png
md5  daae7534fc2a219d58e145339a9ea236   commit1-01-high_contrast_dark.png
```

Byte-identical across four minor versions, on a real application. That is
stronger than the visual comparison originally asked for: hashing converts
"we could not see a difference" into a fact.

**Its scope is one application, one preset, one dialog, four minors** — not a
general proof. The other two integrations still have no visual verification:
apimokka's is blocked on an internal gate (four commit SHAs recorded so the
comparison can be reconstructed), and orbok states theirs is outstanding for
both 0.30.0 and 0.33.0.

So the guarantee is **test-backed, with one downstream confirmation**, and
neither "unverified" nor "downstream-confirmed" is accurate on its own. The
distinction matters because this project has been bitten by the gap between a
true-sounding claim and its evidence — see gate 9's history below, and
RFC-041.

Remaining blockers: iced upgrade (gate 1), third-party app (gate 3),
render-semantics negative coverage (gate 5, reopened 2026-09-01,
RFC-084) — see the gate table above for the full status rather than a
second, separately-maintained list here. The previous "Gate 9 fully satisfied:
binary-size has three CI data points" claim was wrong on two counts:
`v0.17.0`'s `runner_os` is `unknown` (not CI), and all three rows are
`N/A` — so the honest count was never eight of ten. See
`docs/src/reference/binary-size-budget.md` and `build-cost-budget.md` for
the full data-integrity record.

The 9a/9b split followed the same principle as that correction. RFC-041 was
raised because a gate had been declared satisfied on data that did not
support the claim; 9b was closed with its real noise floor stated
(~4.4%, see the table row above) rather than glossed over, which is how
this project avoided a quieter instance of the same mistake.

## How to use this document

- Open this file in any PR that changes a public type, feature flag,
  builder method, or documentation item.
- Update the relevant row(s) to reflect the new state.
- If you are completing a gate, add the version number.
- This document is **not** a process checklist run once at 1.0 — it is
  a living readiness tracker maintained from now until 1.0.

## Snora Design gate set (separate from core 1.0)

The design-system track has its own stability gates, tracked here for
visibility alongside the core gates. These are the RFC-034 design 1.0
gates; they do not block snora core's 1.0 release.

| Gate | Status |
|---|---|
| D-1. One iced major upgrade survived with design feature enabled | ⬜ (coupled to core Gate 1) |
| D-2. Minimal path clean after iced upgrade | ⬜ (coupled to core Gate 1) |
| D-3. Token model stable for ≥2 consecutive minors | ✅ v0.20–v0.25 (token model unchanged across six consecutive minors; freeze review RFC-036) |
| D-4. Style bridge stable for ≥2 consecutive minors | ✅ v0.20–v0.25 (style bridge additive-only across six consecutive minors; freeze review RFC-036) |
| D-5. ≥1 real app in serious production use of design tokens | ⬜ (coupled to core Gate 3) |
| D-6. Promotion process used at least once with evidence | ⬜ (recipes published v0.23; no promotion yet) |
| D-7. No component catalog creep (scope review complete) | ⬜ (review at each minor — clean through v0.24) |
| D-8. `snora-design` published (`publish = false` flipped) | ✅ v0.20.0 |

The D-3/D-4 closure is **qualified**, not an unbroken surface: across
v0.20.0 → v0.25.2, `crates/snora-design/src/palette.rs` narrowed
`Palette::roles()` from `pub` to `#[cfg(test)] pub(crate)` (DEC-12 — a
removal from the public API, deliberate SemVer hardening against a future
breaking change on role addition to `#[non_exhaustive] Palette`), and
`crates/snora-design/src/contrast.rs`'s `composite_over` gained a
debug-only precondition (`debug_assert!(bg.is_opaque())`) with no signature
change. Both changes were deliberate hardening, and neither altered the
token *model* (all 18 `Palette` role fields, `Tokens`, and every preset are
byte-for-byte unchanged) or the style bridge (which changed by addition
only — `style::progress::toned`, v0.21). D-3 and D-4 ask whether the token
model and style bridge are stable, not whether the surface is frozen solid;
they are. See RFC-036 §Evidence for the full `git diff` record and the
additive-only covenant (`api-governance.md`) that now governs what may
change next.

See `docs/src/contributing/api-governance.md` for the full promotion,
deprecation, and release-review governance process.
