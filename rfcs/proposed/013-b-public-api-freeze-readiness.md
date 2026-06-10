# RFC-013-B — Public API Freeze Readiness

Status: Proposed  
Target release: v0.13+ and pre-1.0  
Priority: High  
Type: Stabilization / Governance

## 1. Summary

Define a public API freeze review process for Snora before declaring 1.0. The review should examine all exported types,
feature flags, builder methods, examples, docs, and migration bridges.

## 2. Motivation

Snora is intentionally pre-1.0. Minor releases may still carry small breaking changes with deprecation bridges. That is
healthy now, but 1.0 changes the promise. Before 1.0, the project should deliberately verify that public vocabulary and
feature policies are stable enough.

## 3. Goals

- Avoid accidental 1.0 stabilization of awkward names or incomplete policies.
- Ensure feature flags are coherent.
- Ensure deprecation bridges are resolved or intentionally retained.
- Ensure docs match public API.
- Ensure 1.0 gates are objective enough to guide maintainers.

## 4. Non-Goals

- Do not rush 1.0.
- Do not freeze internal implementation details.
- Do not promise semantic version compatibility for pre-1.0 releases.
- Do not add a new governance process beyond what the small project needs.

## 5. API Freeze Checklist

### 5.1 Crate-Level Surface

- [ ] `snora-core` has no iced dependency.
- [ ] `snora-widgets` depends on `snora-core` and iced, not on `snora`.
- [ ] `snora` re-exports intended vocabulary and widgets.
- [ ] Feature flags are documented and tested.
- [ ] Engine-only build remains supported.

### 5.2 Type Names and Enum Variants

Review:

- `AppLayout`
- `LayoutDirection`
- `Edge`
- `Dialog`
- `Sheet`
- `SheetEdge`
- `SheetSize`
- `Toast`
- `ToastIntent`
- `ToastLifetime`
- `ToastPosition`
- `Menu`, `MenuItem`, `MenuAction`
- `SideBar`, `SideBarItem`
- `Tab`, `TabBar`, `TabAction`
- `Crumb`, `BreadcrumbAction`
- `Icon`

Questions:

- Are names clear and stable?
- Do variants use logical concepts where appropriate?
- Are defaults right?
- Are any variants too app-specific?
- Are any types missing `Debug`, `Clone`, `PartialEq`, or other expected derives?

### 5.3 Builder Method Review

- [ ] Every public field that should be set by users has a builder method.
- [ ] Builder names are consistent.
- [ ] Builder methods do not consume surprising values.
- [ ] `#[must_use]` is applied to builder methods.
- [ ] AppLayout construction policy from RFC-011-C is implemented.

### 5.4 Feature Flag Review

- [ ] `widgets` remains the coarse default feature.
- [ ] `lucide-icons` and `svg-icons` behavior is documented.
- [ ] Unsupported subordinate feature combinations are either impossible or documented.
- [ ] Feature matrix CI covers supported combinations.
- [ ] Per-widget feature gates are still unjustified or intentionally introduced.

### 5.5 Semantic Contract Review

- [ ] Z-stack order documented and tested.
- [ ] Overlay interaction semantics documented.
- [ ] Toast ordering documented and tested.
- [ ] Toast lifecycle helpers documented and tested.
- [ ] ABDD checklist adopted.
- [ ] Direction-sensitive examples exist.

### 5.6 Documentation Review

- [ ] README one-liner is accurate.
- [ ] Getting started path is current.
- [ ] Reference vocabulary matches source.
- [ ] Migration guides cover breaking pre-1.0 changes.
- [ ] Docs distinguish ABDD from full i18n/accessibility.
- [ ] docs.rs feature annotations are clear if adopted.

### 5.7 Release Hygiene Review

- [ ] CHANGELOG is complete.
- [ ] ROADMAP is current.
- [ ] Binary-size rows exist for recent releases.
- [ ] Compile-time trend data exists or is intentionally deferred.
- [ ] CI passes on clean branch.
- [ ] mdBook build and tests are green according to docs policy.

## 6. Internal Design

Add a pre-1.0 review document:

```text
docs/src/contributing/api-freeze-review.md
```

Or, if this should not be in the public docs, add:

```text
docs/internal/api-freeze-review.md
```

Preferred: public contributing docs, because the surface is public and external users benefit from seeing the discipline.

## 7. 1.0 Gate Update

Update roadmap with expanded gates:

1. one iced major upgrade completed;
2. two consecutive minors without vocabulary churn;
3. at least one third-party or production-grade app;
4. AppLayout construction policy decided;
5. render-semantics tests exist;
6. feature-matrix CI exists;
7. public API freeze review completed;
8. showcase/workbench example exists.

## 8. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Review delays 1.0. | That is acceptable; premature 1.0 is more expensive. |
| Checklist becomes stale. | Update it in the same PR as public API changes. |
| Review overfocuses on names and misses behavior. | Include semantic contract section. |
| Too much process for a small crate. | Keep it as a single checklist document. |

## 9. Acceptance Criteria

- API freeze checklist exists.
- Roadmap 1.0 gates are updated.
- Checklist is used before any 1.0 declaration.
- Public API changes after this RFC state whether they affect freeze readiness.
