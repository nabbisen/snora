use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{button, column, container, row, scrollable, space, text},
};
use snora_core::contract::stack::ToastIntent;

use crate::style::container_box_style;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub intent: ToastIntent,
    pub timestamp: String,
    pub message: String,
}

pub fn app_footer<'a, Message: Clone + 'a>(
    status_text: &'a str,
    logs: &'a [LogEntry],
    is_log_expanded: bool,
    active_filter: Option<ToastIntent>,
    on_toggle_log: Message,
) -> Element<'a, Message> {
    let mut main_col = column![];

    if is_log_expanded {
        let mut log_col = column![].spacing(4).width(Length::Fill);
        for log in logs
            .iter()
            .filter(|l| active_filter.map_or(true, |f| f == l.intent))
        {
            log_col = log_col
                .push(row![text(&log.timestamp).size(12), text(&log.message).size(13),].spacing(8));
        }

        main_col = main_col.push(
            container(scrollable(log_col).height(Length::Fixed(150.0)))
                .width(Length::Fill)
                .padding(16)
                .style(container_box_style),
        );
    }

    let error_count = logs
        .iter()
        .filter(|l| l.intent == ToastIntent::Error)
        .count();
    let toggle_btn_text = if is_log_expanded {
        "▼ Hide Logs"
    } else {
        "▲ Show Logs"
    };

    let status_bar = container(
        row![
            text(status_text).size(12),
            container(space()).width(Length::Fill),
            text(format!("{} Errors", error_count)).size(12),
            container(space()).width(Length::Fixed(16.0)),
            button(text(toggle_btn_text).size(12))
                .padding(Padding::from([4.0, 8.0]))
                .on_press(on_toggle_log)
        ]
        .align_y(Center),
    )
    .width(Length::Fill)
    .padding(Padding::from([6.0, 16.0]))
    .style(container_box_style);

    main_col.push(status_bar).into()
}
