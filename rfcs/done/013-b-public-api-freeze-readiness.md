# RFC-013-B — Public API Freeze Readiness

**Status.** Implemented (v0.13.0)
**Tracks.** Stabilization / Governance.
**Touches.** `docs/src/contributing/api-freeze-review.md` (new),
`docs/src/SUMMARY.md`, ROADMAP.md (1.0 gates expanded).

## 1. Summary

Add the public API freeze review checklist and update the 1.0 gates in
the roadmap. Many items on the checklist are already satisfied by the
v0.11/v0.12 work; this RFC records the current state and identifies what
remains before 1.0 is realistic.

## 2. Current state of each checklist section

### 5.1 Crate-Level Surface
- `snora-core` iced-free: ✅ verified every release.
- `snora-widgets` depends on core+iced not snora: ✅
- `snora` re-exports vocabulary and widgets: ✅
- Feature flags documented and CI-tested: ✅ (RFC-011-A)
- Engine-only build: ✅

### 5.2 Type Names
All 17 listed types exist and are stable across the 0.11/0.12 releases.
Derives audit needed at freeze time.

### 5.3 Builder Methods
- All AppLayout fields have `#[must_use]` builders: ✅ (RFC-011-C audit)
- AppLayout construction policy decided: ✅

### 5.4 Feature Flags
- `widgets` coarse default: ✅
- Feature matrix CI: ✅ (RFC-011-A)
- Per-widget gates unjustified: ✅

### 5.5 Semantic Contract
- Z-stack documented and tested: ✅ (RFC-011-D/E, RFC-012 expansion)
- Overlay interaction semantics: ✅ (RFC-011-E)
- Toast ordering documented and tested: ✅ (RFC-011-B)
- ABDD checklist adopted: ✅ (RFC-012-A)
- Direction-sensitive examples: ✅ (workbench + rtl example)

### 5.6 Documentation
- README accurate: ✅
- Getting started current: ✅
- Reference vocabulary matches source: needs v0.13 audit
- Migration guides: ✅ (0.10→0.11 exists)
- ABDD vs i18n distinguished: ✅ (overlay semantics Law 8, direction guide)

### 5.7 Release Hygiene
- CHANGELOG complete: ✅
- ROADMAP current: ✅
- Binary-size rows: pending first tag
- Compile-time data: pending first tag (RFC-012-C infra ready)
- CI green: ✅ (RFC-011-A)
- mdBook test green: ✅ (RFC-012-D)

## 3. Updated 1.0 gates

The planning-pack gates (§7) plus additions from this study:

1. One iced major upgrade completed and lived on for ≥1 minor.
2. Two consecutive minors without vocabulary churn.
3. At least one third-party or production-grade app.
4. AppLayout construction policy decided. **✅ (v0.11)**
5. Render-semantics tests cover z-stack, dismissal, toast, RTL. **✅ (v0.12)**
6. Feature-matrix CI stable. **✅ (v0.11)**
7. Public API freeze review completed (this RFC).
8. Showcase/workbench example exercises all major surfaces. **✅ (v0.12)**
9. Binary-size and compile-time trends monitored with ≥2 data points.
10. API freeze review doc exists and is used.
11. No hidden feature-combination failures.

Gates 4, 5, 6, 8 are now satisfied. Gates 1, 2, 3, 7, 9, 10, 11 remain.

## 4. Acceptance criteria

- `api-freeze-review.md` exists with the populated checklist and current
  status of each item.
- ROADMAP 1.0-gates section updated with the expanded list including
  satisfied items marked.
- Checklist is referenced from the contributing docs index.
