# 1 — Install

Add snora and iced to your `Cargo.toml`. snora targets **iced 0.14** and
**Rust edition 2024**. The declared minimum supported Rust version is
**rustc ≥ 1.88**, which `iced` and `wgpu` require, and which snora's own code
needs too (it uses `let` chains, stable from 1.88).

*(Corrected 2026-11-28, from the 0.67 audit: this said snora's own code needs
only 1.85.)*

```toml
[dependencies]
iced  = { version = "0.14", features = ["tokio"] }
snora = "0.66"
```

You normally do **not** depend on `snora-core` directly. The `snora` crate
re-exports the vocabulary (`AppLayout`, `Toast`, `ToastPosition`,
`Dialog`, `Sheet`, `LayoutDirection`, …), so a single `use snora::…`
suffices. **Two exceptions until 0.67:** `ShortcutRoute` and `TargetRole`, the
types `Shortcuts::routes()` returns, are not re-exported, so code that names
them needs `snora-core`. Most applications never call `routes()`.

## Which crate should I depend on?

| Use case | Depend on |
|---|---|
| Building a typical application | `snora` only |
| Testing `update` state logic (no renderer needed) | `snora` only — `pub` fields on vocabulary types are directly assertable |
| Building an alternate engine or test double | `snora-core` directly (no iced) |
| Extending widget visuals | `snora-widgets` directly (rare; `snora` re-exports everything) |

Advanced users who want to depend on `snora-core` without iced can do so — the crate's only dependency is the optional `lucide-icons`. The `snora` umbrella crate is still the recommended path for applications.

## Optional features

| Feature | What it adds | When to enable |
|---|---|---|
| `widgets` | Prefab `app_header` / `app_side_bar` / `app_footer` / `app_tab_bar` / `app_breadcrumb` / `render_menu` / `icon_element`, and the five form controls (`snora::{checkbox, toggler, radio_group, select, slider}`). **On by default.** | You want a working app on screen quickly without writing chrome from scratch |
| `iced-default` | iced's own default features (the renderers, the windowing backends, `linux-theme-detection`). **On by default.** | Leave it on unless you name your own iced features; see [Engine-only build](#engine-only-build) |
| `design` | Snora Design: `snora::design` (tokens, the token-derived iced theme, the design primitives with `widgets`) | You want snora's tokens, presets and design components |
| `lucide-icons` | `Icon::Lucide` variant + `snora::lucide` re-exports | You want the [Lucide icon set](https://lucide.dev/). **Your application must load the font** (`snora::lucide::LUCIDE_FONT_BYTES`): see [Icons](../guides/icons.md#iconlucide--the-curated-icon-set) |
| `svg-icons` | `Icon::Svg(PathBuf)` variant | You want to load custom SVG files at runtime |

Enable them on the `snora` line:

```toml
snora = { version = "0.66", features = ["lucide-icons"] }
```

When a feature is disabled the corresponding `Icon` variant does not
exist in the enum at all — there is no runtime "feature missing" branch.

## Engine-only build

Applications that supply 100 % of their UI parts (header, sidebar,
footer, menu) and do not want the prefab widgets compiled in can opt
out:

```toml
snora = { version = "0.66", default-features = false }
```

In this configuration the `snora-widgets` crate is not pulled in,
`snora::widget` does not exist, and your build pays only for the
engine (`render`, overlay layers, toast lifecycle).

**This also turns off `iced-default`** (RFC-112), the feature that passes
iced's own defaults through, such as the renderers and windowing. An
application already has its own `iced` dependency, which supplies them, so
nothing changes for it. To choose iced's features yourself, see
[feature flags](../design/feature-flags.md#choosing-iceds-features-yourself-iced-default-rfc-112).

## Verify

```text
cargo build
```

If the build succeeds you are ready for [Hello, snora](02-hello-world.md).
