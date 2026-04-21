use iced::{Element, widget::center};
use snora_core::contract::stack::Dialog;

pub fn render_dialog<'a, Message>(
    dialog: Option<Dialog<Element<'a, Message>, Message>>,
) -> Option<Element<'a, Message>>
where
    Message: 'a + Clone,
{
    let dialog = if let Some(x) = dialog {
        x
    } else {
        return None;
    };

    let dialog_content = center(dialog.content);

    Some(dialog_content.into())
}
