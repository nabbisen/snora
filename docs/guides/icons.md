# Icons

`Icon` is a single enum with feature-gated variants. Choose your icon
source per call; nothing is global.

```rust
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

```rust
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
snora = { version = "0.5", features = ["lucide-icons"] }
```

```rust
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
snora = { version = "0.5", features = ["svg-icons"] }
```

```rust
let i = Icon::Svg(std::path::PathBuf::from("assets/logo.svg"));
```

The engine reads the file at render time using iced's SVG widget.
Pixel size is the same default as the other variants.

## Sizing

The default size is 14 px to match the default body text. To override:

```rust
use snora::widget::icon::icon_element_sized;

let big_logo = icon_element_sized(&Icon::Text("✓".into()), 24.0);
```

## ABDD: icons should not be the only signal

Icons are a *secondary* signal. Always pair them with a text label or
a tooltip — keyboard users, screen-reader users, and users with low
vision rely on the text. The prefab `app_side_bar` enforces this by
requiring `tooltip: String` on every `SideBarItem`; do the same in
your custom widgets.

In the same spirit, the toast renderer encodes intent via *both* the
background color and the surrounding text, so red is never the sole
signal of an error.
