//! Feedback & overlay surfaces — `context_menu`, `dialog`, `toasts`,
//! `bottom_sheet`.
//!
//! These producers each read the App's overlay state and yield either `None`
//! (no surface this tick) or a snora-shaped struct / Element that the `Section`
//! body hands to the framework via `PageContract`. Keeping them here rather
//! than inside `section.rs` keeps each surface a plain function you can read
//! top-to-bottom without chasing trait impls.

use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use snora::{BottomSheet, Dialog, Toast};

use super::{App, Message, ToastFlavor};

// --------------------------------------------------------------------------
// Toast: convert App-side ToastData into snora's Toast<Message> each frame,
// baking the per-toast Dismiss message into each entry.
// --------------------------------------------------------------------------

pub fn toasts(app: &App) -> Vec<Toast<Message>> {
    app.toasts
        .iter()
        .map(|t| Toast {
            title: t.title.clone(),
            message: t.message.clone(),
            intent: t.intent,
            on_close: Message::DismissToast(t.id),
        })
        .collect()
}

// --------------------------------------------------------------------------
// Dialog: a centered "About" card, shown whenever `show_dialog` is true.
// The framework centers it for us and installs the dim backdrop; we only
// hand it the card content and an outside-click sink.
// --------------------------------------------------------------------------

pub fn dialog(app: &App) -> Option<Dialog<Element<'_, Message>, Message>> {
    if !app.show_dialog {
        return None;
    }

    let card: Element<'_, Message> = container(
        column![
            text("About Snora Showcase").size(20),
            text(
                "Snora is an iced-based GUI framework that favours logical \
                 Start/End layouts, PageContract-driven overlays, and \
                 feature-gated dead-code elimination for icons. This dialog \
                 is rendered by the framework's modal stack."
            )
            .size(14),
            row![
                container(space()).width(Length::Fill),
                button(text("Close").size(12)).on_press(Message::CloseModals),
            ]
            .align_y(Center),
        ]
        .spacing(12),
    )
    .padding(Padding::from([20.0, 24.0]))
    .width(Length::Fixed(420.0))
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.18))),
        text_color: Some(Color::from_rgb(0.95, 0.95, 0.97)),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
        },
        ..Default::default()
    })
    .into();

    Some(Dialog {
        content: card,
        on_outside_click: Some(Message::CloseModals),
    })
}

// --------------------------------------------------------------------------
// Bottom sheet: a log viewer that covers the bottom third of the screen.
// Snora's `render_bottom_sheet` owns the top-click-to-close behaviour; we
// just provide the content + close sink.
// --------------------------------------------------------------------------

pub fn bottom_sheet(app: &App) -> Option<BottomSheet<Element<'_, Message>, Message>> {
    if !app.show_bottom_sheet {
        return None;
    }

    let mut log_col = column![
        row![
            text("Log").size(18),
            container(space()).width(Length::Fill),
            button(text("Close").size(12)).on_press(Message::CloseModals),
        ]
        .align_y(Center),
    ]
    .spacing(6);

    if app.logs.is_empty() {
        log_col = log_col.push(text("(no entries)").size(12));
    } else {
        // Newest first, cap at 20 to keep layout predictable.
        for entry in app.logs.iter().rev().take(20) {
            log_col = log_col.push(
                text(format!(
                    "[{}] {}: {}",
                    entry.timestamp, entry.intent, entry.message
                ))
                .size(12),
            );
        }
    }

    let content: Element<'_, Message> = container(log_col)
        .padding(Padding::from([16.0, 24.0]))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
            ..Default::default()
        })
        .into();

    Some(BottomSheet {
        content,
        on_close: Some(Message::CloseModals),
    })
}

// --------------------------------------------------------------------------
// Context menu: a small floating card positioned at the recorded click point.
// When `context_menu_pos` is `Some`, returning `Some(_)` here tells
// `render_app` to install its transparent click-outside backdrop — so
// `CloseMenus` (our on_close_menus sink) fires on any outside click.
// --------------------------------------------------------------------------

pub fn context_menu(app: &App) -> Option<Element<'_, Message>> {
    let pos = app.context_menu_pos?;

    let menu: Element<'_, Message> = container(
        column![
            button(text("Show Info Toast").size(12))
                .on_press(Message::ShowToast(ToastFlavor::Info)),
            button(text("Add Log Entry").size(12)).on_press(Message::AddLog),
            button(text("Dismiss").size(12)).on_press(Message::CloseMenus),
        ]
        .spacing(4),
    )
    .padding(8)
    .width(Length::Fixed(180.0))
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.18, 0.98))),
        text_color: Some(Color::from_rgb(0.95, 0.95, 0.97)),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
        },
        ..Default::default()
    })
    .into();

    // Position the menu near `pos` using Space-based padding. iced 0.14
    // doesn't expose absolute positioning for overlay layers, and good-enough
    // placement is fine for a showcase. The framework still owns the
    // click-outside backdrop regardless.
    let positioned = column![
        space().height(Length::Fixed(pos.y)),
        row![space().width(Length::Fixed(pos.x)), menu,],
    ];

    Some(
        container(positioned)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}
