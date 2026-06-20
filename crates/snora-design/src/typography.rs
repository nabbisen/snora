//! Text roles and the [`Typography`] scale.

/// A single text role: a font size paired with a line-height multiplier.
///
/// `line_height` is a **renderer-independent multiplier** (e.g. `1.4` means
/// 140% of the font size). Conversion to the pinned iced version's
/// line-height configuration happens in `snora-widgets`, not here.
///
/// ```
/// use snora_design::TextRole;
/// let body = TextRole { size: 16.0, line_height: 1.4 };
/// assert!(body.line_height > 1.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRole {
    /// Font size in logical pixels.
    pub size: f32,
    /// Line-height multiplier (e.g. `1.4`), relative to `size`.
    pub line_height: f32,
}

/// The text-role scale.
///
/// Roles by purpose: `body` ordinary explanatory text; `body_small` compact
/// help/metadata; `label` button/chip/control labels; `title`
/// card/dialog/notice titles; `heading` page/section headings; `display` rare
/// major page titles. Longer text surfaces (notices, empty states, guidance)
/// should use roles with readable line-height.
///
/// No font family is selected in v0.20 — that remains application-owned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typography {
    /// Ordinary explanatory text.
    pub body: TextRole,
    /// Secondary metadata / compact help.
    pub body_small: TextRole,
    /// Button, field, and chip labels.
    pub label: TextRole,
    /// Card / dialog / notice title.
    pub title: TextRole,
    /// Page or section heading.
    pub heading: TextRole,
    /// Rare major page title.
    pub display: TextRole,
}

impl Typography {
    /// The default text-role scale shared by all v0.20 presets.
    #[must_use]
    pub const fn default_roles() -> Self {
        Self {
            body: TextRole {
                size: 16.0,
                line_height: 1.4,
            },
            body_small: TextRole {
                size: 14.0,
                line_height: 1.35,
            },
            label: TextRole {
                size: 14.0,
                line_height: 1.2,
            },
            title: TextRole {
                size: 18.0,
                line_height: 1.3,
            },
            heading: TextRole {
                size: 24.0,
                line_height: 1.25,
            },
            display: TextRole {
                size: 32.0,
                line_height: 1.2,
            },
        }
    }
}

#[cfg(test)]
mod tests;
