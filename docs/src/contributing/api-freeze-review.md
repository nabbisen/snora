# Public API freeze review

This page tracks readiness for declaring Snora 1.0. It is maintained
alongside the codebase: update it in any PR that changes a checked item.

**Current status (v0.13.0):** Several gates are now satisfied; the
remaining blockers are: iced major upgrade, vocabulary stability proof,
third-party adoption, and freeze review completion.

## Crate-level surface

| Item | Status |
|---|---|
| `snora-core` has no iced dependency | ✅ verified every release |
| `snora-widgets` depends on core + iced, not on `snora` | ✅ |
| `snora` re-exports intended vocabulary and widgets | ✅ |
| Feature flags documented and CI-tested | ✅ RFC-011-A |
| Engine-only build (`--no-default-features`) supported | ✅ |

## Type names and enum variants

Types to review before 1.0: `AppLayout`, `LayoutDirection`, `Edge`,
`Dialog`, `Sheet`, `SheetEdge`, `SheetSize`, `Toast`, `ToastIntent`,
`ToastLifetime`, `ToastPosition`, `Menu`, `MenuItem`, `MenuAction`,
`SideBar`, `SideBarItem`, `Tab`, `TabBar`, `TabAction`, `Crumb`,
`BreadcrumbAction`, `Icon`.

Review questions:
- [ ] Names are clear, stable, and free of LTR-only assumptions.
- [ ] Variants use logical concepts (`Start`/`End`) where appropriate.
- [ ] Defaults are sensible under both LTR and RTL.
- [ ] No variant is too app-specific.
- [ ] All expected derives are present (`Debug`, `Clone`, `PartialEq`, etc.).

*This section requires a dedicated audit pass before 1.0.*

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
| `lucide-icons` / `svg-icons` behavior documented | ✅ feature-gating-criteria.md |
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
| Direction-sensitive examples exist | ✅ workbench + rtl example |

## Documentation review

| Item | Status |
|---|---|
| README one-liner is accurate | ✅ |
| Getting started path is current | ✅ |
| Reference vocabulary matches source | ⬜ needs audit pass |
| Migration guides cover breaking pre-1.0 changes | ✅ 0.10→0.11 guide |
| Docs distinguish ABDD from full i18n/accessibility | ✅ Laws 7–8, direction guide |
| docs.rs feature annotations clean | ⬜ RFC-015-B |

## Release hygiene review

| Item | Status |
|---|---|
| CHANGELOG is complete | ✅ |
| ROADMAP is current | ✅ |
| Binary-size rows exist (≥2 releases) | ⬜ pending first tag |
| Compile-time trend data (≥2 releases) | ⬜ infra ready (RFC-012-C) |
| CI passes on clean branch | ✅ RFC-011-A |
| mdBook build and test green | ✅ RFC-012-D |

## 1.0 gates (current status)

| Gate | Status |
|---|---|
| 1. One iced major upgrade completed and lived on ≥1 minor | ⬜ |
| 2. Two consecutive minors without vocabulary churn | ⬜ |
| 3. At least one third-party or production-grade app | ⬜ |
| 4. AppLayout construction policy decided | ✅ v0.11 |
| 5. Render-semantics tests cover z-stack, dismissal, toast, RTL | ✅ v0.12 |
| 6. Feature-matrix CI stable | ✅ v0.11 |
| 7. Public API freeze review completed (this doc) | ⬜ in progress |
| 8. Showcase/workbench example exercises all major surfaces | ✅ v0.12 |
| 9. Binary-size and compile-time trends monitored (≥2 data points) | ⬜ |
| 10. No hidden feature-combination failures | ✅ (CI gate) |

Gates 4, 5, 6, 8, 10 are satisfied. The remaining five are the real 1.0
blockers; do not declare 1.0 until all ten are ✅.

## How to use this document

- Open this file in any PR that changes a public type, feature flag,
  builder method, or documentation item.
- Update the relevant row(s) to reflect the new state.
- If you are completing a gate, add the version number.
- This document is **not** a process checklist run once at 1.0 — it is
  a living readiness tracker maintained from now until 1.0.
