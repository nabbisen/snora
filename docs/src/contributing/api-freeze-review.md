# Public API freeze review

This page tracks readiness for declaring Snora 1.0. It is maintained
alongside the codebase: update it in any PR that changes a checked item.

**Current status (v0.25.3):** Seven of ten core gates satisfied. Remaining
blockers: gate 1 (iced major upgrade), gate 3 (confirmed third-party
production app), and gate 9 (measurement automation never fired on a
release tag — reopened by RFC-041; see below). Design-track D-gates
tracking in progress; see table below.

## Crate-level surface

| Item | Status |
|---|---|
| `snora-core` has no iced dependency | ✅ verified every release |
| `snora-widgets` depends on core + iced, not on `snora` | ✅ |
| `snora` re-exports intended vocabulary and widgets | ✅ |
| Feature flags documented and CI-tested | ✅ RFC-011-A, RFC-014-D |
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
| Z-stack order documented and tested | ✅ RFC-011-D/E, RFC-012 |
| Overlay interaction semantics documented | ✅ RFC-011-E |
| Toast ordering documented and tested | ✅ RFC-011-B |
| Toast lifecycle helpers documented and tested | ✅ |
| ABDD checklist adopted | ✅ RFC-012-A |
| Direction-sensitive integration tests | ✅ RFC-017 — 2 RTL render-semantics tests added |
| `keyboard::dismiss_on_escape` tested | ✅ 7 unit tests (RFC-014-A) |

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
| mdBook build and test green | ✅ RFC-012-D |

## 1.0 gates (current status)

| Gate | Status |
|---|---|
| 1. One iced major upgrade completed and lived on ≥1 minor | ⬜ |
| 2. Two consecutive minors without vocabulary churn | ✅ v0.13–v0.16 |
| 3. At least one third-party or production-grade app | ⬜ **verdict open; evidence updated v0.33.0.** The v0.18.1 entry (a build-failure report from `nabbisen/logolig`) is superseded. Three integrations now exist: **apimokka** (desktop GUI for apimock-rs, public repository, on 0.29.0, engine + `design`, zero `snora::widget::*` call sites), **arama** (image/video browser, on 0.25.0), and **orbok** (AI-driven document search, on 0.25.1, `widgets` + `design`, the only consumer exercising the prefab widgets and chrome geometry). Between them they have driven RFCs 045–056 across eight releases. What remains a judgement rather than a fact: whether any of these is *third-party* — all three are adjacent projects, not unaffiliated adopters — and whether "production-grade" is met by an application whose own visual-verification pass is still outstanding. Decide those two words before ticking this. |
| 4. AppLayout construction policy decided | ✅ v0.11 |
| 5. Render-semantics tests cover z-stack, dismissal, toast, RTL | ✅ v0.17 — 10 tests at the time, **11 as of v0.29.0**, including 2 RTL. **Semantic, not pixel** — see the note below. |
| 6. Feature-matrix CI stable | ✅ v0.11 |
| 7. Public API freeze review completed | ✅ v0.18 — all sections green; API declared ready pending gates 1, 3, 9 |
| 8. Showcase/workbench example exercises all major surfaces | ✅ v0.12 |
| 9a. **Binary-size** trend monitored (≥2 data points) | ✅ v0.29.0 — four post-fix rows on one runner and methodology (0.27.0, 0.27.1, 0.28.0, 0.28.1, all `ubuntu-latest`, same rustc). The series tracks real change: `widgets_diff_bytes` 44,928 → 45,056 → 46,592 → 46,720. Across the documentation-only 0.28.1, engine size moved **−0.0008%** — signal dominates noise. |
| 9b. **Compile-time** trend monitored (≥2 data points) | ⬜ **open. RFC-050 supplies the noise-controlled metric this gate has been waiting for (`design_overhead_ratio`, 2.5% spread on five identical-runner rows vs. 36–60% for the absolute columns), but the gate does not close on the RFC landing.** Closure condition: **≥2 released versions measured with `design_overhead_ratio` present** — the same discipline RFC-044 applied to itself, and the reason this row stays open even though the methodology problem is now resolved. **Read "trend monitored ✅" here, when it eventually appears, as meaning the ratio only — the absolute millisecond columns remain runner-dominated regardless of what this row says**; see `build-cost-budget.md`'s RFC-050 note for the full evidence, including the two directional controls (a documentation-only release moved every absolute column +36% to +55%; a code-changing release moved them −11% to −21%) that the ratio was unmoved by. **Status 2026-08-18 (0.36.0): the literal closure condition is now met and the gate is deliberately still open.** Two released versions carry the ratio — 0.35.0 at `0.042941` and 0.36.0 at `0.043451`, a **1.19% spread** across an interval in which both its own inputs moved ~18% (`example_hello_ms` 150,579 → 123,197). The metric is behaving exactly as selected. We are nonetheless holding to **9a's precedent, which closed on four post-fix rows** despite carrying the same "(≥2 data points)" wording — closing 9b at its literal minimum would be the first time gate 9 was closed on the weakest reading of its own condition, and this gate has been reopened or clock-reset three times (RFC-041, RFC-043, RFC-052), with RFC-041 existing *because* gate 9 was once declared satisfied on data that did not support it. Waiting is free: the ratio appends automatically on every release tag. **Close at four rows.** 0.36.1 makes three. |
| 10. No hidden feature-combination failures | ✅ (CI gate) |

**Gates satisfied: 2, 4, 5, 6, 7, 8, 9a, 10 = seven of ten, plus the
binary-size half of gate 9.**

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
compile-time measurement noise (gate 9b). The previous
"Gate 9 fully satisfied: binary-size has three CI data points" claim was
wrong on two counts: `v0.17.0`'s `runner_os` is `unknown` (not CI), and all
three rows are `N/A` — so the honest count was never eight of ten. See
`docs/src/reference/binary-size-budget.md` and `build-cost-budget.md` for
the full data-integrity record.

The 9a/9b split follows the same principle as that correction. RFC-041 was
raised because a gate had been declared satisfied on data that did not
support the claim; satisfying 9b now on a 25%-noise series would be a
quieter instance of the same mistake.

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
