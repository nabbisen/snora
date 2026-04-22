//! Dialog — the centered modal card.

use iced::{Element, widget::center};
use snora_core::Dialog;

/// Center the dialog content in the window. The surrounding dim layer is
/// owned by [`crate::render::render`].
pub(crate) fn render_dialog<'a, Message>(
    dialog: Dialog<Element<'a, Message>, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    center(dialog.content).into()
}
