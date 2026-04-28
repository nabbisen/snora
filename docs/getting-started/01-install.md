# 1 — Install

Add snora and iced to your `Cargo.toml`. snora targets **iced 0.14** and
**Rust edition 2024** (rustc ≥ 1.85).

```toml
[dependencies]
iced  = { version = "0.14", features = ["tokio"] }
snora = "0.5"
```

You normally do **not** depend on `snora-core` directly. The `snora` crate
re-exports the entire vocabulary (`AppLayout`, `Toast`, `ToastPosition`,
`Dialog`, `BottomSheet`, `LayoutDirection`, …), so a single `use snora::…`
suffices.

## Optional features

| Feature | What it adds | When to enable |
|---|---|---|
| `lucide-icons` | `Icon::Lucide` variant + `snora::lucide` re-exports | You want the [Lucide icon set](https://lucide.dev/) |
| `svg-icons` | `Icon::Svg(PathBuf)` variant | You want to load custom SVG files at runtime |

Enable them on the `snora` line:

```toml
snora = { version = "0.5", features = ["lucide-icons"] }
```

When a feature is disabled the corresponding `Icon` variant does not
exist in the enum at all — there is no runtime "feature missing" branch.

## Verify

```text
cargo build
```

If the build succeeds you are ready for [Hello, snora](02-hello-world.md).
