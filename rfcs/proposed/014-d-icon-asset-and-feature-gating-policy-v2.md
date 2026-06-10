# RFC-014-D — Icon, Asset, and Feature-Gating Policy v2

Status: Proposed  
Target release: v0.14  
Priority: Medium  
Type: Feature policy / dependency control

## 1. Summary

Tighten the policy for optional icon and asset features so Snora remains small while still supporting practical prefab visuals.

## 2. Motivation

Snora currently exposes optional icon-related features such as Lucide and SVG. Icons are useful for prefab navigation surfaces, but icon packs and asset handling can become stealth sources of dependency growth, compile-time cost, and binary-size growth. The project already has feature-gating criteria; this RFC applies those principles specifically to icons and assets.

## 3. Goals

- Keep text-only icons always available.
- Keep Lucide and SVG support optional.
- Prevent adding multiple overlapping icon ecosystems without evidence.
- Define acceptance criteria for new asset-related features.
- Ensure docs clearly show feature combinations.

## 4. Non-Goals

- Do not add a general asset pipeline.
- Do not bundle large icon sets by default.
- Do not add raster image management.
- Do not add theming or dynamic recoloring beyond what iced/widget code already supports.
- Do not add per-widget feature gates unless existing criteria are triggered.

## 5. External Design

Policy:

| Capability | Default? | Policy |
|---|---:|---|
| Text icon | Yes | Always available through `Icon::Text` or equivalent. |
| Lucide icon constants | No / feature-gated | Allowed behind `lucide-icons`; must be documented. |
| SVG source support | No / feature-gated | Allowed behind `svg-icons`; must be documented. |
| Additional icon pack | No | Requires RFC and evidence of repeated demand. |
| Raster asset helpers | No | Application responsibility. |

Public docs should show:

```toml
snora = { version = "..." }
snora = { version = "...", default-features = false }
snora = { version = "...", features = ["widgets", "lucide-icons"] }
snora = { version = "...", features = ["widgets", "svg-icons"] }
```

Feature combinations that are invalid or meaningless should either fail clearly or be documented.

## 6. Internal Design

Implementation checks:

- Feature definitions remain in workspace and crate `Cargo.toml` files.
- `snora-core` remains iced-free even when icon vocabulary has feature variants.
- `snora-widgets` owns rendering of icon variants.
- `snora` re-exports icon-related modules only under the correct feature combinations.
- Add `#[cfg(feature = "...")]` and docs.rs `doc(cfg)` where appropriate.

Internal anti-patterns:

- Do not put iced SVG types in `snora-core`.
- Do not make Lucide constants available when `widgets` is disabled if they require widget rendering.
- Do not add asset-loading side effects.

## 7. Testing and Acceptance

Acceptance criteria:

- Feature-matrix CI covers default, no-default, widgets-only, widgets+lucide, widgets+svg, and all-features.
- docs.rs builds with feature annotations.
- Example code in icon guide compiles or is clearly marked as feature-specific.
- Binary-size workflow records any meaningful increase from icon feature changes.

## 8. Documentation Updates

Update:

- `docs/src/guides/icons.md`
- `docs/src/contributing/feature-gating-criteria.md`
- `docs/src/reference/widgets.md`
- README feature examples

Add a short "Why icons are feature-gated" section.

## 9. Compatibility and Migration

Compatible if documentation-only.

If feature names change, use pre-1.0 deprecation bridges where feasible and document migration clearly. Prefer not to rename existing feature flags without strong reason.

## 10. Open Questions

- Should `lucide-icons` require `widgets`, or can it be meaningful with `snora-core` vocabulary alone?
- Should docs show `all-features` as a development mode but not recommended app mode?
- Should icon feature size be listed separately in the binary-size budget?
