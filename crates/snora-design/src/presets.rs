//! Built-in token presets. Reached via `Tokens::{light,dark,...}`.

pub(crate) mod dark;
pub(crate) mod high_contrast_dark;
pub(crate) mod high_contrast_light;
pub(crate) mod light;

use crate::{Density, FocusTokens, Palette, Radius, Spacing, Tokens, Typography};

/// Shared assembly: combine a palette and a focus-ring width with the default
/// spacing/typography/radius scales and comfortable density.
pub(crate) fn assemble(palette: Palette, ring_width: f32) -> Tokens {
    let focus = FocusTokens::new(ring_width, 2.0, palette.focus);
    Tokens {
        palette,
        spacing: Spacing::comfortable(),
        typography: Typography::default_roles(),
        radius: Radius::default_roles(),
        focus,
        density: Density::Comfortable,
    }
}
