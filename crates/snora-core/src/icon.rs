//! Icon vocabulary.
//!
//! snora supports icons from multiple sources. Each source is gated behind
//! a Cargo feature so that unused icon backends are eliminated at compile
//! time (DCE) — no bundled asset blob you aren't using.
//!
//! | Source | Feature flag | Variant |
//! |--------|--------------|---------|
//! | Plain text (glyph / emoji) | always available | [`Icon::Text`] |
//! | Lucide icon set | `lucide-icons` | [`Icon::Lucide`] |
//! | Custom SVG file | `svg-icons` | [`Icon::Svg`] |
//!
//! When a feature is disabled, its variant does not exist in the enum at
//! all — the compiler will refuse code that references it, so there are no
//! runtime "unimplemented!" paths.
//!
//! # Fallback ergonomics
//!
//! `Icon` accepts `&str` / `String` conversions directly so that icon fields
//! in user code can degrade gracefully even with all features disabled:
//!
//! ```rust
//! # use snora_core::Icon;
//! let icon: Icon = "★".into();        // always works
//! let icon: Icon = String::from("★").into();
//! ```

/// An icon, with feature-gated source variants.
///
/// See the crate-level documentation and the
/// [Icons guide](https://github.com/nabbisen/snora/blob/main/docs/guides/icons.md)
/// for the full discussion of when to use each variant.
#[derive(Debug, Clone)]
pub enum Icon {
    /// Renders the given string as text. The engine may choose its font
    /// and size; a single-glyph string acts as a tiny glyph icon.
    Text(String),

    /// A built-in Lucide icon. Requires the `lucide-icons` feature.
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),

    /// A custom SVG file loaded from disk. Requires the `svg-icons`
    /// feature. The engine is responsible for reading and rasterizing
    /// the file.
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}

impl From<&str> for Icon {
    fn from(s: &str) -> Self {
        Icon::Text(s.to_string())
    }
}

impl From<String> for Icon {
    fn from(s: String) -> Self {
        Icon::Text(s)
    }
}

#[cfg(feature = "lucide-icons")]
impl From<lucide_icons::Icon> for Icon {
    fn from(i: lucide_icons::Icon) -> Self {
        Icon::Lucide(i)
    }
}
