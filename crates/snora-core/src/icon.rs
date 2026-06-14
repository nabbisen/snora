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
///
/// Implements [`PartialEq`] when all active features support it.
/// `Icon::Text` and `Icon::Svg` are always comparable; `Icon::Lucide`
/// requires `lucide-icons` to expose `PartialEq` on the inner type.
/// If `lucide-icons` is enabled, a manual impl is used that compares the
/// enum discriminant only for the `Lucide` variant.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "lucide-icons"), derive(PartialEq))]
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

/// Manual `PartialEq` when `lucide-icons` is active, since
/// `lucide_icons::Icon` does not derive `PartialEq` itself.
/// Two `Lucide` variants are considered equal only when they hold the
/// same discriminant value (compared via `as usize`).
#[cfg(feature = "lucide-icons")]
impl PartialEq for Icon {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Icon::Text(a), Icon::Text(b)) => a == b,
            (Icon::Lucide(a), Icon::Lucide(b)) => (*a as usize) == (*b as usize),
            #[cfg(feature = "svg-icons")]
            (Icon::Svg(a), Icon::Svg(b)) => a == b,
            _ => false,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_icons_equal_when_same_string() {
        let a: Icon = "★".into();
        let b: Icon = "★".into();
        assert_eq!(a, b);
    }

    #[test]
    fn text_icons_not_equal_when_different_string() {
        let a: Icon = "★".into();
        let b: Icon = "☆".into();
        assert_ne!(a, b);
    }

    #[test]
    fn text_icon_from_string_vs_str() {
        let a: Icon = "hello".into();
        let b: Icon = String::from("hello").into();
        assert_eq!(a, b);
    }

    #[cfg(feature = "svg-icons")]
    #[test]
    fn svg_icons_equal_when_same_path() {
        let a = Icon::Svg(std::path::PathBuf::from("icons/foo.svg"));
        let b = Icon::Svg(std::path::PathBuf::from("icons/foo.svg"));
        assert_eq!(a, b);
    }

    #[cfg(feature = "svg-icons")]
    #[test]
    fn svg_icons_not_equal_when_different_path() {
        let a = Icon::Svg(std::path::PathBuf::from("icons/foo.svg"));
        let b = Icon::Svg(std::path::PathBuf::from("icons/bar.svg"));
        assert_ne!(a, b);
    }
}
