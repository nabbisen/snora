# Public API freeze review

This page tracks readiness for declaring Snora 1.0. It is maintained
alongside the codebase: update it in any PR that changes a checked item.

**Current status (v0.47.0):** Eight of ten core gates satisfied. Remaining
blockers: gate 1 (iced major upgrade) and gate 3 (confirmed third-party
production app). **Design-track D-gates: four of eight satisfied** — D-3 and
D-4 **re-earned at 0.47.0** after their 0.45.0 reset, and D-7 closed by the
scope review recorded in its own row. D-1, D-2, D-5 and D-6 remain open.

**All four remaining D-gates depend on something outside this project** —
D-1/D-2 on an upstream iced major, D-5 on an adopter reaching production, and
D-6 on consumer usage evidence a recipe has not yet accumulated (see its row).
The controllable part of the design track is now finished. The same is true of
the core track's two blockers, and `ROADMAP.md` says so at the top rather than
leaving 1.0 to read as a work queue.

**Both fractions are stated on purpose (audit 2026-09-12).** This header
previously gave a precise number for the core track and the phrase *"D-gates
tracking in progress"* for the design track — precise where the news was good and
vague where it was not. Vagueness also cannot register change: 0.45.0 took the
design track from three closed to one and the sentence did not move, because it
was equally true before and after. That is the shape RFC-062 was raised to
eliminate — *"a prose verdict alone is what let 'Within budget' sit beside a
3.2×-over-threshold figure for ten minors."*

**Gate 5 re-ticked 2026-09-02**, having been reopened 2026-09-01 (RFC-084) — see
its own row for why it was wrong for 24 minors and what closed it. **Gate 9
closed at v0.37.0** — 9a at v0.29.0, 9b on four `design_overhead_ratio` rows, the
latter with its sensitivity stated in the row below rather than ticked clean.

## Crate-level surface

| Item | Status |
|---|---|
| `snora-core` has no iced dependency | ✅ **Continuously enforced, not periodically verified.** `ci.yaml`'s `design-isolation` job fails if `cargo tree -p snora-core --all-features` contains `iced`, and since 0.47.0 a second step in the same job asserts `snora-core` and `snora-design` do not depend on each other in either direction. Re-confirmed by hand 2026-09-12 (0 iced edges). *(re-derived 2026-09-12)* |
| `snora-widgets` depends on core + iced, not on `snora` | ✅ `cargo tree -p snora-widgets --all-features` contains no `snora` edge — checked directly rather than inherited. Nothing gates this one: the `design-isolation` job covers the two leaves and iced-freedom, not this direction. Recorded as verified-by-hand so the difference from the row above is visible. *(re-derived 2026-09-12)* |
| `snora` re-exports intended vocabulary and widgets | ✅ **Re-derived 2026-09-12 against the 25-type surface**, not the 22-type one this row predates: 22 names are re-exported flat and the three zone-navigation types arrive via `pub use snora_core::focus` — a module rather than flat names, ruled deliberately by RFC-076 Q-1 so `keyboard::cycle_zones`'s return type is nameable from `snora` alone. Every public `snora-core` type is reachable without depending on `snora-core`. *(re-derived 2026-09-12)* |
| Feature flags documented and CI-tested | ✅ **Re-ticked 2026-09-12.** Reopened 2026-09-03 (RFC-094) because `crates/snora/Cargo.toml` documented `design` as independent of `widgets` while `--no-default-features --features design` appeared in no CI job. **The entry now exists** — `design-only` is one of ten feature-matrix combinations, and it ran green on this commit. The row closes the way RFC-094 said it would: when the entry exists, not when the claim is softened. *(re-derived 2026-09-12)* |
| Engine-only build (`--no-default-features`) supported | ✅ **Gated**: `no-default-features` is one of the ten feature-matrix entries, plus a standalone `cargo check -p snora --no-default-features` step. *(re-derived 2026-09-12)* |

## Type names and enum variants (audited v0.17.0, **re-derived 2026-09-12 at v0.47.0**)

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

### Re-derivation, 2026-09-12 — the audit held, and was three types short

This row was the first of the two stale instances that fired the unswept-rows
deferral. It read *"complete as of v0.17.0"* for twenty-nine minors. Re-derived
now by enumerating the public surface rather than re-reading the old list.

**All 22 originally audited types are still present under their audited names.**
No renames, no removals from `snora-core`.

**Three public types in `snora-core` were never covered by the audit**, all
from RFC-060's zone navigation (0.39.0): `FocusZone`, `Cycle`, and
`ZonePresence`. Put through the same seven questions:

| Question | `FocusZone` / `Cycle` / `ZonePresence` |
|---|---|
| Names clear, stable, LTR-assumption-free | ✅ `FocusZone` names slots (`Header`, `SideBar`, `Body`, `Footer`), never physical sides |
| Variants use logical concepts | ✅ `Cycle::{Forward, Backward}` — **not** `Left`/`Right`, which is the trap this question exists to catch |
| Defaults sensible under LTR and RTL | ✅ `ZonePresence::default()` is all-optional-slots-absent, the body-only case, and is direction-free. `FocusZone`/`Cycle` have no `Default`, correctly — neither has a defensible one |
| No variant too app-specific | ✅ all four zones are `AppLayout` slots; `Tab` and `Crumb` are deliberately *not* zones |
| `Debug`, `Clone` present | ✅ all three also derive `Copy`, `PartialEq`, `Eq`, `Hash` |
| `PartialEq` on value types | ✅ all three |
| Cycle order needs no RTL mirroring | ✅ the order is logical (`Header → SideBar → Body → Footer`); under RTL the sidebar moves physically but remains the start-edge rail after the header — stated in the type's own rustdoc as a deliberate ABDD decision |

**They pass.** The audit's criteria were sound enough that types added
twenty-two minors later satisfy them without amendment — which is the useful
finding, more than the tick.

**One real gap fell out of the re-derivation, and is fixed in the same change.**
`reference/vocabulary.md` opens *"Every public enum in snora-core"* and did not
list `FocusZone` or `Cycle`; the page carried a completeness claim about itself
that was false since 0.39.0. A **Zone navigation** section now covers all three
types. The Documentation-review row below is re-derived accordingly.

**Design-track vocabulary is governed separately** (D-3/D-4, the additive-only
covenant) and is not audited here. For the record at 0.47.0 it is ten types —
`Color`, `Density`, `FocusTokens`, `Palette`, `Radius`, `Spacing`, `TextRole`,
`Tokens`, `Tone`, `Typography` — with `Emphasis` and `Size` confirmed gone from
every crate (0.45.0, RFC-095). `snora-style` exposes no public struct or enum.

## Builder method review

| Item | Status |
|---|---|
| Every public field has a `#[must_use]` builder | ✅ **Re-derived mechanically 2026-09-12**: `snora-core` has **20** `pub fn …(mut self, …)` builders and **20** are preceded by `#[must_use]` — no gap. A builder that silently discards its receiver is the defect this row guards, and the count is now a command rather than an audit memory. *(re-derived 2026-09-12)* |
| Builder names are consistent | ✅ |
| `AppLayout` construction policy decided | ✅ RFC-011-C |

## Feature flag review

| Item | Status |
|---|---|
| `widgets` is the coarse default feature | ✅ |
| `lucide-icons` / `svg-icons` behavior documented | ✅ `guides/icons.md` covers both, and its per-feature install snippets are machine-checked by `scripts/check-version-snippets.sh` — so the *version* half of this page cannot go stale silently, though the *behaviour* half still relies on review. *(re-derived 2026-09-12)* |
| Feature matrix CI covers supported combinations | ✅ **Ten entries as of 0.47.0** — default, no-default-features, widgets, widgets+lucide, widgets+svg, widgets+design, widgets+design+lucide, widgets+design+svg, design-only, all-features. `design-only` was the gap RFC-094 reopened the crate-level row for; it exists now. *(re-derived 2026-09-12)* |
| Per-widget feature gates unjustified (or intentionally added) | ✅ **Re-derived 2026-09-12 against a measurement, for the first time.** `feature-gating-criteria.md`'s five indicators are all *not met*, and indicator 1 — the only one that could still have gone either way — was measured that day: **~0.2 s marginal, roughly 150× under its 30 000 ms threshold.** The trigger needs two or more indicators met; it has none. Previously this row was a bare tick. *(re-derived 2026-09-12)* |

## Semantic contract review

| Item | Status |
|---|---|
| Z-stack order documented and tested | ✅ RFC-011-D/E, RFC-012 — **narrowed to "consequences" on 2026-09-03 (RFC-094) and restored to "order" the same day, once the evidence existed.** The dialog↔sheet boundary is now directly asserted by `sheet_renders_above_dialog_when_both_overlap`: with an overlapping `SheetSize::Ratio(1.0)` sheet, a click at the dialog's own button produces no `DialogOk`, because the later-pushed sheet intercepts it. **Verified by swapping the two `layers.push()` calls: the new test fails and `dialog_and_sheet_coexist_…` still passes** — the older test provably cannot see this defect. Boundaries covered: dialog↔sheet (direct), sheet↔dim and toast↔dim (implicit, via reachability through a blocking dim), menu↔modal (Law 2 domination). **Still not asserted: the full 0–7 sequence as a sequence** — see RFC-094's Unit 2 item 3 for why a decision function was rejected rather than overlooked. *(re-derived 2026-09-03, RFC-094)* |
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
| Getting started path is current | ✅ **Now machine-checked, which is a stronger state than this row previously recorded.** `scripts/check-version-snippets.sh` (RFC-074) derives the expected minor from `Cargo.toml` and fails on any stale snippet in `getting-started/`; it is a CI gate and passed on this commit. The prose path still relies on review. *(re-derived 2026-09-12)* |
| Reference vocabulary matches source | ✅ **Re-derived 2026-09-12, after being found false.** The row claimed *all 22 core types present*; the surface is **25**, and `reference/vocabulary.md` — which opens *"Every public enum in snora-core"* — omitted `FocusZone`, `Cycle` and `ZonePresence` from 0.39.0 onward. A **Zone navigation** section was added in the same change, so the page's own completeness claim is true again. The three types were never undocumented in rustdoc (`missing_docs` is enforced) nor in the 0.38 → 0.39 guide; what was missing was the reference enumeration this row audits. *(re-derived 2026-09-12)* |
| Migration guides cover breaking pre-1.0 changes | ✅ **Now gated, not merely present.** `scripts/check-migration-guides.sh` fails on any minor from 0.39 onward lacking a guide, and the release checklist runs it **with the pending version** — the argument that makes it catch an omission during the cut rather than after, which is exactly how 0.46.0 shipped without one. *(re-derived 2026-09-12)* |
| Docs distinguish ABDD from full i18n/accessibility | ✅ Laws 7–8, overlays.md, direction guide |
| docs.rs feature annotations | ✅ Confirmed present: `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`, so docs.rs renders the full gated surface rather than the default one. *(re-derived 2026-09-12)* |
| Versioning policy documented | ✅ RFC-015-A |

## Release hygiene review

| Item | Status |
|---|---|
| CHANGELOG is complete | ✅ |
| ROADMAP is current | ✅ |
| Binary-size first data point recorded | ✅ **Closed 2026-09-12.** The row was opened because every value through 0.25.2 was `N/A` or a sandbox run — RFC-041's tag-automation bug meant CI never populated real ones. It has since been fixed: `binary-size.csv` now carries **36 rows with real CI values**, `runner_os = ubuntu-latest`, the newest appended by the `binary-size` workflow on the 0.47.0 tag. *(re-derived 2026-09-12)* |
| Compile-time first data point recorded | ✅ **Closed 2026-09-12**, same fix and same evidence shape as the row above: `compile-time.csv` carries **38 rows with real CI values**. Read the `design_overhead_ratio` column rather than the millisecond columns — the absolute figures vary 36–60% between identical-runner releases (RFC-050), which is why gate 9b was closed on the ratio. *(re-derived 2026-09-12)* |
| CI passes on clean branch | ✅ RFC-011-A |
| mdBook build and test green | ✅ RFC-012-D — holds, and is the most continuously re-derived row here: `ci.yaml`'s `docs` job runs `mdbook build` **and** `mdbook test` on every PR and push. Note it is `ci.yaml`'s copy that runs `mdbook test`, not `docs.yaml`'s, which only builds and deploys. *(re-derived 2026-09-03, RFC-094)* |

## 1.0 gates (current status)

| Gate | Status |
|---|---|
| 0. *(Not a gate — a 1.0 decision recorded here so it is not rediscovered)* **`Emphasis` and `Size` removed at 0.45.0 (RFC-095).** Both shipped in v0.19 (RFC-020…RFC-030) as shared variant vocabulary and were read by nothing for 24 minors — checked 2026-09-02, then asked of all six adopting teams (2026-09-04 letter) rather than inferred from silence (RFC-078's error), and all six confirmed neither is referenced anywhere in their trees. `Size` also shadowed `iced::Size` with no compiler error to warn a consumer who reached for the wrong one. Removal is a forbidden change under RFC-036's additive-only covenant, paid explicitly via its own reopening condition — see the D-3/D-4 rows below. `Tone` (read by `notice` and `progress`) and `Density` (a `Tokens` field) are untouched. | — |
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
| 2. Two consecutive minors without vocabulary churn | ✅ v0.13–v0.16 — **scope stated 2026-09-12, having previously been undefined.** This gate covers the **core** vocabulary (`snora-core`); the design vocabulary has its own stability gates (D-3/D-4) and its own covenant, which is why 0.45.0's removal of `Emphasis`/`Size` reset those two and not this one. Under that reading the gate is undisturbed: no `snora-core` type has been renamed or removed since the audit, and the only movement is additive (three zone-navigation types at 0.39.0). The tick was never wrong — it was unfalsifiable, because nobody had written down which vocabulary it meant. *(re-derived 2026-09-12)* |
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

## Sweep record — 2026-09-12

RFC-094 swept the seven test-backed rows (its Q-1) and left the rest listed but
unswept, with a deferral whose condition was **a second row found stale by other
work**. That condition fired: the 2026-09-12 audit found the type-names row 29
minors stale, and re-deriving D-7 found its stated per-minor scope review had
not run since v0.24 — 23 minors during which `snora-widgets/src` moved
+1,897/−788 across 28 files. Two instances is the pattern the deferral was
waiting for, so this is that sweep.

**What it changed.** Nineteen rows now carry a re-derivation date. Four gates
moved: **D-3** and **D-4** re-earned, **D-7** closed on a review actually
performed, and the crate-level feature-flag row re-ticked because the
`design-only` matrix entry RFC-094 demanded now exists. Two release-hygiene
rows closed on evidence that accumulated after they were opened — 36 and 38
rows of real CI measurements where RFC-041's bug had left `N/A`.

**What it found.** The type-names audit **held** — types added twenty-two
minors after it was written satisfy its criteria without amendment — but it was
**three types short**, and chasing that turned up a real defect:
`reference/vocabulary.md` claimed to list *"every public enum in snora-core"*
and had omitted `FocusZone` and `Cycle` since 0.39.0. Fixed in the same change.
Gate 2's tick turned out not to be wrong but **unfalsifiable**, because nobody
had recorded which vocabulary it governed; its scope is now written down.

**What it deliberately did not do.** Rows resting on judgement rather than on a
command — *"README one-liner is accurate"*, *"Docs distinguish ABDD from full
i18n/accessibility"*, *"ABDD checklist adopted"*, *"CHANGELOG is complete"*,
*"Builder names are consistent"* and the like — **were left undated on purpose.**
Dating them would assert a re-derivation that did not happen, and under RFC-094's
Q-2 convention an undated row already says the true thing: nobody has re-derived
it. A sweep that dated every row to look complete would destroy the one signal
this register has.

**The mechanism worth noting.** Several rows did not need re-deriving so much as
**reclassifying**: `snora-core` has no iced dependency, the engine-only build, the
feature matrix, the getting-started path and the migration guides are now all
enforced by CI gates that did not exist when their rows were written. Those rows
moved from *verified once* to *cannot silently become false* — which is the
distinction this project has spent RFC-090 to RFC-097 building, arriving in its
own 1.0 register.

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
| D-3. Token model stable for ≥2 consecutive minors | ✅ **Re-earned 0.47.0**, having been reset 2026-09-06 (RFC-095) when `Emphasis` and `Size` were removed under RFC-036's reopening condition. Two consecutive stable minors, **verified mechanically rather than asserted**: `crates/snora-design/src/variants.rs` and `tokens.rs` are byte-unchanged since the 0.45.0 tag, and the only diff to `snora-design` across 0.46.0 → 0.47.0 is `#![forbid(unsafe_code)]` plus doc comments in a `#[cfg(test)]` module — neither a surface change. *(re-derived 2026-09-12)*
| D-4. Style bridge stable for ≥2 consecutive minors | ✅ **Re-earned 0.47.0**, same two minors as D-3. The bridge never moved even at the reset — the covenant resets both gates together regardless of which frozen item changed — and `crates/snora-style/src` carries the same single `#![forbid(unsafe_code)]` line as its only diff across 0.46.0 → 0.47.0. `snora-style` exposes no public struct or enum at all, which is why its surface is stable by construction rather than by discipline. *(re-derived 2026-09-12)*
| D-5. ≥1 real app in serious production use of design tokens | ⬜ (coupled to core Gate 3) |
| D-6. Promotion process used at least once with evidence | ⬜ **Open, and externally gated — re-derived 2026-09-12.** All four published recipes (empty state, background task, error recovery, result card) are still status *Recipe*; none has even reached *Candidate*. That is not neglect of the process: `api-governance.md`'s promotion criterion 1 requires **use in two real applications, or one strong dogfood app plus one documented external request**, and no consumer has reported using a recipe. **Promoting something to close this gate would prove the opposite of what the gate tests**, so nothing has been promoted. This gate therefore belongs with D-5 and gate 3 as adoption-dependent, not with the controllable work. |
| D-7. No component catalog creep (scope review complete) | ✅ **Review performed 2026-09-12 — the first since v0.24**, 23 minors during which `snora-widgets/src` moved +1,897/−788 across 28 files, so the row's stated per-minor cadence had silently lapsed and the row was one of the two stale instances that fired the unswept-rows deferral. **Reviewed against `api-governance.md`'s permanent scope boundary** (*helpers ship only if direction-aware and semantics-light; forms, data grids, charts, routing, workflow engines and domain-specific cards are outside scope forever*). Full catalog enumerated: five `app_*` chrome prefabs (header, side_bar, footer, tab_bar, breadcrumb), `render_menu`, the design helpers (card, notice tones, button variants, chips, progress), three direction helpers (`row`, `row_dir`, `row_dir_three`) and the style hooks. **No form, data grid, chart, router, workflow engine or domain-specific card is present**; `card` is a generic surface container, which is the boundary's own distinction. Clean. *(re-derived 2026-09-12)* |
| D-8. `snora-design` published (`publish = false` flipped) | ✅ v0.20.0 |

**D-3 and D-4 are reopened as of 0.45.0 (RFC-095) — the first time either has
been.** From v0.20.0 through v0.44.0 the closure held, and was itself
**qualified**, not an unbroken surface: across v0.20.0 → v0.25.2,
`crates/snora-design/src/palette.rs` narrowed `Palette::roles()` from `pub`
to `#[cfg(test)] pub(crate)` (DEC-12 — a removal from the public API,
deliberate SemVer hardening against a future breaking change on role
addition to `#[non_exhaustive] Palette`), and
`crates/snora-design/src/contrast.rs`'s `composite_over` gained a
debug-only precondition (`debug_assert!(bg.is_opaque())`) with no signature
change. Both were deliberate hardening, and neither altered the token
*model* (all 18 `Palette` role fields, `Tokens`, and every preset were
byte-for-byte unchanged) or the style bridge (which changed by addition
only — `style::progress::toned`, v0.21). See RFC-036 §Evidence for the
full `git diff` record from that period.

**What actually reopened it:** `Emphasis` and `Size`, two of the four
enums in `variants.rs`, were read by nothing for 24 minors — confirmed by
all six adopting teams, not inferred from their silence — and `Size`
additionally shadowed `iced::Size` with no compiler error to warn a
consumer. Removing either is a forbidden change under the additive-only
covenant (`api-governance.md`); RFC-095 paid that price explicitly rather
than treating "nothing reads them" as license to remove them quietly. The
covenant's own condition is that a forbidden change **must** reset D-3 and
D-4 in the same change, regardless of which frozen item moved or whether
the style bridge itself was touched — the gate's value is in being
expensive to reverse, not in precisely scoping the blast radius. `Tone`
and `Density` are unaffected; the token model is otherwise unchanged.

D-3 and D-4 are re-earned the same way they were the first time: by
holding stable across two consecutive minors, no sooner than **0.47.0**.

See `docs/src/contributing/api-governance.md` for the full promotion,
deprecation, and release-review governance process.
