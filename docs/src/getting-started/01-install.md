# 1 — Install

Add snora and iced to your `Cargo.toml`. snora targets **iced 0.14** and
**Rust edition 2024** (rustc ≥ 1.85).

```toml
[dependencies]
iced  = { version = "0.14", features = ["tokio"] }
snora = "0.7"
```

You normally do **not** depend on `snora-core` directly. The `snora` crate
re-exports the entire vocabulary (`AppLayout`, `Toast`, `ToastPosition`,
`Dialog`, `Sheet`, `LayoutDirection`, …), so a single `use snora::…`
suffices.

## Optional features

| Feature | What it adds | When to enable |
|---|---|---|
| `widgets` | Prefab `app_header` / `app_side_bar` / `app_footer` / `render_menu` / `icon_element`. **On by default.** | You want a working app on screen quickly without writing chrome from scratch |
| `lucide-icons` | `Icon::Lucide` variant + `snora::lucide` re-exports | You want the [Lucide icon set](https://lucide.dev/) |
| `svg-icons` | `Icon::Svg(PathBuf)` variant | You want to load custom SVG files at runtime |

Enable them on the `snora` line:

```toml
snora = { version = "0.7", features = ["lucide-icons"] }
```

When a feature is disabled the corresponding `Icon` variant does not
exist in the enum at all — there is no runtime "feature missing" branch.

## Engine-only build

Applications that supply 100 % of their UI parts (header, sidebar,
footer, menu) and do not want the prefab widgets compiled in can opt
out:

```toml
snora = { version = "0.7", default-features = false }
```

In this configuration the `snora-widgets` crate is not pulled in,
`snora::widget` does not exist, and your build pays only for the
engine (`render`, overlay layers, toast lifecycle).

## Verify

```text
cargo build
```

If the build succeeds you are ready for [Hello, snora](02-hello-world.md).
