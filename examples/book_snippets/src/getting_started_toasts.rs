//! Anchored source for `docs/src/getting-started/04-toasts.md` (RFC-136): the application, in the pieces the page shows.

// ANCHOR: toasts_state
use iced::{Subscription, Task};
use snora::toast::Holds;
use snora::{Toast, ToastAttention, ToastIntent, ToastLifetime};
use std::time::Instant;

struct App {
    toasts: Vec<Toast<Message>>,
    holds: Holds, // which toasts the reader is attending to
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Message {
    ShowSaved,
    Dismiss(u64),
    ToastAttention(u64, ToastAttention), // pointer or keyboard came or went
    ToastTick,                           // framework asks us to sweep
}
// ANCHOR_END: toasts_state

impl App {
    #[rustfmt::skip]
    // ANCHOR: toasts_update
    fn update(&mut self, msg: Message) -> Task<Message> {
    if let Message::ShowSaved = msg {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast::new(
            id,
            ToastIntent::Success,
            "Saved",
            "Document written to disk.",
            Message::Dismiss(id),
        ));
    }
    if let Message::Dismiss(id) = msg {
        self.toasts.retain(|t| t.id != id);
    }
    if let Message::ToastAttention(id, attention) = msg {
        // Held while the pointer rests on it or the keyboard is on its ×;
        // when both have gone, the countdown restarts in full. `Holds`
        // does the restart itself.
        self.holds.attend(id, attention, &mut self.toasts, Instant::now());
    }
    if let Message::ToastTick = msg {
        self.holds.sweep(&mut self.toasts, Instant::now());
    }
    Task::none()
}
    // ANCHOR_END: toasts_update
}

impl App {
    #[rustfmt::skip]
    // ANCHOR: toasts_subscription
    fn subscription(&self) -> Subscription<Message> {
    snora::toast::subscription(&self.toasts, || Message::ToastTick)
}
    // ANCHOR_END: toasts_subscription
}

impl App {
    // The reader's own content.
    fn my_body(&self) -> iced::Element<'_, Message> {
        iced::widget::space().into()
    }

#[rustfmt::skip]
    // ANCHOR: toasts_view
    fn view(&self) -> iced::Element<'_, Message> {
    let body: iced::Element<'_, Message> = self.my_body();
    let layout = snora::AppLayout::new(body)
        .toasts(self.toasts.clone())
        .on_toast_attention(Message::ToastAttention);
    snora::render(layout)
}
    // ANCHOR_END: toasts_view
}
