//! Shared styling for snora's built-in widgets.
//!
//! These are small, opinionated defaults. Applications that need a
//! distinct visual identity should not try to override these — they
//! should instead skip the built-in widgets in this crate and build
//! their own elements for the relevant
//! [`snora_core::AppLayout`] slots. The framework does **not** gate that
//! path: AppLayout slots are raw `Element`s, so your own widgets compose
//! into the skeleton without touching snora's style surface.

use iced::{
    Border, Color, Shadow, Theme,
    widget::{button, container},
};

/// The widest a tooltip body may be (RFC-105 R-1).
///
/// A tooltip is content-sized, so long caller text has nothing to wrap
/// against but the viewport — measured before this, a 400-character
/// tooltip was cut off inside the overlay, and wrapping alone would have
/// turned it into a window-wide block instead. This bounds it at a
/// readable line, in the scale of the toast's fixed 340.
///
/// A width, not a text size, so `scripts/check-literal-sizes.sh` does not
/// see it; it is a named constant anyway, because one shared value for
/// every tooltip in the crate is the point.
pub(crate) const TOOLTIP_MAX_WIDTH: f32 = 320.0;

/// Neutral text-only button used for menu entries.
///
/// **Corrected (RFC-085 F-13).** Every status previously used a shade
/// from `ep.primary` — a **background-tier** family — as `text_color`,
/// on `background: None`, so it painted over the dropdown surface's own
/// background. No status reached WCAG AA (measured: 1.89:1 light /
/// 2.20:1 dark at rest, 3.73:1 / 3.70:1 hovered). `background.base.text`
/// is the tier iced itself guarantees is readable against
/// `background.base.color`, which is what this button actually paints
/// over — used uniformly across all statuses, since the background never
/// changes. This trades away the old (illegible) hover/press color
/// differentiation; a future change may reintroduce it via a background
/// highlight rather than a foreground shade, following the same pattern
/// `crate::sidebar::sidebar_button_style` and
/// `crate::crumb::crumb_button_style` already use for their own hover
/// states — not done here, to keep this fix a re-pairing rather than an
/// added visual element.
pub fn menu_button_style(theme: &Theme, _status: button::Status) -> button::Style {
    let ep = theme.extended_palette();
    button::Style {
        background: None,
        text_color: ep.background.base.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Thin-bordered chrome container used for the app header and footer.
///
/// Radius fixed at `0.0` — today's literal, unchanged. See
/// `chrome_container_style_with_radius` for the `design`-gated styled
/// chrome widgets' (RFC-040) token-derived radius.
pub fn chrome_container_style(theme: &Theme) -> container::Style {
    chrome_container_style_with_radius(theme, 0.0)
}

/// Geometry-parameterized variant of [`chrome_container_style`] (RFC-040):
/// identical color logic, but the corner radius is a parameter instead of
/// a hardcoded `0.0`. [`chrome_container_style`] is a thin wrapper over
/// this with `radius = 0.0` — the one-body-two-geometry-sources shape
/// applied to the shared style function itself, not only the widgets that
/// call it.
///
/// Not part of the public style surface `chrome_container_style` is;
/// `pub(crate)` since only the `design::widget` styled variants need the
/// radius parameter.
pub(crate) fn chrome_container_style_with_radius(theme: &Theme, radius: f32) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        text_color: Some(ep.background.base.text),
        background: None,
        border: Border {
            radius: radius.into(),
            width: 1.0,
            // Corrected (RFC-085 F-15): `background.weak.color` measured
            // 1.02–1.48:1 against the page background in every preset and
            // both stock themes, well under the 3.0:1 non-text floor.
            // `background.strong` was tried next and measured better but
            // still short (1.54–1.58:1) — iced's `strong` tier is tuned
            // for a subtle surface fill, not a guaranteed-visible border.
            // `background.base.text` is the only value derivable from
            // `Theme::extended_palette()` alone that iced itself
            // guarantees sufficient contrast against `background.base`,
            // which is what this border is actually drawn over — bolder
            // than a typical hairline border, but reliably visible in
            // every preset and both stock themes.
            color: ep.background.base.text,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Highlight for the currently-active sidebar item. Returns a background
/// color appropriate for the theme.
///
/// **Corrected (RFC-085 F-14).** `primary.weak.color` measured 1.89:1
/// (stock light) / 2.20:1 (stock dark) against the rail's own
/// background — under the 3.0:1 non-text floor, and this was the
/// *only* cue an active item had (also a WCAG 1.4.1 use-of-colour
/// concern, addressed in `crate::sidebar::sidebar_button_style`'s own
/// doc comment). `primary.base` came within 0.01 of the floor on stock
/// Dark (2.99:1) — not a margin to ship on. `primary.strong` clears it
/// with real margin in every preset and both stock themes.
pub fn sidebar_active_color(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.color
}

// ---------------------------------------------------------------------
// The readable-text rule lives in `snora-interaction` (RFC-122 Unit 1)
// ---------------------------------------------------------------------

/// The rule that keeps a preferred colour where it reads and takes the stronger
/// pole where it does not. It used to be a twin of `snora::toast`'s copy; both now
/// call the one in `snora_interaction::contrast`, which sits below them.
pub(crate) use snora_interaction::contrast::readable_text;

// ---------------------------------------------------------------------
// Controls (RFC-116 R-1 Unit 3)
// ---------------------------------------------------------------------

/// What a control needs beyond its look: the minimum-target token and the
/// focus ring's geometry.
///
/// **One value, carried by each widget's geometry struct**, so that the
/// unstyled path and the design path build a control by the same call and
/// differ only in what they hand it. The unstyled path has no tokens, so
/// it has no minimum token (`0` means none: 24 px and 1.5 x text still
/// apply) and the ring's literals, which match `FocusTokens::new(2.0,
/// 2.0, ..)`. The design path reads all four from `Tokens`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ControlGeometry {
    /// `Tokens::targets.min`; `0.0` on the unstyled path.
    pub(crate) min_target: f32,
    /// The ring's stroke width.
    pub(crate) ring_width: f32,
    /// How far outside the control the ring is drawn.
    pub(crate) ring_offset: f32,
    /// The ring colour the design path prefers (`Tokens::focus`), kept
    /// only where it is readable against what is behind the control. The
    /// unstyled path has none and uses the theme's own text colour.
    pub(crate) ring_preferred: Option<Color>,
}

impl ControlGeometry {
    /// The unstyled path: no token, and the ring's literals.
    pub(crate) const fn unstyled() -> Self {
        Self {
            min_target: 0.0,
            ring_width: 2.0,
            ring_offset: 2.0,
            ring_preferred: None,
        }
    }

    /// The design path: every value from the tokens.
    #[cfg(feature = "design")]
    pub(crate) fn from_tokens(tokens: &snora_design::Tokens) -> Self {
        Self {
            min_target: tokens.targets.min,
            ring_width: tokens.focus.ring_width,
            ring_offset: tokens.focus.ring_offset,
            ring_preferred: Some(snora_style::color::to_iced_color(tokens.focus.ring_color)),
        }
    }
}

/// The focus ring for a control that sits on `behind`.
///
/// # Why the caller says what is behind
///
/// The ring has to clear 3:1 against **whatever is behind it**, and
/// `control` cannot sample that. The rule is RFC-114 Unit 3b's: keep the
/// preferred colour where it is readable against `behind`, and fall back
/// to the stronger of black and white where it is not. That fallback is at
/// least 4.58:1 against any opaque fill by construction, so the ring
/// cannot fail on a theme snora has never seen. With no preferred colour
/// the theme's own text colour is the preference.
pub(crate) fn focus_ring<'a>(
    geometry: ControlGeometry,
    behind: impl Fn(&Theme) -> Color + 'a,
) -> snora_interaction::Ring<'a> {
    snora_interaction::Ring {
        color: Box::new(move |theme: &Theme| {
            let preferred = geometry
                .ring_preferred
                .unwrap_or(theme.extended_palette().background.base.text);
            readable_text(behind(theme), preferred)
        }),
        width: geometry.ring_width,
        offset: geometry.ring_offset,
    }
}

/// A control around `content`: `button`'s padding, `style`, the ring, and
/// the minimum-target token — everything but the press handler, which the
/// caller chooses (`on_press`, `on_press_with`, `on_press_maybe`).
///
/// `label_size` is the size of the text the control carries when the
/// caller set one (the design path), so RFC-115's `1.5 x text` is measured
/// against the text actually on it; `None` reads the host's.
pub(crate) fn control<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
    style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
    geometry: ControlGeometry,
    label_size: Option<f32>,
    behind: impl Fn(&Theme) -> Color + 'a,
) -> snora_interaction::Control<'a, Message> {
    let mut control =
        snora_interaction::Control::new(content, Box::new(style), focus_ring(geometry, behind))
            .min_target(geometry.min_target);
    if let Some(size) = label_size {
        control = control.text_size(size);
    }
    control
}

/// What `iced::widget::button` pads its content with when it is not told
/// otherwise (`[5, 10]`). A control replaces a button without moving its
/// content, so the sites that never set a padding set this one.
pub(crate) const BUTTON_DEFAULT_PADDING: iced::Padding = iced::Padding {
    top: 5.0,
    right: 10.0,
    bottom: 5.0,
    left: 10.0,
};

/// The page background, which is what a control with no fill of its own
/// sits on in the unstyled chrome.
pub(crate) fn page_background(theme: &Theme) -> Color {
    theme.extended_palette().background.base.color
}

/// A design-path control: [`control`] with every value from `tokens`.
///
/// The text the control carries is at `Typography::label`, which is what
/// RFC-115's `1.5 x text` is measured against; the minimum-target token and
/// the ring come from `tokens` too. `behind` is the palette role the control
/// sits on, which the ring's colour is checked against. Padding is the
/// caller's.
#[cfg(feature = "design")]
pub(crate) fn design_control<'a, Message>(
    tokens: &snora_design::Tokens,
    content: impl Into<iced::Element<'a, Message>>,
    style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
    behind: snora_design::Color,
) -> snora_interaction::Control<'a, Message> {
    let behind = snora_style::color::to_iced_color(behind);
    control(
        content,
        style,
        ControlGeometry::from_tokens(tokens),
        Some(snora_style::text::label_size(tokens).0),
        move |_| behind,
    )
}

#[cfg(test)]
mod ring_pin {
    //! **The ring's colour, pinned** (RFC-122 review, 3.4). The rule that picks it
    //! (`readable_text` and `stronger_pole`) moved to `snora-interaction`, below
    //! every caller; this table is what each caller drew before it moved, in the
    //! 22 stock themes (the unstyled path) and the 6 presets (the design path), so
    //! the move cannot change a colour.
    use super::*;

    fn rgb(c: Color) -> [u8; 3] {
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        ]
    }

    const PINNED: &[(&str, &str, [u8; 3])] = &[
        ("unstyled", "Light", [0, 0, 0]),
        ("unstyled", "Dark", [230, 230, 230]),
        ("unstyled", "Dracula", [248, 248, 242]),
        ("unstyled", "Nord", [236, 239, 244]),
        ("unstyled", "SolarizedLight", [58, 97, 110]),
        ("unstyled", "SolarizedDark", [157, 180, 183]),
        ("unstyled", "GruvboxLight", [40, 40, 40]),
        ("unstyled", "GruvboxDark", [251, 241, 199]),
        ("unstyled", "CatppuccinLatte", [76, 79, 105]),
        ("unstyled", "CatppuccinFrappe", [198, 208, 245]),
        ("unstyled", "CatppuccinMacchiato", [202, 211, 245]),
        ("unstyled", "CatppuccinMocha", [205, 214, 244]),
        ("unstyled", "TokyoNight", [154, 165, 206]),
        ("unstyled", "TokyoNightStorm", [182, 196, 250]),
        ("unstyled", "TokyoNightLight", [57, 62, 92]),
        ("unstyled", "KanagawaWave", [220, 215, 186]),
        ("unstyled", "KanagawaDragon", [197, 201, 197]),
        ("unstyled", "KanagawaLotus", [84, 84, 100]),
        ("unstyled", "Moonfly", [189, 189, 189]),
        ("unstyled", "Nightfly", [189, 193, 198]),
        ("unstyled", "Oxocarbon", [208, 208, 208]),
        ("unstyled", "Ferra", [254, 205, 178]),
        ("design", "light", [29, 78, 216]),
        ("design", "dark", [127, 177, 255]),
        ("design", "light_aaa", [21, 68, 206]),
        ("design", "dark_aaa", [127, 177, 255]),
        ("design", "high_contrast_light", [0, 48, 176]),
        ("design", "high_contrast_dark", [255, 255, 255]),
    ];

    #[test]
    fn every_ring_colour_is_what_it_was_before_the_rule_moved() {
        let mut off = Vec::new();
        let mut checked = 0;
        for theme in Theme::ALL {
            let ring = focus_ring(ControlGeometry::unstyled(), page_background);
            let got = rgb((ring.color)(theme));
            let name = format!("{theme:?}");
            let want = PINNED
                .iter()
                .find(|(k, n, _)| *k == "unstyled" && *n == name)
                .unwrap_or_else(|| panic!("no pinned colour for {name}"))
                .2;
            checked += 1;
            if got != want {
                off.push(format!("unstyled {name}: {got:?}, was {want:?}"));
            }
        }
        #[cfg(feature = "design")]
        for (name, tokens) in snora_design::Tokens::presets() {
            let theme = snora_style::theme::theme(&tokens);
            let behind = snora_style::color::to_iced_color(tokens.palette.background);
            let ring = focus_ring(ControlGeometry::from_tokens(&tokens), move |_| behind);
            let got = rgb((ring.color)(&theme));
            let want = PINNED
                .iter()
                .find(|(k, n, _)| *k == "design" && *n == name)
                .unwrap_or_else(|| panic!("no pinned colour for {name}"))
                .2;
            checked += 1;
            if got != want {
                off.push(format!("design {name}: {got:?}, was {want:?}"));
            }
        }
        assert!(checked >= 22, "only {checked} colours checked");
        assert!(off.is_empty(), "ring colours changed: {off:?}");
    }
}
