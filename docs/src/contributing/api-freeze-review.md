# Public API freeze review

This page tracks readiness for declaring Snora 1.0. It is maintained
alongside the codebase: update it in any PR that changes a checked item.

**Current status (v0.18.0):** Seven of ten gates satisfied. Gate 7
(public API freeze review) completed this release — all checklist
sections green, maintainer declares API ready for 1.0 pending gates
1, 3, and 9.

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
| Binary-size first data point recorded | ✅ v0.17.0 (sandbox; CI will populate real values) |
| Compile-time first data point recorded | ✅ v0.17.0 (sandbox; CI will populate real values) |
| CI passes on clean branch | ✅ RFC-011-A |
| mdBook build and test green | ✅ RFC-012-D |

## 1.0 gates (current status)

| Gate | Status |
|---|---|
| 1. One iced major upgrade completed and lived on ≥1 minor | ⬜ |
| 2. Two consecutive minors without vocabulary churn | ✅ v0.13–v0.16 |
| 3. At least one third-party or production-grade app | ⬜ first downstream user identified (v0.18.1 build failure report from nabbisen/logolig) |
| 4. AppLayout construction policy decided | ✅ v0.11 |
| 5. Render-semantics tests cover z-stack, dismissal, toast, RTL | ✅ v0.17 — 10 tests including 2 RTL |
| 6. Feature-matrix CI stable | ✅ v0.11 |
| 7. Public API freeze review completed | ✅ v0.18 — all sections green; API declared ready pending gates 1, 3, 9 |
| 8. Showcase/workbench example exercises all major surfaces | ✅ v0.12 |
| 9. Binary-size and compile-time trends monitored (≥2 data points) | ✅ binary-size: v0.17.0, v0.19.0, v0.19.1 on ubuntu-latest. build-cost: v0.17.0 (sandbox), v0.19.1 on ubuntu-latest. |
| 10. No hidden feature-combination failures | ✅ (CI gate) |

**Gates satisfied: 2, 4, 5, 6, 7, 8, 10 = seven of ten.**

Remaining blockers: iced upgrade (gate 1), third-party app (gate 3).
Gate 9 fully satisfied: binary-size has three CI data points (v0.17.0,
v0.19.0, v0.19.1); build-cost has two (v0.17.0 sandbox, v0.19.1 ubuntu-latest).

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
| D-1. One iced major upgrade survived with design feature enabled | ⬜ |
| D-2. Minimal path clean after iced upgrade | ⬜ |
| D-3. Token model stable for ≥2 consecutive minors | ⬜ (v0.20 is the first design minor) |
| D-4. Style bridge stable for ≥2 consecutive minors | ⬜ (v0.20 is the first design minor) |
| D-5. ≥1 real app in serious production use of design tokens | ⬜ |
| D-6. Promotion process used at least once with evidence | ⬜ |
| D-7. No component catalog creep (scope review complete) | ⬜ (review at each minor) |
| D-8. `snora-design` published (`publish = false` flipped) | ⬜ (v0.20 activation pending) |

See `docs/src/contributing/api-governance.md` for the full promotion,
deprecation, and release-review governance process.
