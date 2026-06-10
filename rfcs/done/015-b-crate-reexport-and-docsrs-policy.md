# RFC-015-B — Crate Re-export and docs.rs Policy

**Status.** Implemented (v0.15.0)
**Tracks.** Public API hygiene / documentation.
**Touches.** `crates/snora/Cargo.toml` (add metadata.docs.rs),
`docs/src/getting-started/01-install.md` (version bump + "which crate" section),
`docs/src/reference/architecture.md` (crate-choice guidance).

> RFC-011-C (AppLayout stability) is already satisfied.
> `snora-core` already has `[package.metadata.docs.rs]` and
> `#![cfg_attr(docsrs, feature(doc_cfg))]`. Only `snora` is missing.

## 1. Audit findings

### snora crate: missing docs.rs metadata

`crates/snora/Cargo.toml` has no `[package.metadata.docs.rs]`. Consequence:
docs.rs builds `snora` without `all-features`, so feature-gated items
(widgets, lucide constants, keyboard module) may not appear.

Fix: add

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### install.md: stale version number

`docs/src/getting-started/01-install.md` says `snora = "0.10"` — must
be updated to `"0.14"` and kept in sync at each release.

### "Which crate should I depend on?" — not documented

Add to `01-install.md` and/or `reference/architecture.md`.

## 2. [Decision] doc(cfg) via cfg_attr(docsrs)

`snora` already has `#![cfg_attr(docsrs, feature(doc_cfg))]` in lib.rs.
With the new Cargo metadata, docs.rs will now build with all features
and the `doc_cfg` attribute will annotate feature-gated items. No nightly
required — this is a docs.rs build-time attribute, not a user-facing one.

## 3. [Decision] re-export surface is correct

The current `snora::pub use snora_core::{…}` surface is complete and
intentional. No re-exports need adding or removing. The audit confirms:
- Core vocabulary: all exported ✅
- Engine functions (`render`, `toast::*`, `keyboard::*`): exported ✅
- Widget re-exports: feature-gated under `snora::widget` ✅
- Lower-crate implementation helpers: not exported ✅
- iced types: not broadly re-exported (correct) ✅

## 4. Acceptance criteria

- `snora/Cargo.toml` has `[package.metadata.docs.rs]` with `all-features`.
- `01-install.md` shows `snora = "0.14"` and the "Which crate?" answer.
- `cargo doc --workspace --all-features --no-deps` succeeds.
- CHANGELOG notes the docs.rs metadata fix under Changed.
