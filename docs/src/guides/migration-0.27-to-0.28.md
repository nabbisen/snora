# Migrating from 0.27 to 0.28

v0.28 adds two capabilities that came directly from downstream field
feedback: **the layout's available width**, and **stable identifiers on the
surfaces snora renders itself**.

**No breaking changes.** Both are additive, neither is feature-gated, and
neither changes what an existing application renders.

## What changed

### `snora::responsive_render` — the layout's available width

snora previously observed window size **nowhere**. Any application wanting
breakpoints had to write window observation itself.

```rust,ignore
use snora::{AppLayout, responsive_render};

responsive_render(move |width| {
    let body = /* … */;
    let mut layout = AppLayout::new(body);
    if width >= 900.0 {
        layout = layout.side_bar(sidebar);
    }
    layout
})
```

The closure receives the available width in logical pixels and returns the
`AppLayout` to render at that width.

**snora deliberately prescribes no thresholds.** There is no `Breakpoint`
type, no default collapse behaviour, and no opinion about what 900 should
be. Those are your application's decisions; snora supplies the number.

Whether snora should later offer breakpoint-aware behaviour — a sidebar
that collapses on its own — is deliberately deferred until there is
evidence about which thresholds real applications converge on. This release
is what makes that evidence gatherable.

`snora::render` is unchanged. This is a sibling entry point, not a
replacement, and it is **not** behind the `design` feature.

### Stable identifiers on snora-rendered surfaces

A snora application was externally unobservable: no widget identifiers, no
semantic names, no state query. The only readable signal was the window
title.

The surfaces snora renders are exactly the ones your application *cannot*
label, because it never sees them — the modal dim, the menu backdrop, the
card wrapping your dialog content. Those now carry stable
`iced::widget::Id`s:

- menu backdrop, modal dim (both variants)
- dialog card, sheet panel
- toast stack, and each individual toast
- skeleton regions (header / sidebar / body / footer)

The full list is in
[`reference/rendered-surface-identifiers.md`](../reference/rendered-surface-identifiers.md).

**These names are now a compatibility surface.** Renaming or removing one
is a **minor** release, not a patch — recorded in the
[versioning policy](../contributing/versioning-policy.md). If your tests
assert on them, they will not shift under a patch upgrade.

**What this is not.** It is labels on rendered output — not a test harness,
not a state query API, and not accessibility semantics. An `Id` is not a
role. See [testing](testing.md).

## What did not change

- `snora::render`, `snora::design::render`, and every
  `snora::widget::*` / `snora::design::widget::*` function: unchanged
  signatures and unchanged rendering.
- No new dependency. `Responsive` was already reachable through `iced`.
- Nothing is feature-gated by this release.

## Upgrading

1. Change `snora = "0.27"` to `snora = "0.28"` in `Cargo.toml`.
2. That is the whole migration. Both additions are opt-in by call.

## Also in this cycle — 0.27.1

If you are coming from 0.27.0, the 0.27.1 patch stated snora's
**assistive-technology position** and bounded the ABDD claim: snora will
integrate an accessibility tree when iced exposes one, and ABDD means
layout-direction and visual accessibility — not assistive-technology
support. See the new [accessibility guide](accessibility.md).

Documentation only; no code changed in 0.27.1.

## Minimum supported Rust version

Unchanged: **1.88**, inherited from `iced` and `wgpu`.
