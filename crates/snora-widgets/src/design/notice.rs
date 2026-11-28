//! Notice primitive for Snora Design (RFC-032).
//!
//! A notice is a behavior-light status banner. It communicates information,
//! success, warning, danger, or info to the user and may carry an optional
//! action button and dismiss button. The application owns all state —
//! whether the notice is visible, dismissed, or acted upon.
//!
//! # Semantic construction (RFC-027)
//!
//! - Action and dismiss controls are **focusable**: they are `control`s
//!   (RFC-116), not iced buttons, and iced 0.14's `button` registers no
//!   focus target. Tab reaches them once the application routes the key
//!   with `snora::keyboard::traverse`; Enter and Space press the focused
//!   one and publish the message a click publishes; a ring shows which has
//!   focus. A notice with no action or no dismiss has no control for it,
//!   so there is nothing to Tab to.
//! - **Where focus goes when the notice is dismissed (RFC-120):** if the dismiss
//!   control had focus and the application removes the notice, focus moves to the
//!   nearest control after the notice in the container it sat in, or else the
//!   nearest before it. The notice's own action control goes with it.
//!
//!   If your container is left with no control in it, snora cannot tell that it is
//!   still there, and treats it as gone: focus moves to the nearest control in the
//!   container around it.
//! - **A status tone carries a mark, by shape** (RFC-143, WCAG 1.4.1): at the start edge, after the
//!   accent bar, a small mark that is a different shape for each of `Info`, `Success`, `Warning` and
//!   `Danger` (`Danger` takes the error mark). `Neutral` and `Accent` say no status, so they have none.
//!   It is drawn by snora in the notice's text colour on its surface, needs no font, and takes no Tab
//!   stop and no press. **On by default.** [`Notice::mark`](crate::design::notice::Notice::mark)`(false)`
//!   is for an application that already supplies its own non-colour channel, and only for that.
//! - Tone colors all pass WCAG AA contrast (verified in `snora-design` tests).
//! - The focus ring is drawn by the control, from `Tokens::focus`, in
//!   whichever of that colour or black and white clears 3:1 against the
//!   surface the notice sits on.
//! - The dismiss button shows `"×"` in every build, the same glyph as the
//!   removable chip (RFC-100, RFC-139).
//!   [`Notice::dismiss_tooltip`](crate::design::notice::Notice::dismiss_tooltip)
//!   adds a short hover text such as `"Dismiss"`. **It is a visual tooltip
//!   only and is not exposed to assistive technology:** iced 0.14 has no
//!   accessible-name API for buttons, and snora has no accessibility tree.
//!   Do not rely on it as the dismiss control's accessible name.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_core::LayoutDirection;
//! use snora_design::{Tokens, Tone};
//! use snora_widgets::design::notice::Notice;
//!
//! #[derive(Clone)]
//! enum Message { RebuildIndex, DismissNotice }
//!
//! let tokens = Tokens::light();
//!
//! let _notice: iced::Element<'_, Message> =
//!     Notice::new(&tokens, Tone::Warning, "Index is out of date.", LayoutDirection::Ltr)
//!         .title("Stale index")
//!         .action("Rebuild", Message::RebuildIndex)
//!         .dismiss(Message::DismissNotice)
//!         .render();
//! ```

use iced::widget::text::Wrapping;
use iced::{
    Border, Color, Element, Length, Theme,
    widget::{column, container, text},
};
use snora_core::LayoutDirection;
use snora_design::{Tokens, Tone};
use snora_interaction::{StatusKind, StatusMark};

use snora_style as style;

use super::chip::{close_glyph, with_close_tooltip};
use crate::direction::{LogicalRow, start_aligned};
use crate::style::{BUTTON_DEFAULT_PADDING, design_control};

// ---------------------------------------------------------------------------
// Internal action type
// ---------------------------------------------------------------------------

struct NoticeAction<Message> {
    label: String,
    on_press: Message,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for a toned notice banner.
pub struct Notice<'a, Message> {
    tokens: &'a Tokens,
    tone: Tone,
    direction: LayoutDirection,
    title: Option<String>,
    body: String,
    action: Option<NoticeAction<Message>>,
    dismiss: Option<Message>,
    dismiss_tooltip: Option<String>,
    mark: bool,
}

impl<'a, Message: Clone + 'a> Notice<'a, Message> {
    /// Creates a notice with the given tone and body message.
    ///
    /// **`direction`** (RFC-133): the accent bar, the title and body, and the action and dismiss
    /// controls are laid out from the start edge, in that order, and the text is aligned to it.
    /// Under [`LayoutDirection::Rtl`] the bar is on the right and the controls on the left.
    #[must_use]
    pub fn new(
        tokens: &'a Tokens,
        tone: Tone,
        body: impl Into<String>,
        direction: LayoutDirection,
    ) -> Self {
        Self {
            tokens,
            tone,
            direction,
            title: None,
            body: body.into(),
            action: None,
            dismiss: None,
            dismiss_tooltip: None,
            mark: true,
        }
    }

    /// Adds an optional title line above the body.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds an action: a ghost control, with its label in the accent colour **whatever the
    /// notice's tone** (RFC-128). The tone shows in the bar and the border, which are not text;
    /// the label is, and the accent is a declared text pair in every preset.
    #[must_use]
    pub fn action(mut self, label: impl Into<String>, on_press: Message) -> Self {
        self.action = Some(NoticeAction {
            label: label.into(),
            on_press,
        });
        self
    }

    /// Adds a dismiss button: `"×"` in every build (RFC-139).
    #[must_use]
    pub fn dismiss(mut self, on_press: Message) -> Self {
        self.dismiss = Some(on_press);
        self
    }

    /// Shows `tooltip` above the dismiss button while the pointer hovers it
    /// — for example `"Dismiss"`. Has no effect without [`Notice::dismiss`].
    ///
    /// **This is a visual tooltip, not an accessible name.** It is not
    /// exposed to assistive technology: iced 0.14 has no accessible-name API
    /// for buttons, and snora has no accessibility tree.
    #[must_use]
    pub fn dismiss_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.dismiss_tooltip = Some(tooltip.into());
        self
    }

    /// Whether a status tone shows its **mark** (RFC-143). **On by default.**
    ///
    /// `Info`, `Success`, `Warning` and `Danger` say what kind of message this is by the colour of
    /// the bar and the border, and colour alone fails a reader who cannot tell the colours apart
    /// (WCAG 1.4.1, Level A). So each carries a small mark, a different shape for each tone.
    ///
    /// **This switch is for one case only:** an application that already supplies its own
    /// non-colour channel for the tone (its own icon, say), so that it does not show two. Turning
    /// the mark off for any other reason takes away the one thing that tells these readers what
    /// kind of message it is. It has no effect on `Neutral` and `Accent`, which have no mark.
    #[must_use]
    pub fn mark(mut self, mark: bool) -> Self {
        self.mark = mark;
        self
    }

    /// Renders the notice into an [`Element`].
    #[must_use]
    pub fn render(self) -> Element<'a, Message> {
        let t = self.tokens;
        let direction = self.direction;
        // The text sits at the start edge (RFC-133 R-4): as wide as its cell, aligned to the side
        // the start edge is on.
        let align = start_aligned(direction);

        let accent_color = notice_accent_color(t, self.tone);
        let text_color = style::color::to_iced_color(t.palette.text_primary);
        let surface = style::color::to_iced_color(t.palette.surface);

        // Content: optional title + body
        // `WordOrGlyph` (RFC-105): a notice is as wide as its parent
        // gives it, and caller text with no spaces — a file name, a path,
        // a URL, a hash — used to be cut off at that width with no sign
        // that anything was missing. Measured in a 340px parent: 400
        // unbroken characters rendered 312.0 wide against a natural
        // 3046.4, one line. iced's default `Word` breaks at spaces only,
        // so it does nothing for a string that has none; ordinary prose
        // is unaffected either way.
        let mut content_col: Vec<Element<'a, Message>> = Vec::new();
        if let Some(title) = self.title {
            content_col.push(
                text(title)
                    .size(style::text::label_size(t))
                    .color(text_color)
                    .width(Length::Fill)
                    .align_x(align)
                    .wrapping(Wrapping::WordOrGlyph)
                    .into(),
            );
        }
        content_col.push(
            text(self.body)
                .size(style::text::body_size(t))
                .color(text_color)
                .width(Length::Fill)
                .align_x(align)
                .wrapping(Wrapping::WordOrGlyph)
                .into(),
        );

        // Optional action button (ghost style — low emphasis inside notice)
        let mut controls: Vec<Element<'a, Message>> = Vec::new();
        // Dismissing the notice takes its action control with it, which comes
        // first in Tab order (RFC-120).
        let action_goes = usize::from(self.action.is_some());
        if let Some(act) = self.action {
            let tok = t.clone();
            controls.push(
                design_control(
                    t,
                    // No colour of its own (RFC-128): the ghost style draws the label in
                    // `palette.accent`, a declared text pair. The tone shows in the bar and
                    // the border, which are not text.
                    text(act.label).size(style::text::label_size(t)),
                    move |_theme: &Theme, status| style::button::ghost(&tok, status),
                    t.palette.surface,
                )
                .padding(BUTTON_DEFAULT_PADDING)
                .on_press(act.on_press)
                .into(),
            );
        }

        // Optional dismiss button
        if let Some(on_dismiss) = self.dismiss {
            let tok = t.clone();
            let dismiss_btn = design_control(
                t,
                close_glyph(t, Some(text_color)),
                move |_theme: &Theme, status| style::button::ghost(&tok, status),
                t.palette.surface,
            )
            .padding(BUTTON_DEFAULT_PADDING)
            .removes(action_goes, 0)
            .on_press(on_dismiss)
            .into();
            controls.push(with_close_tooltip(t, dismiss_btn, self.dismiss_tooltip));
        }

        // Main row: the accent bar at the start edge, then the content, then the controls: a logical
        // row, so under RTL the bar is on the right and the controls on the left.
        let content_area: Element<'a, Message> = column(content_col).spacing(t.spacing.xs).into();
        // The tone's mark, at the start edge of the content (RFC-143), centred on it. A logical row, so under
        // RTL it is on the right with the text. It is 1 body line of the text it sits with.
        let content_area: Element<'a, Message> = match tone_mark(self.tone).filter(|_| self.mark) {
            Some(kind) => {
                let mark = StatusMark::new(kind, move |_theme: &Theme| (text_color, surface))
                    .pixels(style::text::body_size(t).0);
                LogicalRow::new(direction)
                    .spacing(t.spacing.sm)
                    .align_y(iced::Alignment::Center)
                    .push(mark)
                    .push(container(content_area).width(Length::Fill))
                    .into()
            }
            None => content_area,
        };

        let mut main_row = LogicalRow::new(direction);

        main_row = main_row.push(
            container(iced::widget::space())
                .width(4.0)
                .height(Length::Fill)
                .style(move |_| notice_accent_bar_style(accent_color)),
        );

        main_row = main_row.push(
            container(content_area)
                .padding([t.spacing.sm, t.spacing.md])
                .width(Length::Fill),
        );

        if !controls.is_empty() {
            let mut control_row = LogicalRow::new(direction)
                .spacing(t.spacing.xs)
                .align_y(iced::Alignment::Center);
            for control in controls {
                control_row = control_row.push(control);
            }
            main_row = main_row.push(
                container(control_row)
                    .padding([t.spacing.sm, t.spacing.sm])
                    .align_y(iced::alignment::Vertical::Center),
            );
        }

        // `Shrink` is set **after** the children are pushed: a row takes the `Fill` of a child it
        // is given (the accent bar is `Fill` tall), so setting it first leaves the row filling the
        // height it is offered, and the notice grows with its parent.
        let main_row = main_row.height(Length::Shrink);

        let radius = t.radius.md;

        container(main_row)
            .style(move |_| notice_outer_style(surface, accent_color, radius))
            .into()
    }
}

/// The mark each tone takes (RFC-143): the matching intent's, with `Danger` taking `Error`'s, and none for the two
/// tones that say no status. Exhaustive on purpose (RFC-063's pattern): a seventh [`Tone`] fails to compile here
/// until it is given a mark, or none.
pub(super) fn tone_mark(tone: Tone) -> Option<StatusKind> {
    match tone {
        Tone::Info => Some(StatusKind::Info),
        Tone::Success => Some(StatusKind::Success),
        Tone::Warning => Some(StatusKind::Warning),
        Tone::Danger => Some(StatusKind::Error),
        Tone::Neutral | Tone::Accent => None,
    }
}

/// Maps a [`Tone`] to the single colour that carries it: the left accent
/// bar and the outer border reuse this same value. It is **not a text
/// colour** and the action's label does not use it (RFC-128). **Colour only** — RFC-093's register for this surface;
/// see `channel_register.rs`'s test asserting nothing else varies by
/// tone in [`notice_accent_bar_style`] or [`notice_outer_style`].
///
/// Exhaustive on purpose (RFC-063's pattern): a seventh [`Tone`] fails
/// to compile here until it is given a colour.
pub(super) fn notice_accent_color(tokens: &Tokens, tone: Tone) -> Color {
    let p = &tokens.palette;
    style::color::to_iced_color(match tone {
        Tone::Accent => p.accent,
        Tone::Success => p.success,
        Tone::Warning => p.warning,
        Tone::Danger => p.danger,
        Tone::Info => p.info,
        Tone::Neutral => p.border,
    })
}

/// Style of the left accent bar. Only `background` varies with `color`
/// (which callers derive from [`notice_accent_color`]); everything else
/// is a fixed literal, checked by `channel_register.rs`.
pub(super) fn notice_accent_bar_style(color: Color) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(color.into()),
        border: Border::default().rounded(0.0),
        ..Default::default()
    }
}

/// Style of the notice's outer container. Only `border.color` varies
/// with the caller-supplied accent colour; `background`, `border.width`,
/// and `border.radius` are independent of tone (checked by
/// `channel_register.rs`), though `radius` does vary with the token
/// bundle's own `radius.md`, unrelated to tone.
pub(super) fn notice_outer_style(
    surface: Color,
    border_color: Color,
    radius: f32,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(surface.into()),
        border: Border::default()
            .rounded(radius)
            .color(border_color)
            .width(1.0),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests;
