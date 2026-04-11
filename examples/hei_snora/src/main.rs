use iced::{
    Element, Length, Task,
    widget::{button, column, container, text},
};
use snora::components::footer::{LogEntry, standard_footer};
use snora::components::header::standard_header;
use snora::icons;
use snora::style::container_box_style;
use snora::{Icon, LayoutDirection, MenuItem, PageLayout};

pub fn main() -> iced::Result {
    iced::application(HeiSnora::new, HeiSnora::update, HeiSnora::view)
        .title(HeiSnora::title)
        .run()
}

struct HeiSnora {
    direction: LayoutDirection,
    is_log_expanded: bool,
    logs: Vec<LogEntry>,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleDirection,
    ToggleLog,
    MenuAction(&'static str),
}

impl HeiSnora {
    fn new() -> Self {
        Self {
            direction: LayoutDirection::Ltr,
            is_log_expanded: false,
            logs: vec![],
        }
    }

    fn title(&self) -> String {
        "Hello Snora Framework".into()
    }

    fn update(state: &mut Self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDirection => {
                state.direction = match state.direction {
                    LayoutDirection::Ltr => LayoutDirection::Rtl,
                    LayoutDirection::Rtl => LayoutDirection::Ltr,
                };
            }
            Message::ToggleLog => {
                state.is_log_expanded = !state.is_log_expanded;
            }
            Message::MenuAction(name) => {
                state.logs.push(LogEntry {
                    intent: snora::ToastIntent::Info,
                    timestamp: "Just now".into(),
                    message: format!("Clicked: {}", name),
                });
            }
        }
        // Return Task::none() since we have no async tasks
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let header = standard_header(
            "Snora App",
            vec![
                MenuItem {
                    label: "File".into(),
                    icon: Some(Icon::Lucide(icons::FileText)),
                    action: Some(Message::MenuAction("File")),
                },
                MenuItem {
                    label: "Settings".into(),
                    icon: Some(Icon::Lucide(icons::Settings)),
                    action: Some(Message::MenuAction("Settings")),
                },
                MenuItem {
                    label: "Emoji".into(),
                    icon: Some("🎉".into()),
                    action: Some(Message::MenuAction("Emoji")),
                },
            ],
            None,
        );

        let footer = standard_footer(
            "Ready",
            &self.logs,
            self.is_log_expanded,
            None,
            Message::ToggleLog,
        );

        let body = container(
            column![
                text("Welcome to Snora Framework!").size(24),
                button("Toggle LTR / RTL").on_press(Message::ToggleDirection)
            ]
            .spacing(20),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        let aside = container(text("Sidebar Area"))
            .width(Length::Fixed(200.0))
            .height(Length::Fill)
            .style(container_box_style)
            .into();

        let layout = PageLayout {
            direction: self.direction,
            header: Some(header),
            body,
            aside: Some(aside),
            footer: Some(footer),
            dialog: None,
            toasts: vec![],
        };

        snora::layout::build_layout(layout)
    }
}

// use iced::widget::{button, column, container, text};
// use iced::{Element, Length, Settings};
// use snora::components::footer::{LogEntry, standard_footer};
// use snora::components::header::standard_header;
// use snora::icons;
// use snora::style::container_box_style;
// use snora::{Icon, LayoutDirection, MenuItem, PageLayout};

// pub fn main() -> iced::Result {
//     HeiSnora::run(Settings::default())
// }

// struct HeiSnora {
//     direction: LayoutDirection,
//     is_log_expanded: bool,
//     logs: Vec<LogEntry>,
// }

// #[derive(Debug, Clone)]
// enum Message {
//     ToggleDirection,
//     ToggleLog,
//     MenuAction(&'static str),
// }

// impl Sandbox for HeiSnora {
//     type Message = Message;

//     fn new() -> Self {
//         Self {
//             direction: LayoutDirection::Ltr,
//             is_log_expanded: false,
//             logs: vec![],
//         }
//     }

//     fn title(&self) -> String {
//         String::from("Hello Snora Framework")
//     }

//     fn update(&mut self, message: Message) {
//         match message {
//             Message::ToggleDirection => {
//                 self.direction = match self.direction {
//                     LayoutDirection::Ltr => LayoutDirection::Rtl,
//                     LayoutDirection::Rtl => LayoutDirection::Ltr,
//                 };
//             }
//             Message::ToggleLog => {
//                 self.is_log_expanded = !self.is_log_expanded;
//             }
//             Message::MenuAction(name) => {
//                 self.logs.push(LogEntry {
//                     intent: snora::ToastIntent::Info,
//                     timestamp: "Just now".into(),
//                     message: format!("Clicked: {}", name),
//                 });
//             }
//         }
//     }

//     fn view(&self) -> Element<Message> {
//         let header = standard_header(
//             "Snora App",
//             vec![
//                 MenuItem {
//                     label: "File".into(),
//                     icon: Some(Icon::Lucide(icons::FileText)),
//                     action: Some(Message::MenuAction("File")),
//                 },
//                 MenuItem {
//                     label: "Settings".into(),
//                     icon: Some(Icon::Lucide(icons::Settings)),
//                     action: Some(Message::MenuAction("Settings")),
//                 },
//                 MenuItem {
//                     label: "Emoji".into(),
//                     icon: Some("🎉".into()), // From<&str> による自動変換
//                     action: Some(Message::MenuAction("Emoji")),
//                 },
//             ],
//             None,
//         );

//         let footer = standard_footer(
//             "Ready",
//             &self.logs,
//             self.is_log_expanded,
//             None,
//             Message::ToggleLog,
//         );

//         let body = container(
//             column![
//                 text("Welcome to Snora Framework!").size(24),
//                 button("Toggle LTR / RTL").on_press(Message::ToggleDirection)
//             ]
//             .spacing(20),
//         )
//         .center_x(Length::Fill)
//         .center_y(Length::Fill)
//         .into();

//         let aside = container(text("Sidebar Area"))
//             .width(Length::Fixed(200.0))
//             .height(Length::Fill)
//             .style(container_box_style)
//             .into();

//         let layout = PageLayout {
//             direction: self.direction,
//             header: Some(header),
//             body,
//             aside: Some(aside),
//             footer: Some(footer),
//             dialog: None,
//             toasts: vec![],
//         };

//         snora::layout::build_layout(layout)
//     }
// }
