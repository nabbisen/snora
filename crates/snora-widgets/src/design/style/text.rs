//! Text style helpers for Snora Design tokens.
//!
//! These helpers derive `iced::Pixels` sizes from a [`Tokens`] typography
//! scale, avoiding magic numbers in application view code.
//!
//! Line-height is not currently configurable through `iced::widget::text` in
//! iced 0.14's standard API; the `line_height` fields in `Typography` are
//! stored in the token vocabulary for future use and for documentation
//! purposes.
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::style::text;
//! use iced::widget::text as iced_text;
//!
//! let tokens = Tokens::light();
//! let heading = iced_text("Settings").size(text::heading_size(&tokens));
//! ```

use iced::Pixels;
use snora_design::Tokens;

/// Returns the `body` text size as [`Pixels`].
#[must_use]
pub fn body_size(tokens: &Tokens) -> Pixels {
    tokens.typography.body.size.into()
}

/// Returns the `body_small` text size as [`Pixels`].
#[must_use]
pub fn body_small_size(tokens: &Tokens) -> Pixels {
    tokens.typography.body_small.size.into()
}

/// Returns the `label` text size as [`Pixels`].
#[must_use]
pub fn label_size(tokens: &Tokens) -> Pixels {
    tokens.typography.label.size.into()
}

/// Returns the `title` text size as [`Pixels`].
#[must_use]
pub fn title_size(tokens: &Tokens) -> Pixels {
    tokens.typography.title.size.into()
}

/// Returns the `heading` text size as [`Pixels`].
#[must_use]
pub fn heading_size(tokens: &Tokens) -> Pixels {
    tokens.typography.heading.size.into()
}

/// Returns the `display` text size as [`Pixels`].
#[must_use]
pub fn display_size(tokens: &Tokens) -> Pixels {
    tokens.typography.display.size.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::Tokens;

    #[test]
    fn sizes_are_positive_and_monotonic() {
        let t = Tokens::light();
        let sizes: [f32; 6] = [
            body_small_size(&t).0,
            label_size(&t).0,
            body_size(&t).0,
            title_size(&t).0,
            heading_size(&t).0,
            display_size(&t).0,
        ];
        assert!(sizes.iter().all(|s| s.is_finite() && *s > 0.0));
        // body_small <= label <= body <= title <= heading <= display
        assert!(sizes.windows(2).all(|w| w[0] <= w[1]));
    }
}
