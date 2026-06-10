# RFC-015-B — Crate Re-export and docs.rs Policy

Status: Proposed  
Target release: v0.15  
Priority: High  
Type: Public API hygiene / documentation

## 1. Summary

Define what `snora` re-exports, how feature-gated items appear on docs.rs, and how users should choose between depending on `snora`, `snora-core`, and `snora-widgets`.

## 2. Motivation

Snora has a layered crate model: `snora-core` vocabulary, `snora-widgets` optional prefab visuals, and `snora` engine/re-export crate. This is a strong design, but re-export policy must remain predictable. Users should normally depend on `snora`, while advanced users may depend on lower crates intentionally. docs.rs should make feature-gated APIs visible and not confusing.

## 3. Goals

- Define stable re-export expectations.
- Make feature-gated API visibility explicit.
- Prevent accidental public API leaks from lower crates.
- Keep normal app imports simple.
- Improve docs.rs readability before 1.0.

## 4. Non-Goals

- Do not collapse crates.
- Do not require applications to depend on all three crates.
- Do not expose internals for testing convenience.
- Do not re-export every dependency type.
- Do not make `snora-core` depend on iced.

## 5. External Design

Public import recommendation:

```rust
use snora::{AppLayout, LayoutDirection, render};
use snora::widget::app_header; // only with widgets feature
```

Policy:

| Item category | Re-export from `snora`? | Notes |
|---|---:|---|
| Core vocabulary | Yes | Primary user path. |
| Engine functions | Yes | `render`, toast lifecycle. |
| Prefab widgets | Yes, feature-gated | Under `snora::widget`. |
| Lower-crate modules | Selectively | Avoid exposing implementation-only modules. |
| iced dependency types | No broad re-export | Use iced directly except where API returns `Element`. |
| icon constants | Feature-gated | Under clear module path. |

Docs.rs policy:

- Build docs with all intended public features enabled.
- Use `doc(cfg)` where feasible.
- Feature-gated modules must say which feature enables them.
- Engine-only usage must have a visible docs path.

## 6. Internal Design

Implementation details:

- Review `crates/snora/src/lib.rs` for all `pub use` statements.
- Review `crates/snora-widgets/src/lib.rs` for public module exposure.
- Add crate-level docs explaining features.
- Configure package metadata for docs.rs if needed:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

- Add `#![cfg_attr(docsrs, feature(doc_cfg))]` only if compatible with stable docs.rs policy and crate MSRV choices. If unstable doc cfg is not acceptable, use prose instead.

Do not make docs.rs polish require nightly for normal users.

## 7. Testing and Acceptance

Acceptance criteria:

- `cargo doc --workspace --all-features --no-deps` succeeds locally/CI if adopted.
- docs.rs metadata is present or deliberately rejected with reason.
- Public import examples compile.
- `default-features = false` docs are clear.
- No lower-crate implementation helper is accidentally advertised as recommended API.

## 8. Documentation Updates

Update:

- crate-level docs in `snora`, `snora-core`, `snora-widgets`
- README installation/import section
- `docs/src/getting-started/01-install.md`
- `docs/src/reference/architecture.md`
- `docs/src/reference/vocabulary.md`

Add "Which crate should I depend on?" section.

## 9. Compatibility and Migration

Mostly compatible.

If re-exports are removed or renamed, follow RFC-015-A migration/deprecation policy. Prefer additive docs cleanup before removal.

## 10. Open Questions

- Should advanced users be encouraged to depend on `snora-core` directly for tests?
- Should `snora-widgets` be documented as public standalone crate or internal-adjacent optional crate?
- Should docs.rs build all-features even if all-features is not the recommended app mode?
