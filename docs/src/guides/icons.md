# Icons

`Icon` is a single enum with feature-gated variants. Choose your icon
source per call; nothing is global.

```rust,ignore
pub enum Icon {
    Text(String),                      // always available
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}
```

When a feature is disabled the variant does not exist in the enum. No
runtime "unknown icon kind" branch is reachable.

## `Icon::Text` — the always-available path

```rust,ignore
let i: Icon = "★".into();                // From<&str>
let i: Icon = String::from("★").into();  // From<String>
let i = Icon::Text("✓".into());           // explicit
```

Strings can be a single Unicode glyph (`★`, `✓`, `↓`, `🛠`) or a short
label (`"OK"`). The engine renders text icons at the same default
font size as labels in built-in widgets, so they line up visually.

This variant has no asset dependency, no feature flag, and works on
every platform that iced supports.

## `Icon::Lucide` — the curated icon set

```toml
[dependencies]
snora = { version = "0.62", features = ["lucide-icons"] }
```

```rust,ignore
use snora::Icon;
use snora::lucide;                     // re-exported variants

let i: Icon = lucide::Home.into();
let i = Icon::Lucide(lucide::Settings);
```

`lucide-icons` ships every Lucide glyph as a variant. Cargo includes
only the ones you reference at compile time — `Icon::Lucide(lucide::Home)`
does not pull `lucide::Settings` into the binary.

## `Icon::Svg` — your own assets

```toml
snora = { version = "0.62", features = ["svg-icons"] }
```

```rust,ignore
let i = Icon::Svg(std::path::PathBuf::from("assets/logo.svg"));
```

The engine reads the file at render time using iced's SVG widget.
Pixel size is the same default as the other variants.

### The path is yours to trust

**snora does not validate the path or the file.** `Icon::Svg` hands the path to
iced, which reads it from disk at render time and parses it as SVG. That is a
filesystem read and an XML parser, on whatever path your application supplies.

For the ordinary case — an `assets/` directory you ship — that is exactly what
you want and there is nothing to think about.

**If the path or the file can come from somewhere you do not control** — a plugin
directory, a theme a user installs, anything downloaded — then the trust boundary
is yours, not ours. snora adds no sandbox, no path restriction, and no size or
complexity limit beyond whatever the SVG renderer imposes. Validate before you
construct the `Icon`.

We state this because *"your own assets"* above is a heading, not a guarantee,
and nothing in the type signature says which of the two situations you are in.

## Sizing

**An icon is 1 em of the text it sits with** (0.55, RFC-115): `icon_element`
reads the host's `default_text_size` when the layout runs, so it is 16 px at
iced's default and 24 px if your application sets 24. Before 0.55 it was a
fixed 14 px that followed nothing. On the design path, snora's widgets draw
their icons at the label token (`Typography::label`, 14), unchanged. The
toast's × is 1.125 em. To name a size yourself:

```rust,ignore
use snora::widget::icon::icon_element_sized;

let big_logo = icon_element_sized(&Icon::Text("✓".into()), 24.0);
```

## ABDD: icons should not be the only signal

Icons are a *secondary* signal. Always pair them with a **visible**
text label. A tooltip is not enough: it appears only under a resting
pointer, so touch users, keyboard users and anyone with an unsteady hand
never see it. iced 0.14 has no accessibility tree, so no screen reader
reads either. The prefab `app_side_bar` shows each item's name on the
rail by default since 0.54 (RFC-114). Do the same in your custom
widgets.

*(Corrected 2026-11-07: this said a tooltip served keyboard and
screen-reader users.)*

In the same spirit, the toast renderer encodes intent via *both* the
background color and the surrounding text, so red is never the sole
signal of an error.

## Why icons are feature-gated

`Icon::Text` has no extra dependencies and is always available. The
richer icon sources are optional because:

- The `lucide-icons` crate ships ~1500 constants and contributes non-trivial
  compile time. Projects that only need emoji or text glyphs should not pay
  that cost.
- `Icon::Svg` requires iced's `svg` feature and a file path. Not all
  applications need SVG rendering.

Keeping both optional means engine-only builds (`--no-default-features`)
stay small and the CI feature matrix can verify each combination
independently.

## Supported feature combinations

```toml
# Default (includes widgets + text icons):
snora = { version = "0.62" }

# Engine only — no widget re-exports, no icon packs:
snora = { version = "0.62", default-features = false }

# Widgets + Lucide icon constants:
snora = { version = "0.62", features = ["widgets", "lucide-icons"] }

# Widgets + SVG icon support:
snora = { version = "0.62", features = ["widgets", "svg-icons"] }

# Widgets + both icon sources:
snora = { version = "0.62", features = ["widgets", "lucide-icons", "svg-icons"] }
```

`lucide-icons` and `svg-icons` are **subordinate** to `widgets`: they
gate widget-side rendering and are not meaningful without it. The CI
feature matrix tests all of these combinations on every PR.
