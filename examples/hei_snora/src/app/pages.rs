//! The three showcase views. `body` dispatches to one of them based on the
//! sidebar's active view id; each is a plain `Element` producer.
//!
//! Together they exercise: toasts (all four intents), dialog, bottom sheet,
//! context menu, search input with submit, and read-only state reporting.
//! Nothing here knows about the framework's composition rules — that's the
//! whole point of the `PageContract` / `Section` layer above.

use iced::{
    Alignment::Center,
    Element, Length, Padding, Point,
    widget::{button, column, container, row, space, text, text_input},
};

use super::{App, Message, ToastFlavor, ViewId};

pub fn body(app: &App) -> Element<'_, Message> {
    // Match on a reference because `ViewId` isn't Copy (the variants carry no
    // data, but the derive is Clone/Eq only — mirroring chrome.rs).
    match &app.active_view_id {
        ViewId::Home => home(app),
        ViewId::Search => search(app),
        ViewId::Settings => settings(app),
    }
}

// --------------------------------------------------------------------------
// Home — the headline view. Buttons for every feedback surface so a user can
// poke each overlay from one place.
// --------------------------------------------------------------------------

fn home(_app: &App) -> Element<'_, Message> {
    let intro = column![
        text("Home").size(28),
        text(
            "This is the Snora framework showcase. Switch views from the sidebar, \
             open menus from the header, and use the buttons below to pop each \
             feedback surface the framework defines."
        )
        .size(14),
    ]
    .spacing(8);

    let toast_row = row![
        text("Toasts:").size(14),
        button(text("Info").size(12)).on_press(Message::ShowToast(ToastFlavor::Info)),
        button(text("Success").size(12)).on_press(Message::ShowToast(ToastFlavor::Success)),
        button(text("Warning").size(12)).on_press(Message::ShowToast(ToastFlavor::Warning)),
        button(text("Error").size(12)).on_press(Message::ShowToast(ToastFlavor::Error)),
    ]
    .spacing(8)
    .align_y(Center);

    let overlay_row = row![
        text("Overlays:").size(14),
        button(text("Open Dialog").size(12)).on_press(Message::OpenDialog),
        button(text("Toggle Bottom Sheet").size(12)).on_press(Message::ToggleSheet),
        // A real right-click would be nicer, but iced 0.14 doesn't expose the
        // pointer position alongside the press event, so we anchor the context
        // menu at a fixed demo point.
        button(text("Open Context Menu").size(12))
            .on_press(Message::OpenContextMenu(Point::new(320.0, 260.0))),
    ]
    .spacing(8)
    .align_y(Center);

    let misc_row = row![
        text("Misc:").size(14),
        button(text("Add Log Entry").size(12)).on_press(Message::AddLog),
    ]
    .spacing(8)
    .align_y(Center);

    container(
        column![
            intro,
            space().height(Length::Fixed(16.0)),
            toast_row,
            overlay_row,
            misc_row,
        ]
        .spacing(12),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// --------------------------------------------------------------------------
// Search — exercises bound input state + submit. The submit handler in
// update.rs emits a warning toast for empty input and a success toast
// otherwise, so the full loop (input → state → toast) is visible.
// --------------------------------------------------------------------------

fn search(app: &App) -> Element<'_, Message> {
    let header = column![
        text("Search").size(28),
        text("Type and press Enter (or click Submit) to dispatch a search.").size(14),
    ]
    .spacing(8);

    let input = text_input("query…", &app.search_query)
        .on_input(Message::SearchChanged)
        .on_submit(Message::SubmitSearch)
        .padding(8);

    let input_row = row![
        container(input).width(Length::Fill),
        button(text("Submit").size(12)).on_press(Message::SubmitSearch),
    ]
    .spacing(8)
    .align_y(Center);

    let hint = text(if app.search_query.is_empty() {
        "(nothing typed yet)".to_string()
    } else {
        format!("Current query: “{}”", app.search_query)
    })
    .size(12);

    container(
        column![
            header,
            space().height(Length::Fixed(16.0)),
            input_row,
            hint,
        ]
        .spacing(12),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// --------------------------------------------------------------------------
// Settings — read-only status pane plus the framework-level direction toggle.
// Useful for confirming that state the header/footer display matches the
// App's actual model.
// --------------------------------------------------------------------------

fn settings(app: &App) -> Element<'_, Message> {
    let header = column![
        text("Settings").size(28),
        text("Framework-level configuration and live status.").size(14),
    ]
    .spacing(8);

    let direction_row = row![
        text(format!("Layout direction: {:?}", app.direction)).size(14),
        button(text("Flip LTR ↔ RTL").size(12)).on_press(Message::ToggleDirection),
    ]
    .spacing(12)
    .align_y(Center);

    let stats = column![
        text(format!("Logs queued:   {}", app.logs.len())).size(13),
        text(format!("Toasts queued: {}", app.toasts.len())).size(13),
        text(format!("Dialog open:   {}", app.show_dialog)).size(13),
        text(format!("Sheet open:    {}", app.show_bottom_sheet)).size(13),
        text(format!(
            "Active menu:   {}",
            match &app.active_menu_id {
                Some(id) => format!("{}", id),
                None => "—".to_string(),
            }
        ))
        .size(13),
    ]
    .spacing(4);

    container(
        column![
            header,
            space().height(Length::Fixed(16.0)),
            direction_row,
            space().height(Length::Fixed(12.0)),
            text("Status").size(16),
            stats,
        ]
        .spacing(12),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
