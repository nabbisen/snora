//! Notice primitive for Snora Design (RFC-032).
//!
//! A notice is a behavior-light status banner. It communicates information,
//! success, warning, danger, or info to the user and may carry an optional
//! action button and dismiss button. The application owns all state —
//! whether the notice is visible, dismissed, or acted upon.
//!
//! # Semantic construction (RFC-027)
//!
//! - Action and dismiss controls are `iced::widget::button` — keyboard
//!   reachable and semantically meaningful.
//! - Tone colors all pass WCAG AA contrast (verified in `snora-design` tests).
//! - Focus rings follow the iced 0.14 limitation (no `Focused` status on
//!   `button::Status`); documented, not a regression.
//! - The dismiss button uses `"×"` as its visible label. iced 0.14 does not
//!   expose a separate accessible label for buttons. If a more descriptive
//!   label is needed for assistive technology, this is a future customization
//!   point (e.g. `.dismiss_label(msg, label)`).
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::{Tokens, Tone};
//! use snora_widgets::design::notice::Notice;
//!
//! let tokens = Tokens::light();
//!
//! Notice::new(&tokens, Tone::Warning, "Index is out of date.")
//!     .title("Stale index")
//!     .action("Rebuild", Message::RebuildIndex)
//!     .dismiss(Message::DismissNotice)
//!     .render()
//! ```

use iced::{
    Border, Element, Length,
    widget::{button, column, container, row, text},
};
use snora_design::{Tone, Tokens};

use super::style;

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
    title: Option<String>,
    body: String,
    action: Option<NoticeAction<Message>>,
    dismiss: Option<Message>,
}

impl<'a, Message: Clone + 'a> Notice<'a, Message> {
    /// Creates a notice with the given tone and body message.
    #[must_use]
    pub fn new(tokens: &'a Tokens, tone: Tone, body: impl Into<String>) -> Self {
        Self {
            tokens,
            tone,
            title: None,
            body: body.into(),
            action: None,
            dismiss: None,
        }
    }

    /// Adds an optional title line above the body.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds a primary action button.
    #[must_use]
    pub fn action(mut self, label: impl Into<String>, on_press: Message) -> Self {
        self.action = Some(NoticeAction { label: label.into(), on_press });
        self
    }

    /// Adds a dismiss (×) button.
    #[must_use]
    pub fn dismiss(mut self, on_press: Message) -> Self {
        self.dismiss = Some(on_press);
        self
    }

    /// Renders the notice into an [`Element`].
    #[must_use]
    pub fn render(self) -> Element<'a, Message> {
        let t = self.tokens;
        let p = &t.palette;

        let accent_color = style::color::to_iced_color(match self.tone {
            Tone::Accent  => p.accent,
            Tone::Success => p.success,
            Tone::Warning => p.warning,
            Tone::Danger  => p.danger,
            Tone::Info    => p.info,
            Tone::Neutral => p.border,
        });
        let text_color = style::color::to_iced_color(p.text_primary);
        let surface    = style::color::to_iced_color(p.surface);

        // Content: optional title + body
        let mut content_col: Vec<Element<'a, Message>> = Vec::new();
        if let Some(title) = self.title {
            content_col.push(
                text(title)
                    .size(style::text::label_size(t))
                    .color(text_color)
                    .into(),
            );
        }
        content_col.push(
            text(self.body)
                .size(style::text::body_size(t))
                .color(text_color)
                .into(),
        );

        // Optional action button (ghost style — low emphasis inside notice)
        let mut controls: Vec<Element<'a, Message>> = Vec::new();
        if let Some(act) = self.action {
            let tok = t.clone();
            controls.push(
                button(
                    text(act.label)
                        .size(style::text::label_size(t))
                        .color(accent_color),
                )
                .on_press(act.on_press)
                .style(move |_theme, status| style::button::ghost(&tok, status))
                .into(),
            );
        }

        // Optional dismiss button
        if let Some(on_dismiss) = self.dismiss {
            let tok = t.clone();
            controls.push(
                button(text("×").size(style::text::label_size(t)).color(text_color))
                    .on_press(on_dismiss)
                    .style(move |_theme, status| style::button::ghost(&tok, status))
                    .into(),
            );
        }

        // Main row: colored left accent bar + content + controls
        let content_area: Element<'a, Message> =
            column(content_col).spacing(t.spacing.xs).into();

        let mut main_row_children: Vec<Element<'a, Message>> = Vec::new();

        // Left accent bar
        let bar_color = accent_color;
        main_row_children.push(
            container(iced::widget::space())
                .width(4.0)
                .height(Length::Fill)
                .style(move |_| iced::widget::container::Style {
                    background: Some(bar_color.into()),
                    border: Border::default().rounded(0.0),
                    ..Default::default()
                })
                .into(),
        );

        main_row_children.push(
            container(content_area)
                .padding([t.spacing.sm, t.spacing.md])
                .width(Length::Fill)
                .into(),
        );

        if !controls.is_empty() {
            main_row_children.push(
                container(
                    row(controls).spacing(t.spacing.xs).align_y(iced::Alignment::Center),
                )
                .padding([t.spacing.sm, t.spacing.sm])
                .align_y(iced::alignment::Vertical::Center)
                .into(),
            );
        }

        let radius = t.radius.md;
        let border_color = accent_color;

        container(row(main_row_children).height(Length::Shrink))
            .style(move |_| iced::widget::container::Style {
                background: Some(surface.into()),
                border: Border::default()
                    .rounded(radius)
                    .color(border_color)
                    .width(1.0),
                ..Default::default()
            })
            .into()
    }
}

#[cfg(test)]
mod tests;
