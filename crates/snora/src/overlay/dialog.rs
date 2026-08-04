//! Dialog — the centered modal card.

use iced::widget::container;
use iced::{Element, widget::center};
use snora_core::Dialog;

/// Style for the dialog's card wrapper (RFC-039). `None` (the default,
/// unstyled path) centers `dialog.content` with no card at all — this is
/// [`crate::render::render`]'s exact pre-RFC-039 behavior, preserved
/// unchanged. `Some` (the `design`-gated path, see
/// [`crate::design::render::render`]) wraps the content in a token-styled
/// container before centering it.
pub(crate) struct DialogCardStyle {
    /// Uniform padding between the card's edge and `dialog.content`.
    pub(crate) padding: f32,
    /// The card's fill, border, and radius. Deliberately no shadow — see
    /// [`crate::design::render`]'s module documentation for why the card
    /// is border-defined, not shadow-defined.
    pub(crate) style: container::Style,
}

/// Center the dialog content in the window, optionally wrapped in a
/// styled card. The surrounding dim layer is owned by
/// [`crate::render::render_with_style`].
pub(crate) fn render_dialog<'a, Message>(
    dialog: Dialog<Element<'a, Message>, Message>,
    card: Option<&DialogCardStyle>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    match card {
        None => center(dialog.content)
            .id(crate::identifiers::DIALOG_CARD)
            .into(),
        Some(card) => {
            let padding = card.padding;
            let style = card.style;
            center(
                container(dialog.content)
                    .padding(padding)
                    .style(move |_theme| style),
            )
            .id(crate::identifiers::DIALOG_CARD)
            .into()
        }
    }
}
