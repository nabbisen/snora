//! Chip primitives for Snora Design (RFC-032).
//!
//! Two variants:
//!
//! * `filter` — a toggle chip for filtering or categorizing. Solid accent
//!   background + `accent_text` foreground when selected; neutral border at
//!   rest.
//! * `removable` — a chip with a separate remove (×) button.
//!
//! Both are backed by `iced::widget::button` and are keyboard-reachable.
//! The application owns selection/filter state.
//!
//! # Contrast design (M-4 fix)
//!
//! Prior to v0.24, the selected chip used a semi-transparent accent tint
//! (alpha 0.15–0.30) as the background with the full accent color as text.
//! After compositing over the surface, hovered/pressed states failed
//! WCAG AA (4.5:1). Replaced with a solid `accent` background +
//! `accent_text` foreground, which yields ≥6.7:1 across all four presets.
//! The contrast test `chip_selected_text_over_accent_background` verifies
//! this.
//!
//! # iced 0.14 focus limitation
//!
//! No custom focus ring — `button::Status` has no `Focused` variant.
//! Documented limitation, not a regression (RFC-025, RFC-027).
//!
//! # The remove glyph, and what a tooltip is not
//!
//! The remove button shows lucide `X` when the `lucide-icons` feature is
//! enabled, and the text glyph `"×"` otherwise (RFC-100). The notice's
//! dismiss button shares the same glyph.
//!
//! [`removable_with_tooltip`](crate::design::chip::removable_with_tooltip)
//! adds a short text shown when the pointer hovers the remove button, such
//! as `"Remove tag"`. **It is a visual tooltip only. It is not exposed to
//! assistive technology:** iced 0.14 has no accessible-name API for buttons,
//! and snora has no accessibility tree. Do not rely on it as the remove
//! control's accessible name.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_widgets::design::chip;
//!
//! #[derive(Clone)]
//! enum Message { ToggleDrafts, ToggleTag, RemoveTag }
//!
//! let tokens = Tokens::light();
//! let show_drafts = true; // stands in for e.g. `self.show_drafts`
//!
//! let _draft_chip: iced::Element<'_, Message> =
//!     chip::filter(&tokens, "Draft", show_drafts, Message::ToggleDrafts);
//! let _tag_chip: iced::Element<'_, Message> = chip::removable(
//!     &tokens,
//!     "Tag: Rust",
//!     true,
//!     Message::ToggleTag,
//!     Message::RemoveTag,
//! );
//! ```

use iced::{
    Border, Color, Element,
    widget::{button, container, row, text, tooltip},
};
use snora_design::Tokens;

use snora_style as style;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Blends a color toward black by `amount`. Used for hover/press states.
fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}

/// The `remove` button's square pointer-target size (RFC-061): computed
/// the same way its height already resolves — `line_box + 2 × spacing.xs`
/// — so the width fix tracks any future token change automatically
/// rather than hard-coding a second "24".
fn remove_btn_target_size(tokens: &Tokens) -> f32 {
    tokens.typography.label.size * tokens.typography.label.line_height + 2.0 * tokens.spacing.xs
}

/// The close glyph shared by the removable chip's remove control and the
/// notice's dismiss control (RFC-100), sized to the label text.
///
/// One function, so the two controls cannot drift apart and the
/// `lucide-icons` `cfg` lives in one place. Lucide `X` under that feature;
/// text `"×"` otherwise, exactly as before.
///
/// `color` of `None` inherits the enclosing button's text colour. Under
/// `lucide-icons` the glyph comes from [`crate::icon::icon_element_sized`],
/// which returns an [`Element`] that cannot be recoloured afterwards, so the
/// colour is applied through a container's default text colour instead —
/// otherwise the notice's glyph would silently take its ghost button's
/// `accent` colour in place of `text_primary`.
pub(super) fn close_glyph<'a, Message: 'a>(
    tokens: &Tokens,
    color: Option<Color>,
) -> Element<'a, Message> {
    let size = style::text::label_size(tokens);

    #[cfg(feature = "lucide-icons")]
    {
        let glyph = crate::icon::icon_element_sized(
            &snora_core::Icon::Lucide(lucide_icons::Icon::X),
            size.0,
        );
        container(glyph)
            .style(move |_| container::Style {
                text_color: color,
                ..container::Style::default()
            })
            .into()
    }

    #[cfg(not(feature = "lucide-icons"))]
    {
        let glyph = text("×").size(size);
        match color {
            Some(color) => glyph.color(color).into(),
            None => glyph.into(),
        }
    }
}

/// Wraps a close control in a visual tooltip, or returns it unchanged when
/// there is none (RFC-100).
///
/// **Above the control** (`Position::Top`). Neither primitive knows the
/// layout direction, so `Left`/`Right` would be wrong under one direction or
/// the other. Of the two vertical positions, `Top` is iced's default, and it
/// keeps the text clear of the mouse pointer, whose arrow extends downward
/// from the hotspot and would cover part of a tooltip placed below.
///
/// The body is [`style::container::card_raised`], the popover style: its
/// `text_primary` on `surface_raised` is an existing, contrast-tested
/// palette pairing.
///
/// **Not an accessible name.** iced 0.14 has no accessible-name API for
/// buttons and snora has no accessibility tree; assistive technology does
/// not see this text.
pub(super) fn with_close_tooltip<'a, Message: 'a>(
    tokens: &Tokens,
    control: Element<'a, Message>,
    tip: Option<String>,
) -> Element<'a, Message> {
    let Some(tip) = tip else {
        return control;
    };
    let t = tokens.clone();
    tooltip(
        control,
        container(text(tip).size(style::text::label_size(tokens)))
            .padding([tokens.spacing.xs, tokens.spacing.sm])
            .style(move |_| style::container::card_raised(&t)),
        tooltip::Position::Top,
    )
    .gap(tokens.spacing.xs)
    .into()
}

/// Selected chip style: solid accent background + accent_text foreground.
///
/// This replaces the previous semi-transparent tint approach, which failed
/// WCAG AA (4.5:1) at hovered (α=0.22) and pressed (α=0.30) states after
/// compositing over the surface. Solid background + paired foreground role
/// yields ≥6.7:1 across all four built-in presets.
fn chip_style_selected(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = style::color::to_iced_color(tokens.palette.accent);
    let accent_text = style::color::to_iced_color(tokens.palette.accent_text);
    let bg = match status {
        button::Status::Active => accent,
        button::Status::Hovered => darken(accent, 0.06),
        button::Status::Pressed => darken(accent, 0.12),
        button::Status::Disabled => Color { a: 0.5, ..accent },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: accent_text,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(accent)
            .width(1.0),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

fn chip_style_unselected(tokens: &Tokens, status: button::Status) -> button::Style {
    let border_col = style::color::to_iced_color(tokens.palette.border);
    let text_col = style::color::to_iced_color(tokens.palette.text_secondary);
    let surface = style::color::to_iced_color(tokens.palette.surface);
    let bg = match status {
        button::Status::Active => surface,
        button::Status::Hovered => darken(surface, 0.04),
        button::Status::Pressed => darken(surface, 0.08),
        button::Status::Disabled => Color { a: 0.5, ..surface },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: text_col,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(border_col)
            .width(1.0),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// A toggle chip for filtering or categorizing content.
///
/// Shows a solid accent background and `accent_text` foreground when
/// `selected` (WCAG AA ≥6.7:1 across all built-in presets). Emits
/// `on_toggle` when pressed. Pass `None` to disable.
#[must_use]
pub fn filter<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    let t = tokens.clone();
    let style_fn = if selected {
        chip_style_selected
    } else {
        chip_style_unselected
    };
    button(text(label.into()).size(style::text::label_size(tokens)))
        .on_press_maybe(on_toggle.into())
        .padding([tokens.spacing.xs, tokens.spacing.sm])
        .style(move |_theme, status| style_fn(&t, status))
        .into()
}

/// A chip with a separate remove button.
///
/// The chip label toggles via `on_toggle`; the remove button emits
/// `on_remove`. Both controls are `iced::widget::button` and are
/// keyboard-reachable. The remove button shows lucide `X` under the
/// `lucide-icons` feature and `"×"` otherwise.
///
/// To show a tooltip on the remove button, use [`removable_with_tooltip`].
#[must_use]
pub fn removable<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    build_removable(tokens, label, selected, on_toggle, on_remove, None)
}

/// [`removable`], with a tooltip on the remove button.
///
/// `tooltip` is shown above the remove button while the pointer hovers it
/// — for example `"Remove tag"`.
///
/// **This is a visual tooltip, not an accessible name.** It is not exposed
/// to assistive technology: iced 0.14 has no accessible-name API for
/// buttons, and snora has no accessibility tree.
#[must_use]
pub fn removable_with_tooltip<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
    tooltip: impl Into<String>,
) -> Element<'a, Message> {
    build_removable(
        tokens,
        label,
        selected,
        on_toggle,
        on_remove,
        Some(tooltip.into()),
    )
}

/// The one implementation behind [`removable`] and
/// [`removable_with_tooltip`].
fn build_removable<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
    tip: Option<String>,
) -> Element<'a, Message> {
    let t_label = tokens.clone();
    let t_remove = tokens.clone();
    let style_fn = if selected {
        chip_style_selected
    } else {
        chip_style_unselected
    };

    let label_btn: Element<'a, Message> =
        button(text(label.into()).size(style::text::label_size(tokens)))
            .on_press_maybe(on_toggle.into())
            .padding([tokens.spacing.xs, tokens.spacing.sm])
            .style(move |_theme, status| style_fn(&t_label, status))
            .into();

    // Pointer-target size (RFC-061): the glyph's own advance width is
    // not token-derivable (font/shaping-dependent — measured at 15.0px
    // total for the shipped fallback font at the current tokens, well
    // under the 24px WCAG 2.5.8 floor; lucide `X` is 1em wide, also short).
    // Padding alone cannot fix this
    // reliably: even bumping to `spacing.sm` only reaches 23.0px on that
    // same font, still short. Instead, the *content* box inside the
    // button is forced to a computed width and its text centered within
    // it — `iced::widget::button`'s own layout does not re-center a
    // child when the button is simply widened (see `layout::padded`),
    // so the centering has to happen one level in.
    let target_size = remove_btn_target_size(tokens);
    let content_width = target_size - 2.0 * tokens.spacing.xs;
    let remove_btn: Element<'a, Message> =
        button(container(close_glyph(tokens, None)).center_x(content_width))
            .on_press_maybe(on_remove.into())
            .padding([tokens.spacing.xs, tokens.spacing.xs])
            .style(move |_theme, status| style_fn(&t_remove, status))
            .into();
    let remove_btn = with_close_tooltip(tokens, remove_btn, tip);

    row![label_btn, remove_btn].spacing(0).into()
}

#[cfg(test)]
mod tests;
