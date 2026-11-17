//! Anchored source for `docs/src/design/recipes/error-recovery.md` (RFC-136).

use iced::widget::column;

/// The application the second fragment is part of.
struct App {
    tokens: snora::design::Tokens,
    sync_error: Option<String>,
}

#[derive(Clone)]
enum Message {
    RetrySyncing,
    DismissSyncError,
}

// ANCHOR: recipe_error_recovery
use iced::Element;
use snora::design::{Tokens, Tone, notice::Notice};

fn error_recovery_notice<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    visible: bool,
    title: &'a str,
    description: &'a str,
    action_label: &'a str,
    on_action: Message,
    on_dismiss: Option<Message>,
    direction: snora::LayoutDirection,
) -> Option<Element<'a, Message>> {
    if !visible {
        return None;
    }

    let mut notice = Notice::new(tokens, Tone::Danger, description, direction)
        .title(title)
        .action(action_label, on_action);

    if let Some(msg) = on_dismiss {
        notice = notice.dismiss(msg);
    }

    Some(notice.render())
}
// ANCHOR_END: recipe_error_recovery

impl App {
    #[rustfmt::skip]
    fn view(&self, direction: snora::LayoutDirection) -> iced::Element<'_, Message> {
let mut main_col = column![];
// ANCHOR: recipe_error_recovery_use
if let Some(notice) = error_recovery_notice(
    &self.tokens,
    self.sync_error.is_some(),
    "Sync failed",
    "Could not reach the server. Check your connection.",
    "Retry",
    Message::RetrySyncing,
    Some(Message::DismissSyncError),
    direction,
) {
    main_col = main_col.push(notice);
}
// ANCHOR_END: recipe_error_recovery_use
main_col.into()
}
}
