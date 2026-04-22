//! Toast rendering and lifecycle utilities.
//!
//! This module exposes three concerns:
//!
//! 1. [`render_toasts`] — internal renderer used by [`crate::render::render`].
//!    Produces a stack-layer that anchors toasts at the bottom-*end*
//!    (bottom-right under LTR, bottom-left under RTL).
//! 2. [`subscription`] — a public helper that emits ticks for TTL sweep.
//!    Applications wire this into their `iced::Application::subscription`.
//! 3. [`sweep_expired`] — a public helper that drops expired transient
//!    toasts from the application's toast queue.
//!
//! Together, (2) and (3) move toast TTL bookkeeping from application code
//! into the framework — the app only stores a `Vec<Toast<Message>>` and
//! calls two one-liners.

use std::time::{Duration, Instant};

use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Shadow, Subscription,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text},
};

use snora_core::{LayoutDirection, Toast, ToastIntent, ToastLifetime};

/// Fixed toast width so stacked toast edges align cleanly regardless of
/// content length. The value is chosen to comfortably hold two lines of
/// 14pt text at default font sizes.
const TOAST_WIDTH: f32 = 340.0;

/// Default sweep interval. Half-second resolution is imperceptible to users
/// and keeps idle wakeups low.
const SWEEP_INTERVAL: Duration = Duration::from_millis(500);

// =========================================================================
// Rendering
// =========================================================================

/// Build the toast layer, or `None` if the queue is empty.
///
/// The layer is positioned at the bottom-*end* of the window:
///
/// * `Ltr` → bottom-right
/// * `Rtl` → bottom-left
pub(crate) fn render_toasts<'a, Message>(
    toasts: Vec<Toast<Message>>,
    direction: LayoutDirection,
) -> Option<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    if toasts.is_empty() {
        return None;
    }

    let mut stack_col = column![].spacing(8);
    for toast in toasts {
        stack_col = stack_col.push(render_single_toast(toast));
    }

    let horizontal_anchor = match direction {
        LayoutDirection::Ltr => Horizontal::Right,
        LayoutDirection::Rtl => Horizontal::Left,
    };

    Some(
        container(stack_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .align_x(horizontal_anchor)
            .align_y(Vertical::Bottom)
            .into(),
    )
}

fn render_single_toast<'a, Message>(toast: Toast<Message>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let intent = toast.intent;

    let text_col = column![
        text(toast.title).size(16),
        text(toast.message).size(14),
    ]
    .spacing(4);

    let close_btn = button(text("×").size(18))
        .on_press(toast.on_dismiss)
        .padding([0, 8])
        .style(|_theme, status| close_button_style(status));

    let body = row![container(text_col).width(Length::Fill), close_btn]
        .align_y(Center)
        .spacing(4);

    container(body)
        .width(Length::Fixed(TOAST_WIDTH))
        .padding(12)
        .style(move |theme| toast_style(theme, intent))
        .into()
}

/// Style a toast surface based on its intent. Colors are pulled from the
/// theme's extended palette where available, with a hand-picked warning
/// color (iced's extended palette has no `warning` pair of its own).
fn toast_style(theme: &iced::Theme, intent: ToastIntent) -> iced::widget::container::Style {
    use iced::widget::container::Style;

    let ep = theme.extended_palette();
    let (background, text_color) = match intent {
        ToastIntent::Debug => (ep.background.strong.color, ep.background.strong.text),
        ToastIntent::Info => (ep.primary.base.color, ep.primary.base.text),
        ToastIntent::Success => (ep.success.base.color, ep.success.base.text),
        // iced's extended palette has no `warning`; use a stable orange
        // that remains readable against both light and dark themes.
        ToastIntent::Warning => (Color::from_rgb8(0xD9, 0x77, 0x06), Color::WHITE),
        ToastIntent::Error => (ep.danger.base.color, ep.danger.base.text),
    };

    Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_color),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

fn close_button_style(status: button::Status) -> button::Style {
    let alpha = match status {
        button::Status::Hovered => 1.0,
        _ => 0.75,
    };
    button::Style {
        background: None,
        text_color: Color {
            a: alpha,
            ..Color::WHITE
        },
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

// =========================================================================
// Lifecycle helpers
// =========================================================================

/// Subscribe to periodic ticks for TTL sweep.
///
/// Wire this into your `iced::Application::subscription` like so:
///
/// ```ignore
/// fn subscription(&self) -> Subscription<Message> {
///     snora::toast::subscription(&self.toasts, || Message::ToastTick)
/// }
///
/// // ...
/// fn update(&mut self, msg: Message) -> Task<Message> {
///     match msg {
///         Message::ToastTick => {
///             snora::toast::sweep_expired(&mut self.toasts, std::time::Instant::now());
///         }
///         // ...
///     }
///     Task::none()
/// }
/// ```
///
/// The subscription is only active while at least one *transient* toast
/// is present. An all-persistent or empty queue returns
/// [`Subscription::none`] so the runtime does not wake for nothing.
///
/// The `tick_message` closure must be `Clone` because iced clones it for
/// each wake-up. The simplest form is a zero-capture closure like
/// `|| Message::ToastTick`.
pub fn subscription<Message, F>(
    toasts: &[Toast<Message>],
    tick_message: F,
) -> Subscription<Message>
where
    Message: Clone + Send + 'static,
    F: Fn() -> Message + Send + Sync + Clone + 'static,
{
    let has_transient = toasts
        .iter()
        .any(|t| matches!(t.lifetime, ToastLifetime::Transient(_)));
    if has_transient {
        iced::time::every(SWEEP_INTERVAL).map(move |_| tick_message())
    } else {
        Subscription::none()
    }
}

/// Drop expired transient toasts from the queue.
///
/// Persistent toasts are always retained. Call this from your update
/// function when handling the tick message produced by [`subscription`].
pub fn sweep_expired<Message: Clone>(toasts: &mut Vec<Toast<Message>>, now: Instant) {
    toasts.retain(|t| !t.is_expired(now));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_drops_only_expired_transient() {
        let base = Instant::now();

        let live_transient = Toast::new(1, ToastIntent::Info, "a", "b", ())
            .with_lifetime(ToastLifetime::seconds(10))
            .with_created_at(base);
        let dead_transient = Toast::new(2, ToastIntent::Info, "a", "b", ())
            .with_lifetime(ToastLifetime::millis(100))
            .with_created_at(base);
        let persistent = Toast::new(3, ToastIntent::Error, "a", "b", ())
            .persistent()
            .with_created_at(base);

        let mut v = vec![live_transient, dead_transient, persistent];
        sweep_expired(&mut v, base + Duration::from_secs(1));

        let remaining_ids: Vec<u64> = v.iter().map(|t| t.id).collect();
        assert_eq!(remaining_ids, vec![1, 3]);
    }
}
